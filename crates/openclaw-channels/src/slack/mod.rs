//! Slack channel adapter using the Web API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use openclaw_core::secrets::ApiKey;
use openclaw_core::types::{
    Attachment, AttachmentKind, ChannelId, DeliveryResult, Message, PeerId, PeerType,
};

use crate::traits::{
    Channel, ChannelCapabilities, ChannelContext, ChannelError, ChannelInbound, ChannelOutbound,
    ChannelProbe, DeliveryMode, OutboundContext,
};

const SLACK_API_BASE: &str = "https://slack.com/api";

/// Slack channel adapter.
pub struct SlackChannel {
    client: Client,
    token: ApiKey,
    state: Arc<RwLock<SlackState>>,
}

#[derive(Debug, Default)]
struct SlackState {
    bot_id: Option<String>,
    bot_user_id: Option<String>,
    team_id: Option<String>,
    team_name: Option<String>,
    connected: bool,
}

impl SlackChannel {
    /// Create a new Slack channel.
    #[must_use]
    pub fn new(token: ApiKey) -> Self {
        Self {
            client: Client::new(),
            token,
            state: Arc::new(RwLock::new(SlackState::default())),
        }
    }

    /// Call a Slack Web API method.
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Option<&impl Serialize>,
    ) -> Result<T, ChannelError> {
        let url = format!("{SLACK_API_BASE}/{method}");

        let mut request = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token.expose()))
            .header("Content-Type", "application/json; charset=utf-8");

        if let Some(p) = params {
            request = request.json(p);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 429 {
                return Err(ChannelError::RateLimited);
            }
            let text = response.text().await.unwrap_or_default();
            return Err(ChannelError::Network(format!("{status}: {text}")));
        }

        let result: SlackResponse<T> = response
            .json()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if result.ok {
            Ok(result.data)
        } else {
            Err(ChannelError::Network(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }
}

#[async_trait]
impl Channel for SlackChannel {
    fn id(&self) -> &'static str {
        "slack"
    }

    fn label(&self) -> &'static str {
        "Slack"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            text: true,
            images: true,
            videos: true,
            voice: false,
            files: true,
            threads: true,
            reactions: true,
            editing: true,
            deletion: true,
        }
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), ChannelError> {
        // Get bot identity using auth.test
        let auth: AuthTestResponse = self.call("auth.test", None::<&()>).await?;

        let mut state = self.state.write().await;
        state.bot_id = Some(auth.bot_id.unwrap_or_default());
        state.bot_user_id = Some(auth.user_id);
        state.team_id = Some(auth.team_id);
        state.team_name = Some(auth.team.unwrap_or_default());
        state.connected = true;

        tracing::info!(
            "Slack bot connected: {} in {}",
            auth.user.unwrap_or_default(),
            state.team_name.as_deref().unwrap_or("unknown")
        );
        Ok(())
    }

    async fn stop(&self) -> Result<(), ChannelError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }

    async fn probe(&self) -> Result<ChannelProbe, ChannelError> {
        match self
            .call::<AuthTestResponse>("auth.test", None::<&()>)
            .await
        {
            Ok(auth) => Ok(ChannelProbe {
                connected: true,
                account_id: Some(auth.user_id),
                display_name: auth.user,
                error: None,
            }),
            Err(e) => Ok(ChannelProbe {
                connected: false,
                account_id: None,
                display_name: None,
                error: Some(e.to_string()),
            }),
        }
    }
}

#[async_trait]
impl ChannelOutbound for SlackChannel {
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError> {
        let params = ChatPostMessageParams {
            channel: ctx.chat_id.clone(),
            text: Some(text.to_string()),
            thread_ts: ctx.thread_id.clone(),
            reply_broadcast: None,
            blocks: None,
            attachments: None,
        };

        let result: ChatPostMessageResponse = self.call("chat.postMessage", Some(&params)).await?;

        Ok(DeliveryResult {
            message_id: result.ts.clone(),
            channel: ChannelId::slack(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(result.channel),
            meta: None,
        })
    }

    async fn send_media(
        &self,
        ctx: OutboundContext,
        media: &[Attachment],
    ) -> Result<DeliveryResult, ChannelError> {
        // Send media as Slack attachments with URLs
        let slack_attachments: Vec<SlackAttachment> = media
            .iter()
            .map(|a| SlackAttachment {
                fallback: a.filename.clone(),
                image_url: if matches!(a.kind, AttachmentKind::Image) {
                    Some(a.url.clone())
                } else {
                    None
                },
                video_url: if matches!(a.kind, AttachmentKind::Video) {
                    Some(a.url.clone())
                } else {
                    None
                },
                title: a.filename.clone(),
                title_link: Some(a.url.clone()),
            })
            .collect();

        let params = ChatPostMessageParams {
            channel: ctx.chat_id.clone(),
            text: None,
            thread_ts: ctx.thread_id.clone(),
            reply_broadcast: None,
            blocks: None,
            attachments: Some(slack_attachments),
        };

        let result: ChatPostMessageResponse = self.call("chat.postMessage", Some(&params)).await?;

        Ok(DeliveryResult {
            message_id: result.ts.clone(),
            channel: ChannelId::slack(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(result.channel),
            meta: None,
        })
    }

    fn text_chunk_limit(&self) -> usize {
        40000 // Slack text limit (blocks have different limits)
    }

    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::Immediate
    }
}

#[async_trait]
impl ChannelInbound for SlackChannel {
    type RawMessage = SlackEvent;

    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError> {
        let raw_value = serde_json::to_value(&raw).unwrap_or_default();

        let event = raw
            .event
            .ok_or_else(|| ChannelError::Config("No event data".to_string()))?;

        // Only handle message events
        if event.event_type != "message" {
            return Err(ChannelError::Config("Not a message event".to_string()));
        }

        let user_id = event
            .user
            .ok_or_else(|| ChannelError::Config("No user in message".to_string()))?;

        // Determine peer type
        let peer_type = if event.channel_type.as_deref() == Some("im") {
            PeerType::Dm
        } else if event.thread_ts.is_some() {
            PeerType::Thread
        } else {
            PeerType::Group
        };

        let state = futures::executor::block_on(self.state.read());
        let account_id = state.bot_user_id.clone().unwrap_or_default();

        // Convert files to attachments
        let attachments = event
            .files
            .unwrap_or_default()
            .into_iter()
            .map(|f| {
                let kind = if f.mimetype.starts_with("image/") {
                    AttachmentKind::Image
                } else if f.mimetype.starts_with("video/") {
                    AttachmentKind::Video
                } else if f.mimetype.starts_with("audio/") {
                    AttachmentKind::Audio
                } else {
                    AttachmentKind::Document
                };

                Attachment {
                    kind,
                    url: f.url_private.unwrap_or_default(),
                    mime_type: Some(f.mimetype),
                    filename: Some(f.name),
                    size: Some(f.size as u64),
                    thumbnail_url: f.thumb_360,
                }
            })
            .collect();

        // Parse timestamp from Slack ts format (e.g., "1234567890.123456")
        let timestamp = event
            .ts
            .as_ref()
            .and_then(|ts| ts.split('.').next())
            .and_then(|s| s.parse::<i64>().ok())
            .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
            .unwrap_or_else(chrono::Utc::now);

        Ok(Message {
            id: event.ts.clone().unwrap_or_default(),
            channel: ChannelId::slack(),
            account_id,
            peer_id: PeerId::new(user_id),
            peer_type,
            content: event.text.unwrap_or_default(),
            attachments,
            timestamp,
            reply_to: None, // Slack uses thread_ts, not explicit replies
            thread_id: event.thread_ts,
            mentions: Vec::new(), // Would need to parse <@USER_ID> from text
            raw: Some(raw_value),
        })
    }

    async fn acknowledge(&self, _message_id: &str) -> Result<(), ChannelError> {
        // Slack Events API handles acknowledgement at HTTP level
        Ok(())
    }
}

// Slack API types

/// Generic Slack API response wrapper.
#[derive(Debug, Deserialize)]
struct SlackResponse<T> {
    ok: bool,
    #[serde(flatten)]
    data: T,
    error: Option<String>,
}

/// auth.test response.
#[derive(Debug, Deserialize)]
struct AuthTestResponse {
    user_id: String,
    user: Option<String>,
    team_id: String,
    team: Option<String>,
    bot_id: Option<String>,
}

/// chat.postMessage response.
#[derive(Debug, Deserialize)]
struct ChatPostMessageResponse {
    channel: String,
    ts: String,
}

/// chat.postMessage parameters.
#[derive(Debug, Serialize)]
struct ChatPostMessageParams {
    channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_broadcast: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocks: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<SlackAttachment>>,
}

/// Slack attachment (legacy format, still works).
#[derive(Debug, Serialize)]
struct SlackAttachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title_link: Option<String>,
}

/// Slack Events API event wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackEvent {
    /// Event type (always "`event_callback`" for events).
    #[serde(rename = "type")]
    pub event_type: String,
    /// Team ID.
    pub team_id: Option<String>,
    /// Event data.
    pub event: Option<SlackMessageEvent>,
}

/// Slack message event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessageEvent {
    /// Event type (e.g., "message").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Channel ID.
    pub channel: Option<String>,
    /// Channel type (e.g., "im", "channel", "group").
    pub channel_type: Option<String>,
    /// User ID who sent the message.
    pub user: Option<String>,
    /// Message text.
    pub text: Option<String>,
    /// Message timestamp (unique ID).
    pub ts: Option<String>,
    /// Thread timestamp (if in a thread).
    pub thread_ts: Option<String>,
    /// Attached files.
    pub files: Option<Vec<SlackFile>>,
    /// Message subtype (e.g., "`bot_message`").
    pub subtype: Option<String>,
}

/// Slack file object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackFile {
    /// File ID.
    pub id: String,
    /// Filename.
    pub name: String,
    /// MIME type.
    pub mimetype: String,
    /// File size in bytes.
    pub size: i64,
    /// Private download URL.
    pub url_private: Option<String>,
    /// Thumbnail URL (360px).
    pub thumb_360: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let channel = SlackChannel::new(ApiKey::new("test".to_string()));
        assert_eq!(channel.id(), "slack");
    }

    #[test]
    fn test_capabilities() {
        let channel = SlackChannel::new(ApiKey::new("test".to_string()));
        let caps = channel.capabilities();
        assert!(caps.text);
        assert!(caps.files);
        assert!(caps.threads);
        assert!(caps.reactions);
    }

    #[test]
    fn test_text_limit() {
        let channel = SlackChannel::new(ApiKey::new("test".to_string()));
        assert_eq!(channel.text_chunk_limit(), 40000);
    }
}

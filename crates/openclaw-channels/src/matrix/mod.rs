//! Matrix channel adapter using the Client-Server API.

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

/// Matrix channel adapter.
pub struct MatrixChannel {
    client: Client,
    homeserver_url: String,
    access_token: ApiKey,
    state: Arc<RwLock<MatrixState>>,
}

#[derive(Debug, Default)]
struct MatrixState {
    user_id: Option<String>,
    device_id: Option<String>,
    connected: bool,
    next_batch: Option<String>,
}

impl MatrixChannel {
    /// Create a new Matrix channel.
    ///
    /// # Arguments
    /// * `homeserver_url` - URL of the Matrix homeserver (e.g., "https://matrix.org")
    /// * `access_token` - Access token for authentication
    #[must_use]
    pub fn new(homeserver_url: impl Into<String>, access_token: ApiKey) -> Self {
        Self {
            client: Client::new(),
            homeserver_url: homeserver_url.into(),
            access_token,
            state: Arc::new(RwLock::new(MatrixState::default())),
        }
    }

    /// Build API URL.
    fn api_url(&self, path: &str) -> String {
        format!("{}/_matrix/client/v3{}", self.homeserver_url, path)
    }

    /// Call a Matrix API endpoint.
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ChannelError> {
        let url = self.api_url(path);

        let mut request = self
            .client
            .request(method, &url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.expose()),
            )
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            request = request.json(b);
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
            return Err(ChannelError::Network(format!("{}: {}", status, text)));
        }

        response
            .json()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))
    }

    /// Generate a transaction ID for idempotency.
    fn txn_id() -> String {
        format!("openclaw_{}", uuid::Uuid::new_v4())
    }
}

#[async_trait]
impl Channel for MatrixChannel {
    fn id(&self) -> &str {
        "matrix"
    }

    fn label(&self) -> &str {
        "Matrix"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            text: true,
            images: true,
            videos: true,
            voice: true,
            files: true,
            threads: true, // Matrix has reply threads
            reactions: true,
            editing: true,
            deletion: true,
        }
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), ChannelError> {
        // Get whoami to verify credentials
        let whoami: WhoAmIResponse = self
            .call(reqwest::Method::GET, "/account/whoami", None::<&()>)
            .await?;

        let mut state = self.state.write().await;
        state.user_id = Some(whoami.user_id.clone());
        state.device_id = whoami.device_id;
        state.connected = true;

        tracing::info!("Matrix connected: {}", whoami.user_id);
        Ok(())
    }

    async fn stop(&self) -> Result<(), ChannelError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }

    async fn probe(&self) -> Result<ChannelProbe, ChannelError> {
        match self
            .call::<WhoAmIResponse>(reqwest::Method::GET, "/account/whoami", None::<&()>)
            .await
        {
            Ok(whoami) => Ok(ChannelProbe {
                connected: true,
                account_id: Some(whoami.user_id.clone()),
                display_name: Some(whoami.user_id),
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
impl ChannelOutbound for MatrixChannel {
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError> {
        let room_id = urlencoding::encode(&ctx.chat_id);
        let txn_id = Self::txn_id();
        let path = format!("/rooms/{}/send/m.room.message/{}", room_id, txn_id);

        let mut content = MessageContent {
            msgtype: "m.text".to_string(),
            body: text.to_string(),
            format: Some("org.matrix.custom.html".to_string()),
            formatted_body: Some(text.to_string()),
            relates_to: None,
        };

        // Add reply relation if replying
        if let Some(reply_to) = ctx.reply_to {
            content.relates_to = Some(RelatesTo {
                in_reply_to: Some(InReplyTo { event_id: reply_to }),
                rel_type: None,
                event_id: None,
            });
        }

        let result: SendEventResponse = self
            .call(reqwest::Method::PUT, &path, Some(&content))
            .await?;

        Ok(DeliveryResult {
            message_id: result.event_id.clone(),
            channel: ChannelId::matrix(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(ctx.chat_id),
            meta: None,
        })
    }

    async fn send_media(
        &self,
        ctx: OutboundContext,
        media: &[Attachment],
    ) -> Result<DeliveryResult, ChannelError> {
        let room_id = urlencoding::encode(&ctx.chat_id);
        let mut last_event_id = String::new();

        for attachment in media {
            let txn_id = Self::txn_id();
            let path = format!("/rooms/{}/send/m.room.message/{}", room_id, txn_id);

            let msgtype = match attachment.kind {
                AttachmentKind::Image => "m.image",
                AttachmentKind::Video => "m.video",
                AttachmentKind::Audio | AttachmentKind::Voice => "m.audio",
                _ => "m.file",
            };

            let content = MediaMessageContent {
                msgtype: msgtype.to_string(),
                body: attachment
                    .filename
                    .clone()
                    .unwrap_or_else(|| "file".to_string()),
                url: attachment.url.clone(),
                info: Some(MediaInfo {
                    mimetype: attachment.mime_type.clone(),
                    size: attachment.size.map(|s| s as i64),
                    thumbnail_url: None,
                }),
            };

            let result: SendEventResponse = self
                .call(reqwest::Method::PUT, &path, Some(&content))
                .await?;

            last_event_id = result.event_id;
        }

        Ok(DeliveryResult {
            message_id: last_event_id,
            channel: ChannelId::matrix(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(ctx.chat_id),
            meta: None,
        })
    }

    fn text_chunk_limit(&self) -> usize {
        // Matrix doesn't have a strict limit, but events should be < 64KB
        60000
    }

    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::Immediate
    }
}

#[async_trait]
impl ChannelInbound for MatrixChannel {
    type RawMessage = MatrixEvent;

    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError> {
        let raw_value = serde_json::to_value(&raw).unwrap_or_default();

        let sender = raw
            .sender
            .ok_or_else(|| ChannelError::Config("No sender in event".to_string()))?;

        let _room_id = raw
            .room_id
            .ok_or_else(|| ChannelError::Config("No room_id in event".to_string()))?;

        let content = raw
            .content
            .ok_or_else(|| ChannelError::Config("No content in event".to_string()))?;

        let state = futures::executor::block_on(self.state.read());
        let account_id = state.user_id.clone().unwrap_or_default();

        // Extract text content
        let text = content.body.unwrap_or_default();

        // Handle attachments (m.image, m.video, m.audio, m.file)
        let attachments = if let Some(url) = content.url {
            let kind = match content.msgtype.as_deref() {
                Some("m.image") => AttachmentKind::Image,
                Some("m.video") => AttachmentKind::Video,
                Some("m.audio") => AttachmentKind::Audio,
                _ => AttachmentKind::Document,
            };

            vec![Attachment {
                kind,
                url,
                mime_type: content.info.as_ref().and_then(|i| i.mimetype.clone()),
                filename: Some(text.clone()),
                size: content.info.as_ref().and_then(|i| i.size.map(|s| s as u64)),
                thumbnail_url: content.info.and_then(|i| i.thumbnail_url),
            }]
        } else {
            Vec::new()
        };

        // Determine peer type (DM vs group)
        // In Matrix, room membership determines this, but we simplify
        let peer_type = PeerType::Group; // Most Matrix rooms are group-like

        // Parse timestamp from origin_server_ts (milliseconds)
        let timestamp = raw
            .origin_server_ts
            .and_then(|ts| chrono::DateTime::from_timestamp_millis(ts))
            .unwrap_or_else(chrono::Utc::now);

        // Extract reply-to from m.relates_to
        let reply_to = content
            .relates_to
            .and_then(|r| r.in_reply_to)
            .map(|r| r.event_id);

        Ok(Message {
            id: raw.event_id.unwrap_or_default(),
            channel: ChannelId::matrix(),
            account_id,
            peer_id: PeerId::new(sender),
            peer_type,
            content: text,
            attachments,
            timestamp,
            reply_to,
            thread_id: None, // Matrix uses reply chains, not explicit threads
            mentions: Vec::new(),
            raw: Some(raw_value),
        })
    }

    async fn acknowledge(&self, _message_id: &str) -> Result<(), ChannelError> {
        // Matrix uses read receipts, could implement with /receipt endpoint
        Ok(())
    }
}

// Matrix API types

/// whoami response.
#[derive(Debug, Deserialize)]
struct WhoAmIResponse {
    user_id: String,
    device_id: Option<String>,
}

/// Send event response.
#[derive(Debug, Deserialize)]
struct SendEventResponse {
    event_id: String,
}

/// Message content for m.room.message.
#[derive(Debug, Serialize)]
struct MessageContent {
    msgtype: String,
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    formatted_body: Option<String>,
    #[serde(rename = "m.relates_to")]
    #[serde(skip_serializing_if = "Option::is_none")]
    relates_to: Option<RelatesTo>,
}

/// Media message content.
#[derive(Debug, Serialize)]
struct MediaMessageContent {
    msgtype: String,
    body: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<MediaInfo>,
}

/// Media info.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediaInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    mimetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_url: Option<String>,
}

/// Relates-to for replies/threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelatesTo {
    #[serde(rename = "m.in_reply_to")]
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<InReplyTo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rel_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
}

/// In-reply-to reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InReplyTo {
    event_id: String,
}

/// Matrix room event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEvent {
    /// Event ID.
    pub event_id: Option<String>,
    /// Event type (e.g., "m.room.message").
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Room ID.
    pub room_id: Option<String>,
    /// Sender user ID.
    pub sender: Option<String>,
    /// Origin server timestamp (milliseconds).
    pub origin_server_ts: Option<i64>,
    /// Event content.
    pub content: Option<MatrixMessageContent>,
}

/// Matrix message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixMessageContent {
    /// Message type (e.g., "m.text", "m.image").
    pub msgtype: Option<String>,
    /// Message body.
    pub body: Option<String>,
    /// Media URL (for m.image, m.video, etc.).
    pub url: Option<String>,
    /// Media info.
    pub info: Option<MediaInfo>,
    /// Formatted body (HTML).
    pub formatted_body: Option<String>,
    /// Relations (replies, threads).
    #[serde(rename = "m.relates_to")]
    pub relates_to: Option<RelatesTo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let channel = MatrixChannel::new("https://matrix.org", ApiKey::new("test".to_string()));
        assert_eq!(channel.id(), "matrix");
    }

    #[test]
    fn test_capabilities() {
        let channel = MatrixChannel::new("https://matrix.org", ApiKey::new("test".to_string()));
        let caps = channel.capabilities();
        assert!(caps.text);
        assert!(caps.images);
        assert!(caps.reactions);
        assert!(caps.threads);
        assert!(caps.editing);
    }
}

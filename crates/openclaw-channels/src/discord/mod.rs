//! Discord channel adapter using the Bot REST API.

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

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord channel adapter.
pub struct DiscordChannel {
    client: Client,
    token: ApiKey,
    state: Arc<RwLock<DiscordState>>,
}

#[derive(Debug, Default)]
struct DiscordState {
    bot_id: Option<String>,
    username: Option<String>,
    discriminator: Option<String>,
    connected: bool,
}

impl DiscordChannel {
    /// Create a new Discord channel.
    #[must_use]
    pub fn new(token: ApiKey) -> Self {
        Self {
            client: Client::new(),
            token,
            state: Arc::new(RwLock::new(DiscordState::default())),
        }
    }

    /// Call a Discord API endpoint.
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ChannelError> {
        let url = format!("{DISCORD_API_BASE}{endpoint}");

        let mut request = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bot {}", self.token.expose()))
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
            return Err(ChannelError::Network(format!("{status}: {text}")));
        }

        response
            .json()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))
    }

    /// Call Discord API without expecting response body.
    async fn call_no_response(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
    ) -> Result<(), ChannelError> {
        let url = format!("{DISCORD_API_BASE}{endpoint}");

        let mut request = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bot {}", self.token.expose()))
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
            return Err(ChannelError::Network(format!("{status}: {text}")));
        }

        Ok(())
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn id(&self) -> &'static str {
        "discord"
    }

    fn label(&self) -> &'static str {
        "Discord"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            text: true,
            images: true,
            videos: true,
            voice: false, // Voice requires gateway, not REST
            files: true,
            threads: true,
            reactions: true,
            editing: true,
            deletion: true,
        }
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), ChannelError> {
        // Get current bot user
        let me: DiscordUser = self
            .call(reqwest::Method::GET, "/users/@me", None::<&()>)
            .await?;

        let mut state = self.state.write().await;
        state.bot_id = Some(me.id.clone());
        state.username = Some(me.username.clone());
        state.discriminator = me.discriminator;
        state.connected = true;

        tracing::info!("Discord bot connected: {}", me.username);
        Ok(())
    }

    async fn stop(&self) -> Result<(), ChannelError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }

    async fn probe(&self) -> Result<ChannelProbe, ChannelError> {
        match self
            .call::<DiscordUser>(reqwest::Method::GET, "/users/@me", None::<&()>)
            .await
        {
            Ok(me) => Ok(ChannelProbe {
                connected: true,
                account_id: Some(me.id),
                display_name: Some(me.username),
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
impl ChannelOutbound for DiscordChannel {
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError> {
        let endpoint = format!("/channels/{}/messages", ctx.chat_id);

        let params = CreateMessageParams {
            content: Some(text.to_string()),
            message_reference: ctx.reply_to.map(|id| MessageReference {
                message_id: Some(id),
                channel_id: None,
                guild_id: None,
            }),
            embeds: None,
            allowed_mentions: Some(AllowedMentions::default()),
        };

        let result: DiscordMessage = self
            .call(reqwest::Method::POST, &endpoint, Some(&params))
            .await?;

        Ok(DeliveryResult {
            message_id: result.id,
            channel: ChannelId::discord(),
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
        // For Discord, we can send embeds with URLs or upload files
        // Simplified: send as embeds with URLs
        let endpoint = format!("/channels/{}/messages", ctx.chat_id);

        let embeds: Vec<DiscordEmbed> = media
            .iter()
            .filter_map(|a| match a.kind {
                AttachmentKind::Image => Some(DiscordEmbed {
                    image: Some(EmbedImage { url: a.url.clone() }),
                    ..Default::default()
                }),
                AttachmentKind::Video => Some(DiscordEmbed {
                    video: Some(EmbedVideo { url: a.url.clone() }),
                    ..Default::default()
                }),
                _ => None,
            })
            .collect();

        let params = CreateMessageParams {
            content: None,
            message_reference: ctx.reply_to.map(|id| MessageReference {
                message_id: Some(id),
                channel_id: None,
                guild_id: None,
            }),
            embeds: if embeds.is_empty() {
                None
            } else {
                Some(embeds)
            },
            allowed_mentions: Some(AllowedMentions::default()),
        };

        let result: DiscordMessage = self
            .call(reqwest::Method::POST, &endpoint, Some(&params))
            .await?;

        Ok(DeliveryResult {
            message_id: result.id,
            channel: ChannelId::discord(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(ctx.chat_id),
            meta: None,
        })
    }

    fn text_chunk_limit(&self) -> usize {
        2000 // Discord message limit
    }

    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::Immediate
    }
}

#[async_trait]
impl ChannelInbound for DiscordChannel {
    type RawMessage = DiscordGatewayEvent;

    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError> {
        let raw_value = serde_json::to_value(&raw).unwrap_or_default();

        let msg = raw
            .d
            .ok_or_else(|| ChannelError::Config("No message data in event".to_string()))?;

        let author = msg
            .author
            .ok_or_else(|| ChannelError::Config("No author in message".to_string()))?;

        // Determine peer type based on guild presence and channel type
        let peer_type = if msg.guild_id.is_some() {
            if msg.thread.is_some() {
                PeerType::Thread
            } else {
                PeerType::Group
            }
        } else {
            PeerType::Dm
        };

        let state = futures::executor::block_on(self.state.read());
        let account_id = state.bot_id.clone().unwrap_or_default();

        // Convert attachments
        let attachments = msg
            .attachments
            .unwrap_or_default()
            .into_iter()
            .map(|a| {
                let kind = if a
                    .content_type
                    .as_ref()
                    .is_some_and(|ct| ct.starts_with("image/"))
                {
                    AttachmentKind::Image
                } else if a
                    .content_type
                    .as_ref()
                    .is_some_and(|ct| ct.starts_with("video/"))
                {
                    AttachmentKind::Video
                } else if a
                    .content_type
                    .as_ref()
                    .is_some_and(|ct| ct.starts_with("audio/"))
                {
                    AttachmentKind::Audio
                } else {
                    AttachmentKind::Document
                };

                Attachment {
                    kind,
                    url: a.url,
                    mime_type: a.content_type,
                    filename: Some(a.filename),
                    size: Some(a.size as u64),
                    thumbnail_url: a.proxy_url,
                }
            })
            .collect();

        // Extract mentions
        let mentions = msg
            .mentions
            .unwrap_or_default()
            .into_iter()
            .map(|u| u.id)
            .collect();

        // Parse timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339(&msg.timestamp)
            .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));

        Ok(Message {
            id: msg.id.clone(),
            channel: ChannelId::discord(),
            account_id,
            peer_id: PeerId::new(author.id),
            peer_type,
            content: msg.content.unwrap_or_default(),
            attachments,
            timestamp,
            reply_to: msg.message_reference.and_then(|r| r.message_id),
            thread_id: msg.thread.map(|t| t.id),
            mentions,
            raw: Some(raw_value),
        })
    }

    async fn acknowledge(&self, _message_id: &str) -> Result<(), ChannelError> {
        // Discord doesn't require explicit acknowledgement for bot messages
        Ok(())
    }
}

// Discord API types

/// Discord user object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    /// User's unique ID.
    pub id: String,
    /// User's username.
    pub username: String,
    /// User's discriminator (legacy, may be "0").
    pub discriminator: Option<String>,
    /// User's avatar hash.
    pub avatar: Option<String>,
    /// Whether the user is a bot.
    #[serde(default)]
    pub bot: bool,
}

/// Discord message object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    /// Message ID.
    pub id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Guild ID (if in a guild).
    pub guild_id: Option<String>,
    /// Message author.
    pub author: Option<DiscordUser>,
    /// Message content.
    pub content: Option<String>,
    /// Message timestamp (ISO 8601).
    pub timestamp: String,
    /// Edited timestamp.
    pub edited_timestamp: Option<String>,
    /// Message attachments.
    pub attachments: Option<Vec<DiscordAttachment>>,
    /// Message embeds.
    pub embeds: Option<Vec<DiscordEmbed>>,
    /// Mentioned users.
    pub mentions: Option<Vec<DiscordUser>>,
    /// Message reference (for replies).
    pub message_reference: Option<MessageReference>,
    /// Thread info (if message started a thread).
    pub thread: Option<DiscordThread>,
}

/// Discord attachment object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordAttachment {
    /// Attachment ID.
    pub id: String,
    /// Filename.
    pub filename: String,
    /// File size in bytes.
    pub size: i64,
    /// Source URL.
    pub url: String,
    /// Proxy URL.
    pub proxy_url: Option<String>,
    /// Content type (MIME).
    pub content_type: Option<String>,
    /// Image width.
    pub width: Option<i32>,
    /// Image height.
    pub height: Option<i32>,
}

/// Discord embed object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscordEmbed {
    /// Embed title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Embed description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Embed URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Embed color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<i32>,
    /// Embed image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    /// Embed video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<EmbedVideo>,
}

/// Embed image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedImage {
    /// Image URL.
    pub url: String,
}

/// Embed video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedVideo {
    /// Video URL.
    pub url: String,
}

/// Message reference for replies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReference {
    /// Referenced message ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Referenced channel ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Referenced guild ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
}

/// Discord thread object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordThread {
    /// Thread ID.
    pub id: String,
    /// Thread name.
    pub name: Option<String>,
    /// Parent channel ID.
    pub parent_id: Option<String>,
}

/// Create message parameters.
#[derive(Debug, Serialize)]
struct CreateMessageParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<DiscordEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<AllowedMentions>,
}

/// Allowed mentions configuration.
#[derive(Debug, Default, Serialize)]
struct AllowedMentions {
    parse: Vec<String>,
}

/// Gateway event wrapper (for inbound messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGatewayEvent {
    /// Event type.
    pub t: Option<String>,
    /// Event data.
    pub d: Option<DiscordMessage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let channel = DiscordChannel::new(ApiKey::new("test".to_string()));
        assert_eq!(channel.id(), "discord");
    }

    #[test]
    fn test_capabilities() {
        let channel = DiscordChannel::new(ApiKey::new("test".to_string()));
        let caps = channel.capabilities();
        assert!(caps.text);
        assert!(caps.images);
        assert!(caps.reactions);
        assert!(caps.threads);
        assert!(!caps.voice); // REST API doesn't support voice
    }

    #[test]
    fn test_text_limit() {
        let channel = DiscordChannel::new(ApiKey::new("test".to_string()));
        assert_eq!(channel.text_chunk_limit(), 2000);
    }
}

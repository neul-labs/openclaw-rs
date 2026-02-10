//! Telegram channel adapter using the Bot API.

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

const TELEGRAM_API_BASE: &str = "https://api.telegram.org";

/// Telegram channel adapter.
pub struct TelegramChannel {
    client: Client,
    token: ApiKey,
    state: Arc<RwLock<TelegramState>>,
}

#[derive(Debug, Default)]
struct TelegramState {
    account_id: Option<String>,
    username: Option<String>,
    connected: bool,
    last_update_id: Option<i64>,
}

impl TelegramChannel {
    /// Create a new Telegram channel.
    #[must_use]
    pub fn new(token: ApiKey) -> Self {
        Self {
            client: Client::new(),
            token,
            state: Arc::new(RwLock::new(TelegramState::default())),
        }
    }

    /// Get the Bot API URL.
    fn api_url(&self, method: &str) -> String {
        format!(
            "{}/bot{}/{}",
            TELEGRAM_API_BASE,
            self.token.expose(),
            method
        )
    }

    /// Call a Telegram Bot API method.
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Option<&impl Serialize>,
    ) -> Result<T, ChannelError> {
        let url = self.api_url(method);

        let response = match params {
            Some(p) => self.client.post(&url).json(p).send().await,
            None => self.client.get(&url).send().await,
        }
        .map_err(|e| ChannelError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 429 {
                return Err(ChannelError::RateLimited);
            }
            let text = response.text().await.unwrap_or_default();
            return Err(ChannelError::Network(format!("{}: {}", status, text)));
        }

        let result: TelegramResponse<T> = response
            .json()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if result.ok {
            result
                .result
                .ok_or_else(|| ChannelError::Network("Empty response".to_string()))
        } else {
            Err(ChannelError::Network(
                result
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    fn id(&self) -> &str {
        "telegram"
    }

    fn label(&self) -> &str {
        "Telegram"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            text: true,
            images: true,
            videos: true,
            voice: true,
            files: true,
            threads: true,    // Reply threads
            reactions: false, // Bot API doesn't support reactions well
            editing: true,
            deletion: true,
        }
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), ChannelError> {
        // Verify connection and get bot info
        let me: TelegramUser = self.call("getMe", None::<&()>).await?;

        let mut state = self.state.write().await;
        state.account_id = Some(me.id.to_string());
        state.username = me.username;
        state.connected = true;

        tracing::info!("Telegram bot connected: {}", me.first_name);
        Ok(())
    }

    async fn stop(&self) -> Result<(), ChannelError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }

    async fn probe(&self) -> Result<ChannelProbe, ChannelError> {
        match self.call::<TelegramUser>("getMe", None::<&()>).await {
            Ok(me) => Ok(ChannelProbe {
                connected: true,
                account_id: Some(me.id.to_string()),
                display_name: Some(me.first_name),
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
impl ChannelOutbound for TelegramChannel {
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError> {
        let params = SendMessageParams {
            chat_id: ctx.chat_id.clone(),
            text: text.to_string(),
            reply_to_message_id: ctx.reply_to.as_ref().and_then(|id| id.parse().ok()),
            message_thread_id: ctx.thread_id.as_ref().and_then(|id| id.parse().ok()),
            parse_mode: Some("HTML".to_string()),
        };

        let result: TelegramMessage = self.call("sendMessage", Some(&params)).await?;

        Ok(DeliveryResult {
            message_id: result.message_id.to_string(),
            channel: ChannelId::telegram(),
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
        // Send each attachment (simplified - could use sendMediaGroup for multiple)
        let mut last_id = String::new();

        for attachment in media {
            let method = match attachment.kind {
                AttachmentKind::Image => "sendPhoto",
                AttachmentKind::Video => "sendVideo",
                AttachmentKind::Audio => "sendAudio",
                AttachmentKind::Voice => "sendVoice",
                AttachmentKind::Document | _ => "sendDocument",
            };

            let params = serde_json::json!({
                "chat_id": ctx.chat_id,
                "document": attachment.url,
                "reply_to_message_id": ctx.reply_to,
                "message_thread_id": ctx.thread_id,
            });

            let result: TelegramMessage = self.call(method, Some(&params)).await?;
            last_id = result.message_id.to_string();
        }

        Ok(DeliveryResult {
            message_id: last_id,
            channel: ChannelId::telegram(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(ctx.chat_id),
            meta: None,
        })
    }

    fn text_chunk_limit(&self) -> usize {
        4096 // Telegram message limit
    }

    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::Immediate
    }
}

#[async_trait]
impl ChannelInbound for TelegramChannel {
    type RawMessage = TelegramUpdate;

    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError> {
        // Serialize before extracting fields (raw is consumed)
        let raw_value = serde_json::to_value(&raw).unwrap_or_default();

        let message = raw
            .message
            .or(raw.edited_message)
            .or(raw.channel_post)
            .ok_or_else(|| ChannelError::Config("No message in update".to_string()))?;

        let from = message
            .from
            .ok_or_else(|| ChannelError::Config("No sender in message".to_string()))?;

        let chat = &message.chat;
        let peer_type = match chat.chat_type.as_str() {
            "private" => PeerType::Dm,
            "group" | "supergroup" => PeerType::Group,
            "channel" => PeerType::Channel,
            _ => PeerType::Dm,
        };

        let state = futures::executor::block_on(self.state.read());
        let account_id = state.account_id.clone().unwrap_or_default();

        let mut attachments = Vec::new();

        // Handle photo
        if let Some(photos) = message.photo {
            if let Some(largest) = photos.last() {
                attachments.push(Attachment {
                    kind: AttachmentKind::Image,
                    url: largest.file_id.clone(),
                    mime_type: Some("image/jpeg".to_string()),
                    filename: None,
                    size: Some(largest.file_size.unwrap_or(0) as u64),
                    thumbnail_url: None,
                });
            }
        }

        // Handle document
        if let Some(doc) = message.document {
            attachments.push(Attachment {
                kind: AttachmentKind::Document,
                url: doc.file_id,
                mime_type: doc.mime_type,
                filename: doc.file_name,
                size: doc.file_size.map(|s| s as u64),
                thumbnail_url: None,
            });
        }

        // Handle voice
        if let Some(voice) = message.voice {
            attachments.push(Attachment {
                kind: AttachmentKind::Voice,
                url: voice.file_id,
                mime_type: Some(voice.mime_type.unwrap_or_else(|| "audio/ogg".to_string())),
                filename: None,
                size: Some(voice.file_size.unwrap_or(0) as u64),
                thumbnail_url: None,
            });
        }

        Ok(Message {
            id: message.message_id.to_string(),
            channel: ChannelId::telegram(),
            account_id,
            peer_id: PeerId::new(from.id.to_string()),
            peer_type,
            content: message.text.unwrap_or_default(),
            attachments,
            timestamp: chrono::DateTime::from_timestamp(message.date, 0)
                .unwrap_or_else(chrono::Utc::now),
            reply_to: message.reply_to_message.map(|m| m.message_id.to_string()),
            thread_id: message.message_thread_id.map(|id| id.to_string()),
            mentions: Vec::new(),
            raw: Some(raw_value),
        })
    }

    async fn acknowledge(&self, _message_id: &str) -> Result<(), ChannelError> {
        // Telegram doesn't require explicit acknowledgement
        Ok(())
    }
}

// Telegram API types

#[derive(Debug, Deserialize)]
struct TelegramResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    /// User ID.
    pub id: i64,
    /// Whether user is a bot.
    pub is_bot: bool,
    /// First name.
    pub first_name: String,
    /// Last name.
    pub last_name: Option<String>,
    /// Username.
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub edited_message: Option<TelegramMessage>,
    pub channel_post: Option<TelegramMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub date: i64,
    pub chat: TelegramChat,
    pub from: Option<TelegramUser>,
    pub text: Option<String>,
    pub caption: Option<String>,
    pub reply_to_message: Option<Box<TelegramMessage>>,
    pub message_thread_id: Option<i64>,
    pub photo: Option<Vec<TelegramPhotoSize>>,
    pub document: Option<TelegramDocument>,
    pub voice: Option<TelegramVoice>,
    pub video: Option<TelegramVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramPhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub file_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramDocument {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramVoice {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    pub mime_type: Option<String>,
    pub file_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramVideo {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    pub mime_type: Option<String>,
    pub file_size: Option<i32>,
}

#[derive(Debug, Serialize)]
struct SendMessageParams {
    chat_id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to_message_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_thread_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let channel = TelegramChannel::new(ApiKey::new("test".to_string()));
        assert_eq!(channel.id(), "telegram");
    }

    #[test]
    fn test_capabilities() {
        let channel = TelegramChannel::new(ApiKey::new("test".to_string()));
        let caps = channel.capabilities();
        assert!(caps.text);
        assert!(caps.images);
        assert!(caps.voice);
    }
}

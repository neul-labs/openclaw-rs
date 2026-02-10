//! WhatsApp channel adapter using the Cloud API.

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

const WHATSAPP_API_BASE: &str = "https://graph.facebook.com/v18.0";

/// WhatsApp channel adapter using the Cloud API (Business Platform).
pub struct WhatsAppChannel {
    client: Client,
    access_token: ApiKey,
    phone_number_id: String,
    state: Arc<RwLock<WhatsAppState>>,
}

#[derive(Debug, Default)]
struct WhatsAppState {
    display_phone_number: Option<String>,
    verified_name: Option<String>,
    connected: bool,
}

impl WhatsAppChannel {
    /// Create a new WhatsApp channel.
    ///
    /// # Arguments
    /// * `access_token` - Meta/Facebook access token
    /// * `phone_number_id` - WhatsApp Business phone number ID
    #[must_use]
    pub fn new(access_token: ApiKey, phone_number_id: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            access_token,
            phone_number_id: phone_number_id.into(),
            state: Arc::new(RwLock::new(WhatsAppState::default())),
        }
    }

    /// Call a WhatsApp Cloud API endpoint.
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ChannelError> {
        let url = format!("{}{}", WHATSAPP_API_BASE, endpoint);

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
}

#[async_trait]
impl Channel for WhatsAppChannel {
    fn id(&self) -> &str {
        "whatsapp"
    }

    fn label(&self) -> &str {
        "WhatsApp"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            text: true,
            images: true,
            videos: true,
            voice: true,
            files: true,
            threads: false, // WhatsApp doesn't have threads
            reactions: true,
            editing: false, // WhatsApp doesn't support editing
            deletion: true,
        }
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), ChannelError> {
        // Get phone number details
        let endpoint = format!("/{}", self.phone_number_id);
        let info: PhoneNumberInfo = self
            .call(reqwest::Method::GET, &endpoint, None::<&()>)
            .await?;

        let mut state = self.state.write().await;
        state.display_phone_number = Some(info.display_phone_number);
        state.verified_name = info.verified_name;
        state.connected = true;

        tracing::info!(
            "WhatsApp connected: {}",
            state.display_phone_number.as_deref().unwrap_or("unknown")
        );
        Ok(())
    }

    async fn stop(&self) -> Result<(), ChannelError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }

    async fn probe(&self) -> Result<ChannelProbe, ChannelError> {
        let endpoint = format!("/{}", self.phone_number_id);
        match self
            .call::<PhoneNumberInfo>(reqwest::Method::GET, &endpoint, None::<&()>)
            .await
        {
            Ok(info) => Ok(ChannelProbe {
                connected: true,
                account_id: Some(self.phone_number_id.clone()),
                display_name: Some(info.display_phone_number),
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
impl ChannelOutbound for WhatsAppChannel {
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError> {
        let endpoint = format!("/{}/messages", self.phone_number_id);

        let params = SendMessageRequest {
            messaging_product: "whatsapp".to_string(),
            recipient_type: "individual".to_string(),
            to: ctx.chat_id.clone(),
            message_type: "text".to_string(),
            text: Some(TextContent {
                preview_url: false,
                body: text.to_string(),
            }),
            image: None,
            video: None,
            audio: None,
            document: None,
            context: ctx.reply_to.map(|id| MessageContext { message_id: id }),
        };

        let result: SendMessageResponse = self
            .call(reqwest::Method::POST, &endpoint, Some(&params))
            .await?;

        let message_id = result
            .messages
            .first()
            .map(|m| m.id.clone())
            .unwrap_or_default();

        Ok(DeliveryResult {
            message_id,
            channel: ChannelId::whatsapp(),
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
        let endpoint = format!("/{}/messages", self.phone_number_id);
        let mut last_id = String::new();

        for attachment in media {
            let (message_type, image, video, audio, document) = match attachment.kind {
                AttachmentKind::Image => (
                    "image",
                    Some(MediaContent {
                        link: attachment.url.clone(),
                        caption: attachment.filename.clone(),
                    }),
                    None,
                    None,
                    None,
                ),
                AttachmentKind::Video => (
                    "video",
                    None,
                    Some(MediaContent {
                        link: attachment.url.clone(),
                        caption: attachment.filename.clone(),
                    }),
                    None,
                    None,
                ),
                AttachmentKind::Audio | AttachmentKind::Voice => (
                    "audio",
                    None,
                    None,
                    Some(AudioContent {
                        link: attachment.url.clone(),
                    }),
                    None,
                ),
                _ => (
                    "document",
                    None,
                    None,
                    None,
                    Some(DocumentContent {
                        link: attachment.url.clone(),
                        filename: attachment.filename.clone(),
                        caption: None,
                    }),
                ),
            };

            let params = SendMessageRequest {
                messaging_product: "whatsapp".to_string(),
                recipient_type: "individual".to_string(),
                to: ctx.chat_id.clone(),
                message_type: message_type.to_string(),
                text: None,
                image,
                video,
                audio,
                document,
                context: ctx
                    .reply_to
                    .clone()
                    .map(|id| MessageContext { message_id: id }),
            };

            let result: SendMessageResponse = self
                .call(reqwest::Method::POST, &endpoint, Some(&params))
                .await?;

            last_id = result
                .messages
                .first()
                .map(|m| m.id.clone())
                .unwrap_or_default();
        }

        Ok(DeliveryResult {
            message_id: last_id,
            channel: ChannelId::whatsapp(),
            timestamp: chrono::Utc::now(),
            chat_id: Some(ctx.chat_id),
            meta: None,
        })
    }

    fn text_chunk_limit(&self) -> usize {
        4096 // WhatsApp text message limit
    }

    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::Immediate
    }
}

#[async_trait]
impl ChannelInbound for WhatsAppChannel {
    type RawMessage = WhatsAppWebhookPayload;

    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError> {
        let raw_value = serde_json::to_value(&raw).unwrap_or_default();

        let entry = raw
            .entry
            .into_iter()
            .next()
            .ok_or_else(|| ChannelError::Config("No entry in webhook".to_string()))?;

        let change = entry
            .changes
            .into_iter()
            .next()
            .ok_or_else(|| ChannelError::Config("No changes in entry".to_string()))?;

        let value = change.value;

        let message = value
            .messages
            .and_then(|m| m.into_iter().next())
            .ok_or_else(|| ChannelError::Config("No message in value".to_string()))?;

        let _contact = value.contacts.and_then(|c| c.into_iter().next());

        // Determine peer type (WhatsApp is always DM for Cloud API)
        let peer_type = PeerType::Dm;

        // Extract text content
        let content = message
            .text
            .map(|t| t.body)
            .or(message.caption.clone())
            .unwrap_or_default();

        // Convert media to attachments
        let mut attachments = Vec::new();

        if let Some(img) = message.image {
            attachments.push(Attachment {
                kind: AttachmentKind::Image,
                url: img.id,
                mime_type: Some(img.mime_type),
                filename: None,
                size: None,
                thumbnail_url: None,
            });
        }

        if let Some(vid) = message.video {
            attachments.push(Attachment {
                kind: AttachmentKind::Video,
                url: vid.id,
                mime_type: Some(vid.mime_type),
                filename: None,
                size: None,
                thumbnail_url: None,
            });
        }

        if let Some(aud) = message.audio {
            attachments.push(Attachment {
                kind: if aud.voice.unwrap_or(false) {
                    AttachmentKind::Voice
                } else {
                    AttachmentKind::Audio
                },
                url: aud.id,
                mime_type: Some(aud.mime_type),
                filename: None,
                size: None,
                thumbnail_url: None,
            });
        }

        if let Some(doc) = message.document {
            attachments.push(Attachment {
                kind: AttachmentKind::Document,
                url: doc.id,
                mime_type: Some(doc.mime_type),
                filename: doc.filename,
                size: None,
                thumbnail_url: None,
            });
        }

        // Parse timestamp
        let timestamp = message
            .timestamp
            .parse::<i64>()
            .ok()
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .unwrap_or_else(chrono::Utc::now);

        Ok(Message {
            id: message.id,
            channel: ChannelId::whatsapp(),
            account_id: self.phone_number_id.clone(),
            peer_id: PeerId::new(message.from),
            peer_type,
            content,
            attachments,
            timestamp,
            reply_to: message.context.map(|c| c.id),
            thread_id: None,
            mentions: Vec::new(),
            raw: Some(raw_value),
        })
    }

    async fn acknowledge(&self, _message_id: &str) -> Result<(), ChannelError> {
        // WhatsApp Cloud API handles delivery via webhooks
        Ok(())
    }
}

// WhatsApp Cloud API types

/// Phone number info.
#[derive(Debug, Deserialize)]
struct PhoneNumberInfo {
    display_phone_number: String,
    verified_name: Option<String>,
}

/// Send message request.
#[derive(Debug, Serialize)]
struct SendMessageRequest {
    messaging_product: String,
    recipient_type: String,
    to: String,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<MediaContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<MediaContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<AudioContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<DocumentContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<MessageContext>,
}

/// Text content.
#[derive(Debug, Serialize)]
struct TextContent {
    preview_url: bool,
    body: String,
}

/// Media content (image/video).
#[derive(Debug, Serialize)]
struct MediaContent {
    link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
}

/// Audio content.
#[derive(Debug, Serialize)]
struct AudioContent {
    link: String,
}

/// Document content.
#[derive(Debug, Serialize)]
struct DocumentContent {
    link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
}

/// Message context (for replies).
#[derive(Debug, Serialize)]
struct MessageContext {
    message_id: String,
}

/// Send message response.
#[derive(Debug, Deserialize)]
struct SendMessageResponse {
    messages: Vec<MessageInfo>,
}

/// Message info in response.
#[derive(Debug, Deserialize)]
struct MessageInfo {
    id: String,
}

/// Webhook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppWebhookPayload {
    /// Object type (always "whatsapp_business_account").
    pub object: String,
    /// Entry array.
    pub entry: Vec<WebhookEntry>,
}

/// Webhook entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEntry {
    /// WhatsApp Business Account ID.
    pub id: String,
    /// Changes array.
    pub changes: Vec<WebhookChange>,
}

/// Webhook change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookChange {
    /// Change value.
    pub value: WebhookValue,
    /// Field that changed.
    pub field: String,
}

/// Webhook value containing messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookValue {
    /// Messaging product.
    pub messaging_product: Option<String>,
    /// Metadata.
    pub metadata: Option<WebhookMetadata>,
    /// Contacts.
    pub contacts: Option<Vec<WebhookContact>>,
    /// Messages.
    pub messages: Option<Vec<WebhookMessage>>,
}

/// Webhook metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMetadata {
    /// Display phone number.
    pub display_phone_number: Option<String>,
    /// Phone number ID.
    pub phone_number_id: Option<String>,
}

/// Webhook contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookContact {
    /// Contact profile.
    pub profile: Option<ContactProfile>,
    /// WhatsApp ID (phone number).
    pub wa_id: String,
}

/// Contact profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactProfile {
    /// Contact name.
    pub name: Option<String>,
}

/// Webhook message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessage {
    /// Message ID.
    pub id: String,
    /// Sender's WhatsApp ID.
    pub from: String,
    /// Timestamp (Unix epoch string).
    pub timestamp: String,
    /// Message type.
    #[serde(rename = "type")]
    pub message_type: String,
    /// Text content.
    pub text: Option<WebhookText>,
    /// Image content.
    pub image: Option<WebhookMedia>,
    /// Video content.
    pub video: Option<WebhookMedia>,
    /// Audio content.
    pub audio: Option<WebhookAudio>,
    /// Document content.
    pub document: Option<WebhookDocument>,
    /// Caption (for media).
    pub caption: Option<String>,
    /// Context (for replies).
    pub context: Option<WebhookContext>,
}

/// Webhook text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookText {
    /// Message body.
    pub body: String,
}

/// Webhook media (image/video).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMedia {
    /// Media ID.
    pub id: String,
    /// MIME type.
    pub mime_type: String,
    /// SHA256 hash.
    pub sha256: Option<String>,
}

/// Webhook audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAudio {
    /// Audio ID.
    pub id: String,
    /// MIME type.
    pub mime_type: String,
    /// Whether it's a voice message.
    pub voice: Option<bool>,
}

/// Webhook document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDocument {
    /// Document ID.
    pub id: String,
    /// MIME type.
    pub mime_type: String,
    /// Filename.
    pub filename: Option<String>,
}

/// Webhook context (reply info).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookContext {
    /// Original message ID.
    pub id: String,
    /// Sender of original message.
    pub from: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let channel = WhatsAppChannel::new(ApiKey::new("test".to_string()), "123456789");
        assert_eq!(channel.id(), "whatsapp");
    }

    #[test]
    fn test_capabilities() {
        let channel = WhatsAppChannel::new(ApiKey::new("test".to_string()), "123456789");
        let caps = channel.capabilities();
        assert!(caps.text);
        assert!(caps.images);
        assert!(caps.reactions);
        assert!(!caps.threads);
        assert!(!caps.editing);
    }

    #[test]
    fn test_text_limit() {
        let channel = WhatsAppChannel::new(ApiKey::new("test".to_string()), "123456789");
        assert_eq!(channel.text_chunk_limit(), 4096);
    }
}

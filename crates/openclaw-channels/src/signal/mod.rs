//! Signal channel adapter using Signal CLI REST API.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use openclaw_core::types::{
    Attachment, AttachmentKind, ChannelId, DeliveryResult, Message, PeerId, PeerType,
};

use crate::traits::{
    Channel, ChannelCapabilities, ChannelContext, ChannelError, ChannelInbound, ChannelOutbound,
    ChannelProbe, DeliveryMode, OutboundContext,
};

/// Signal channel adapter.
///
/// Uses signal-cli REST API (https://github.com/bbernhard/signal-cli-rest-api).
pub struct SignalChannel {
    client: Client,
    api_url: String,
    phone_number: String,
    state: Arc<RwLock<SignalState>>,
}

#[derive(Debug, Default)]
struct SignalState {
    registered: bool,
    connected: bool,
}

impl SignalChannel {
    /// Create a new Signal channel.
    ///
    /// # Arguments
    /// * `api_url` - URL of the signal-cli REST API (e.g., "http://localhost:8080")
    /// * `phone_number` - Registered phone number (e.g., "+1234567890")
    #[must_use]
    pub fn new(api_url: impl Into<String>, phone_number: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_url: api_url.into(),
            phone_number: phone_number.into(),
            state: Arc::new(RwLock::new(SignalState::default())),
        }
    }

    /// Call a Signal CLI REST API endpoint.
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ChannelError> {
        let url = format!("{}{}", self.api_url, endpoint);

        let mut request = self
            .client
            .request(method, &url)
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
            let text = response.text().await.unwrap_or_default();
            return Err(ChannelError::Network(format!("{}: {}", status, text)));
        }

        response
            .json()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))
    }

    /// Call API without expecting response body.
    async fn call_no_response(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
    ) -> Result<(), ChannelError> {
        let url = format!("{}{}", self.api_url, endpoint);

        let mut request = self
            .client
            .request(method, &url)
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
            let text = response.text().await.unwrap_or_default();
            return Err(ChannelError::Network(format!("{}: {}", status, text)));
        }

        Ok(())
    }
}

#[async_trait]
impl Channel for SignalChannel {
    fn id(&self) -> &str {
        "signal"
    }

    fn label(&self) -> &str {
        "Signal"
    }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            text: true,
            images: true,
            videos: true,
            voice: true,
            files: true,
            threads: false, // Signal doesn't have threads
            reactions: true, // Signal has reactions
            editing: false,  // Signal doesn't support editing
            deletion: true,  // Signal supports delete for everyone
        }
    }

    async fn start(&self, _ctx: ChannelContext) -> Result<(), ChannelError> {
        // Check if number is registered
        let endpoint = format!("/v1/about/{}", self.phone_number);
        let _: SignalAbout = self
            .call(reqwest::Method::GET, &endpoint, None::<&()>)
            .await?;

        let mut state = self.state.write().await;
        state.registered = true;
        state.connected = true;

        tracing::info!("Signal connected: {}", self.phone_number);
        Ok(())
    }

    async fn stop(&self) -> Result<(), ChannelError> {
        let mut state = self.state.write().await;
        state.connected = false;
        Ok(())
    }

    async fn probe(&self) -> Result<ChannelProbe, ChannelError> {
        let endpoint = format!("/v1/about/{}", self.phone_number);
        match self
            .call::<SignalAbout>(reqwest::Method::GET, &endpoint, None::<&()>)
            .await
        {
            Ok(_) => Ok(ChannelProbe {
                connected: true,
                account_id: Some(self.phone_number.clone()),
                display_name: Some(self.phone_number.clone()),
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
impl ChannelOutbound for SignalChannel {
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError> {
        let endpoint = format!("/v2/send");

        let params = SendMessageParams {
            number: self.phone_number.clone(),
            recipients: vec![ctx.chat_id.clone()],
            message: text.to_string(),
            base64_attachments: None,
        };

        let results: Vec<SendResult> = self
            .call(reqwest::Method::POST, &endpoint, Some(&params))
            .await?;

        let timestamp = results
            .first()
            .map(|r| r.timestamp.to_string())
            .unwrap_or_default();

        Ok(DeliveryResult {
            message_id: timestamp.clone(),
            channel: ChannelId::signal(),
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
        // Signal CLI REST API expects base64 attachments
        // For URLs, we'd need to download and encode them
        // Simplified: send as text with URLs
        let urls: Vec<String> = media.iter().map(|a| a.url.clone()).collect();
        let text = urls.join("\n");

        self.send_text(ctx, &text).await
    }

    fn text_chunk_limit(&self) -> usize {
        // Signal doesn't have a strict limit, but let's be reasonable
        65536
    }

    fn delivery_mode(&self) -> DeliveryMode {
        DeliveryMode::Immediate
    }
}

#[async_trait]
impl ChannelInbound for SignalChannel {
    type RawMessage = SignalMessage;

    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError> {
        let raw_value = serde_json::to_value(&raw).unwrap_or_default();

        let envelope = raw
            .envelope
            .ok_or_else(|| ChannelError::Config("No envelope in message".to_string()))?;

        let source = envelope
            .source
            .ok_or_else(|| ChannelError::Config("No source in envelope".to_string()))?;

        let data_message = envelope
            .data_message
            .ok_or_else(|| ChannelError::Config("No data message".to_string()))?;

        // Determine peer type (group or DM)
        let (peer_type, peer_id_str) = if let Some(group_info) = &data_message.group_info {
            (PeerType::Group, group_info.group_id.clone())
        } else {
            (PeerType::Dm, source.clone())
        };

        // Convert attachments
        let attachments = data_message
            .attachments
            .unwrap_or_default()
            .into_iter()
            .map(|a| {
                let kind = if a.content_type.starts_with("image/") {
                    AttachmentKind::Image
                } else if a.content_type.starts_with("video/") {
                    AttachmentKind::Video
                } else if a.content_type.starts_with("audio/") {
                    AttachmentKind::Audio
                } else {
                    AttachmentKind::Document
                };

                Attachment {
                    kind,
                    url: a.id.clone(),
                    mime_type: Some(a.content_type),
                    filename: a.filename,
                    size: Some(a.size as u64),
                    thumbnail_url: None,
                }
            })
            .collect();

        let timestamp =
            chrono::DateTime::from_timestamp_millis(envelope.timestamp.unwrap_or(0) as i64)
                .unwrap_or_else(chrono::Utc::now);

        Ok(Message {
            id: envelope.timestamp.unwrap_or(0).to_string(),
            channel: ChannelId::signal(),
            account_id: self.phone_number.clone(),
            peer_id: PeerId::new(peer_id_str),
            peer_type,
            content: data_message.message.unwrap_or_default(),
            attachments,
            timestamp,
            reply_to: data_message.quote.map(|q| q.id.to_string()),
            thread_id: None,
            mentions: data_message
                .mentions
                .map(|m| m.into_iter().map(|mention| mention.uuid).collect())
                .unwrap_or_default(),
            raw: Some(raw_value),
        })
    }

    async fn acknowledge(&self, _message_id: &str) -> Result<(), ChannelError> {
        // Signal handles read receipts separately
        Ok(())
    }
}

// Signal CLI REST API types

/// About response.
#[derive(Debug, Deserialize)]
struct SignalAbout {
    versions: Option<serde_json::Value>,
}

/// Send message parameters.
#[derive(Debug, Serialize)]
struct SendMessageParams {
    number: String,
    recipients: Vec<String>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    base64_attachments: Option<Vec<String>>,
}

/// Send result.
#[derive(Debug, Deserialize)]
struct SendResult {
    timestamp: i64,
}

/// Incoming Signal message (from receive endpoint or webhook).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    /// Account that received the message.
    pub account: Option<String>,
    /// Message envelope.
    pub envelope: Option<SignalEnvelope>,
}

/// Signal envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEnvelope {
    /// Source phone number.
    pub source: Option<String>,
    /// Source UUID.
    #[serde(rename = "sourceUuid")]
    pub source_uuid: Option<String>,
    /// Source device.
    #[serde(rename = "sourceDevice")]
    pub source_device: Option<i32>,
    /// Timestamp.
    pub timestamp: Option<i64>,
    /// Data message content.
    #[serde(rename = "dataMessage")]
    pub data_message: Option<SignalDataMessage>,
}

/// Signal data message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDataMessage {
    /// Message text.
    pub message: Option<String>,
    /// Timestamp.
    pub timestamp: Option<i64>,
    /// Attachments.
    pub attachments: Option<Vec<SignalAttachment>>,
    /// Group info (if group message).
    #[serde(rename = "groupInfo")]
    pub group_info: Option<SignalGroupInfo>,
    /// Quote (reply).
    pub quote: Option<SignalQuote>,
    /// Mentions.
    pub mentions: Option<Vec<SignalMention>>,
}

/// Signal attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAttachment {
    /// Attachment ID.
    pub id: String,
    /// Content type.
    #[serde(rename = "contentType")]
    pub content_type: String,
    /// Filename.
    pub filename: Option<String>,
    /// Size in bytes.
    pub size: i64,
}

/// Signal group info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalGroupInfo {
    /// Group ID.
    #[serde(rename = "groupId")]
    pub group_id: String,
    /// Group type.
    #[serde(rename = "type")]
    pub group_type: Option<String>,
}

/// Signal quote (reply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalQuote {
    /// Quoted message ID.
    pub id: i64,
    /// Author of quoted message.
    pub author: Option<String>,
}

/// Signal mention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMention {
    /// Mentioned user UUID.
    pub uuid: String,
    /// Start position in text.
    pub start: Option<i32>,
    /// Length of mention.
    pub length: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let channel = SignalChannel::new("http://localhost:8080", "+1234567890");
        assert_eq!(channel.id(), "signal");
    }

    #[test]
    fn test_capabilities() {
        let channel = SignalChannel::new("http://localhost:8080", "+1234567890");
        let caps = channel.capabilities();
        assert!(caps.text);
        assert!(caps.images);
        assert!(caps.reactions);
        assert!(!caps.threads);
        assert!(!caps.editing);
    }
}

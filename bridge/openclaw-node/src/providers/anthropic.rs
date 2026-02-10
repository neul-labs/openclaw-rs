//! Anthropic Claude provider bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use openclaw_core::secrets::ApiKey;
use openclaw_providers::traits::ChunkType;
use openclaw_providers::{AnthropicProvider as RustAnthropicProvider, Provider};

use super::types::{
    JsCompletionRequest, JsCompletionResponse, JsStreamChunk, convert_request, convert_response,
};
use crate::error::OpenClawError;

/// Anthropic Claude API provider.
///
/// Supports Claude 3.5 Sonnet, Claude 3.5 Haiku, and other Claude models.
#[napi]
pub struct AnthropicProvider {
    inner: Arc<RustAnthropicProvider>,
}

#[napi]
impl AnthropicProvider {
    /// Create a new Anthropic provider with API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Anthropic API key (starts with "sk-ant-")
    #[napi(constructor)]
    #[must_use]
    pub fn new(api_key: String) -> Self {
        let key = ApiKey::new(api_key);
        Self {
            inner: Arc::new(RustAnthropicProvider::new(key)),
        }
    }

    /// Create a provider with custom base URL.
    ///
    /// Useful for proxies or custom endpoints.
    #[napi(factory)]
    #[must_use]
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        let key = ApiKey::new(api_key);
        Self {
            inner: Arc::new(RustAnthropicProvider::with_base_url(key, base_url)),
        }
    }

    /// Provider name ("anthropic").
    #[napi(getter)]
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// List available models.
    ///
    /// Returns an array of model IDs like "claude-3-5-sonnet-20241022".
    #[napi]
    pub async fn list_models(&self) -> Result<Vec<String>> {
        self.inner
            .list_models()
            .await
            .map_err(|e| OpenClawError::from_provider_error(e).into())
    }

    /// Create a completion (non-streaming).
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request with model, messages, etc.
    ///
    /// # Returns
    ///
    /// The completion response with content, tool calls, and usage.
    #[napi]
    pub async fn complete(&self, request: JsCompletionRequest) -> Result<JsCompletionResponse> {
        let rust_request = convert_request(request);
        let response = self
            .inner
            .complete(rust_request)
            .await
            .map_err(OpenClawError::from_provider_error)?;
        Ok(convert_response(response))
    }

    /// Create a streaming completion.
    ///
    /// The callback is called for each chunk received. Chunks have:
    /// - `chunk_type`: "`content_block_delta`", "`message_stop`", etc.
    /// - `delta`: Text content (for delta chunks)
    /// - `stop_reason`: Why generation stopped (for final chunk)
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request
    /// * `callback` - Function called with (error, chunk) for each chunk
    #[napi]
    pub fn complete_stream(
        &self,
        request: JsCompletionRequest,
        #[napi(ts_arg_type = "(err: Error | null, chunk: JsStreamChunk | null) => void")]
        callback: JsFunction,
    ) -> Result<()> {
        use futures::StreamExt;
        use napi::threadsafe_function::{
            ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode,
        };

        // Create threadsafe callback
        let tsfn: ThreadsafeFunction<JsStreamChunk, ErrorStrategy::CalleeHandled> =
            callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

        let inner = self.inner.clone();
        let rust_request = convert_request(request);

        // Spawn async streaming task
        napi::tokio::spawn(async move {
            match inner.complete_stream(rust_request).await {
                Ok(mut stream) => {
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                let js_chunk = convert_stream_chunk(
                                    &chunk.chunk_type,
                                    chunk.delta.as_deref(),
                                    chunk.index,
                                );
                                let _ = tsfn
                                    .call(Ok(js_chunk), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                            Err(e) => {
                                let err = OpenClawError::from_provider_error(e);
                                let _ = tsfn.call(
                                    Err(napi::Error::from_reason(
                                        serde_json::to_string(&err).unwrap_or_default(),
                                    )),
                                    ThreadsafeFunctionCallMode::NonBlocking,
                                );
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let err = OpenClawError::from_provider_error(e);
                    let _ = tsfn.call(
                        Err(napi::Error::from_reason(
                            serde_json::to_string(&err).unwrap_or_default(),
                        )),
                        ThreadsafeFunctionCallMode::NonBlocking,
                    );
                }
            }
        });

        Ok(())
    }
}

/// Convert `ChunkType` to `JsStreamChunk`.
fn convert_stream_chunk(
    chunk_type: &ChunkType,
    delta: Option<&str>,
    index: Option<usize>,
) -> JsStreamChunk {
    let (type_str, stop_reason) = match chunk_type {
        ChunkType::MessageStart => ("message_start", None),
        ChunkType::ContentBlockStart => ("content_block_start", None),
        ChunkType::ContentBlockDelta => ("content_block_delta", None),
        ChunkType::ContentBlockStop => ("content_block_stop", None),
        ChunkType::MessageDelta => ("message_delta", None),
        ChunkType::MessageStop => ("message_stop", None),
    };

    JsStreamChunk {
        chunk_type: type_str.to_string(),
        delta: delta.map(std::string::ToString::to_string),
        index: index.map(|i| i as u32),
        stop_reason,
    }
}

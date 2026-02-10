//! # OpenClaw Node.js Bridge
//!
//! napi-rs bindings to expose Rust core functionality to Node.js.
//!
//! ## Features
//!
//! - **Configuration**: Load and validate OpenClaw config files
//! - **Event Store**: Append-only event storage with CRDT projections
//! - **Providers**: Anthropic Claude and OpenAI GPT API clients
//! - **Auth**: Encrypted credential storage with safe API key handling
//! - **Tools**: Tool registry for agent tool execution
//! - **Validation**: Input validation and session key building
//!
//! ## Example
//!
//! ```javascript
//! const {
//!   loadDefaultConfig,
//!   AnthropicProvider,
//!   NodeApiKey,
//!   CredentialStore,
//!   ToolRegistry,
//!   NodeEventStore,
//! } = require('openclaw-node');
//!
//! // Load configuration
//! const config = JSON.parse(loadDefaultConfig());
//!
//! // Create provider
//! const provider = new AnthropicProvider(process.env.ANTHROPIC_API_KEY);
//!
//! // Create completion
//! const response = await provider.complete({
//!   model: 'claude-3-5-sonnet-20241022',
//!   messages: [{ role: 'user', content: 'Hello!' }],
//!   maxTokens: 1024,
//! });
//!
//! console.log(response.content);
//! ```

#![warn(missing_docs)]

// Error handling
pub mod error;

// Configuration
mod config;
pub use config::{load_config, load_default_config, validate_config};

// Event storage
mod events;
pub use events::NodeEventStore;

// Validation
mod validation;
pub use validation::{build_session_key, validate_message, validate_path};

// AI Providers
pub mod providers;
pub use providers::{
    AnthropicProvider, JsCompletionRequest, JsCompletionResponse, JsMessage, JsStreamChunk,
    JsTokenUsage, JsTool, JsToolCall, OpenAIProvider,
};

// Authentication
pub mod auth;
pub use auth::{CredentialStore, NodeApiKey};

// Agents
pub mod agents;
pub use agents::{JsToolDefinition, JsToolResult, ToolRegistry};

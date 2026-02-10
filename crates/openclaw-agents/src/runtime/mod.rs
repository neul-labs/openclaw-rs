//! Agent runtime.

use std::collections::HashMap;
use std::sync::Arc;

use openclaw_core::events::SessionProjection;
use openclaw_core::types::{AgentId, SessionKey};
use openclaw_providers::traits::Provider;

use crate::tools::ToolRegistry;

/// Agent execution context.
pub struct AgentContext {
    /// Agent ID.
    pub agent_id: AgentId,
    /// Session key.
    pub session_key: SessionKey,
    /// Session state.
    pub session: SessionProjection,
    /// Available tools.
    pub tools: Arc<ToolRegistry>,
    /// Custom context values.
    pub values: HashMap<String, serde_json::Value>,
}

impl AgentContext {
    /// Create a new agent context.
    #[must_use]
    pub fn new(
        agent_id: AgentId,
        session_key: SessionKey,
        session: SessionProjection,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            agent_id,
            session_key,
            session,
            tools,
            values: HashMap::new(),
        }
    }

    /// Set a context value.
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.values.insert(key.into(), value);
    }

    /// Get a context value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.values.get(key)
    }
}

/// Agent runtime for executing agent logic.
pub struct AgentRuntime {
    provider: Arc<dyn Provider>,
    tools: Arc<ToolRegistry>,
    model: String,
    system_prompt: Option<String>,
    max_tokens: u32,
    temperature: f32,
}

impl AgentRuntime {
    /// Create a new agent runtime.
    #[must_use]
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            tools: Arc::new(ToolRegistry::new()),
            model: "claude-3-5-sonnet-20241022".to_string(),
            system_prompt: None,
            max_tokens: 4096,
            temperature: 0.7,
        }
    }

    /// Set the model to use.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the system prompt.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set max tokens.
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set temperature.
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set the tool registry.
    #[must_use]
    pub fn with_tools(mut self, tools: Arc<ToolRegistry>) -> Self {
        self.tools = tools;
        self
    }

    /// Get the tool registry.
    #[must_use]
    pub fn tools(&self) -> &Arc<ToolRegistry> {
        &self.tools
    }

    /// Get the model name.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the system prompt.
    #[must_use]
    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    /// Get max tokens.
    #[must_use]
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    /// Get temperature.
    #[must_use]
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Process a user message and return a response.
    ///
    /// # Errors
    ///
    /// Returns error if provider call fails.
    pub async fn process_message(
        &self,
        ctx: &mut AgentContext,
        message: &str,
    ) -> Result<String, AgentRuntimeError> {
        use openclaw_providers::traits::{CompletionRequest, Message, MessageContent, Role};

        // Build messages from session history
        let mut messages: Vec<Message> = ctx
            .session
            .messages
            .iter()
            .map(|m| match m {
                openclaw_core::events::SessionMessage::Inbound(text) => Message {
                    role: Role::User,
                    content: MessageContent::Text(text.clone()),
                },
                openclaw_core::events::SessionMessage::Outbound(text) => Message {
                    role: Role::Assistant,
                    content: MessageContent::Text(text.clone()),
                },
                openclaw_core::events::SessionMessage::Tool { name, result } => Message {
                    role: Role::Tool,
                    content: MessageContent::Text(format!("[{name}]: {result}")),
                },
            })
            .collect();

        // Add current message
        messages.push(Message {
            role: Role::User,
            content: MessageContent::Text(message.to_string()),
        });

        // Build request
        let request = CompletionRequest {
            model: self.model.clone(),
            messages,
            system: self.system_prompt.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stop: None,
            tools: Some(self.tools.as_tool_definitions()),
        };

        // Call provider
        let response = self.provider.complete(request).await?;

        // Extract text response
        let text = response
            .content
            .iter()
            .filter_map(|block| {
                if let openclaw_providers::traits::ContentBlock::Text { text } = block {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}

/// Agent runtime errors.
#[derive(Debug, thiserror::Error)]
pub enum AgentRuntimeError {
    /// Provider error.
    #[error("Provider error: {0}")]
    Provider(#[from] openclaw_providers::traits::ProviderError),

    /// Tool execution error.
    #[error("Tool error: {0}")]
    Tool(String),

    /// Configuration error.
    #[error("Config error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context() {
        let ctx = AgentContext::new(
            AgentId::default_agent(),
            SessionKey::new("test"),
            openclaw_core::events::SessionProjection::new(
                SessionKey::new("test"),
                "default".to_string(),
                openclaw_core::types::ChannelId::telegram(),
                "user".to_string(),
            ),
            Arc::new(ToolRegistry::new()),
        );

        assert_eq!(ctx.agent_id.as_ref(), "default");
    }
}

//! Agent routing.

use openclaw_core::types::{AgentId, ChannelId, PeerId};

/// Route messages to appropriate agents.
pub struct AgentRouter {
    routes: Vec<RouteRule>,
    default_agent: AgentId,
}

/// Routing rule.
#[derive(Debug, Clone)]
pub struct RouteRule {
    /// Channel pattern.
    pub channel: Option<String>,
    /// Peer ID pattern.
    pub peer_pattern: Option<String>,
    /// Target agent.
    pub agent_id: AgentId,
    /// Priority (higher = first).
    pub priority: i32,
}

impl AgentRouter {
    /// Create a new router with default agent.
    #[must_use]
    pub fn new(default_agent: AgentId) -> Self {
        Self {
            routes: Vec::new(),
            default_agent,
        }
    }

    /// Add a routing rule.
    pub fn add_rule(&mut self, rule: RouteRule) {
        self.routes.push(rule);
        self.routes.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Route a message to an agent.
    #[must_use]
    pub fn route(&self, channel: &ChannelId, peer_id: &PeerId) -> &AgentId {
        for rule in &self.routes {
            if let Some(ch) = &rule.channel {
                if ch != "*" && ch != channel.as_ref() {
                    continue;
                }
            }

            if let Some(pattern) = &rule.peer_pattern {
                if pattern != "*" && pattern != peer_id.as_ref() {
                    continue;
                }
            }

            return &rule.agent_id;
        }

        &self.default_agent
    }
}

impl Default for AgentRouter {
    fn default() -> Self {
        Self::new(AgentId::default_agent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_routing() {
        let router = AgentRouter::default();
        let agent = router.route(&ChannelId::telegram(), &PeerId::new("123"));
        assert_eq!(agent.as_ref(), "default");
    }

    #[test]
    fn test_specific_routing() {
        let mut router = AgentRouter::default();
        router.add_rule(RouteRule {
            channel: Some("telegram".to_string()),
            peer_pattern: Some("vip123".to_string()),
            agent_id: AgentId::new("vip-agent"),
            priority: 100,
        });

        let agent = router.route(&ChannelId::telegram(), &PeerId::new("vip123"));
        assert_eq!(agent.as_ref(), "vip-agent");

        let agent = router.route(&ChannelId::telegram(), &PeerId::new("other"));
        assert_eq!(agent.as_ref(), "default");
    }
}

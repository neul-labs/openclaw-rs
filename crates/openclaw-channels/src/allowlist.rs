//! Allowlist for access control.

use serde::{Deserialize, Serialize};

use openclaw_core::types::{ChannelId, PeerId};

/// Allowlist entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistEntry {
    /// Channel pattern ("*" for any).
    pub channel: String,
    /// Peer ID pattern ("*" for any).
    pub peer_id: String,
    /// Optional label.
    pub label: Option<String>,
}

impl AllowlistEntry {
    /// Create a new allowlist entry.
    #[must_use]
    pub fn new(channel: impl Into<String>, peer_id: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            peer_id: peer_id.into(),
            label: None,
        }
    }

    /// Create an entry that allows any peer on a channel.
    #[must_use]
    pub fn channel_wide(channel: impl Into<String>) -> Self {
        Self::new(channel, "*")
    }

    /// Create an entry that allows a specific peer on any channel.
    #[must_use]
    pub fn peer_anywhere(peer_id: impl Into<String>) -> Self {
        Self::new("*", peer_id)
    }

    /// Check if this entry matches.
    #[must_use]
    pub fn matches(&self, channel: &ChannelId, peer_id: &PeerId) -> bool {
        let channel_matches = self.channel == "*" || self.channel == channel.as_ref();
        let peer_matches = self.peer_id == "*" || self.peer_id == peer_id.as_ref();
        channel_matches && peer_matches
    }
}

/// Allowlist for controlling access.
#[derive(Debug, Clone, Default)]
pub struct Allowlist {
    entries: Vec<AllowlistEntry>,
    default_allow: bool,
}

impl Allowlist {
    /// Create a new allowlist (default deny).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            default_allow: false,
        }
    }

    /// Create an open allowlist (default allow).
    #[must_use]
    pub fn open() -> Self {
        Self {
            entries: Vec::new(),
            default_allow: true,
        }
    }

    /// Add an entry.
    pub fn add(&mut self, entry: AllowlistEntry) {
        self.entries.push(entry);
    }

    /// Check if access is allowed.
    #[must_use]
    pub fn is_allowed(&self, channel: &ChannelId, peer_id: &PeerId) -> bool {
        if self.entries.is_empty() {
            return self.default_allow;
        }

        self.entries.iter().any(|e| e.matches(channel, peer_id))
    }

    /// Get all entries.
    #[must_use]
    pub fn entries(&self) -> &[AllowlistEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist_empty_deny() {
        let allowlist = Allowlist::new();
        assert!(!allowlist.is_allowed(&ChannelId::telegram(), &PeerId::new("123")));
    }

    #[test]
    fn test_allowlist_empty_allow() {
        let allowlist = Allowlist::open();
        assert!(allowlist.is_allowed(&ChannelId::telegram(), &PeerId::new("123")));
    }

    #[test]
    fn test_allowlist_specific() {
        let mut allowlist = Allowlist::new();
        allowlist.add(AllowlistEntry::new("telegram", "123"));

        assert!(allowlist.is_allowed(&ChannelId::telegram(), &PeerId::new("123")));
        assert!(!allowlist.is_allowed(&ChannelId::telegram(), &PeerId::new("456")));
        assert!(!allowlist.is_allowed(&ChannelId::discord(), &PeerId::new("123")));
    }

    #[test]
    fn test_allowlist_wildcard() {
        let mut allowlist = Allowlist::new();
        allowlist.add(AllowlistEntry::channel_wide("telegram"));

        assert!(allowlist.is_allowed(&ChannelId::telegram(), &PeerId::new("123")));
        assert!(allowlist.is_allowed(&ChannelId::telegram(), &PeerId::new("456")));
        assert!(!allowlist.is_allowed(&ChannelId::discord(), &PeerId::new("123")));
    }
}

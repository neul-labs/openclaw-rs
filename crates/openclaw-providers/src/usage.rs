//! Usage tracking for providers.

use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use openclaw_core::types::TokenUsage;

/// Usage tracker for monitoring token consumption.
pub struct UsageTracker {
    totals: RwLock<HashMap<String, ModelUsage>>,
}

/// Usage statistics for a model.
#[derive(Debug, Default)]
pub struct ModelUsage {
    input_tokens: AtomicU64,
    output_tokens: AtomicU64,
    request_count: AtomicU64,
}

impl UsageTracker {
    /// Create a new usage tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            totals: RwLock::new(HashMap::new()),
        }
    }

    /// Record token usage for a model.
    pub fn record(&self, model: &str, usage: &TokenUsage) {
        let mut totals = self.totals.write().unwrap();
        let entry = totals.entry(model.to_string()).or_default();

        entry
            .input_tokens
            .fetch_add(usage.input_tokens, Ordering::Relaxed);
        entry
            .output_tokens
            .fetch_add(usage.output_tokens, Ordering::Relaxed);
        entry.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total usage for a model.
    #[must_use]
    pub fn get_usage(&self, model: &str) -> Option<TokenUsageSummary> {
        let totals = self.totals.read().unwrap();
        totals.get(model).map(|u| TokenUsageSummary {
            input_tokens: u.input_tokens.load(Ordering::Relaxed),
            output_tokens: u.output_tokens.load(Ordering::Relaxed),
            request_count: u.request_count.load(Ordering::Relaxed),
        })
    }

    /// Get total usage across all models.
    #[must_use]
    pub fn total_usage(&self) -> TokenUsageSummary {
        let totals = self.totals.read().unwrap();
        let mut summary = TokenUsageSummary::default();

        for usage in totals.values() {
            summary.input_tokens += usage.input_tokens.load(Ordering::Relaxed);
            summary.output_tokens += usage.output_tokens.load(Ordering::Relaxed);
            summary.request_count += usage.request_count.load(Ordering::Relaxed);
        }

        summary
    }

    /// Reset all usage statistics.
    pub fn reset(&self) {
        let mut totals = self.totals.write().unwrap();
        totals.clear();
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of token usage.
#[derive(Debug, Clone, Default)]
pub struct TokenUsageSummary {
    /// Total input tokens.
    pub input_tokens: u64,
    /// Total output tokens.
    pub output_tokens: u64,
    /// Total request count.
    pub request_count: u64,
}

impl TokenUsageSummary {
    /// Get total tokens (input + output).
    #[must_use]
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_tracking() {
        let tracker = UsageTracker::new();

        tracker.record(
            "claude-3-5-sonnet",
            &TokenUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: None,
                cache_write_tokens: None,
            },
        );

        tracker.record(
            "claude-3-5-sonnet",
            &TokenUsage {
                input_tokens: 200,
                output_tokens: 100,
                cache_read_tokens: None,
                cache_write_tokens: None,
            },
        );

        let usage = tracker.get_usage("claude-3-5-sonnet").unwrap();
        assert_eq!(usage.input_tokens, 300);
        assert_eq!(usage.output_tokens, 150);
        assert_eq!(usage.request_count, 2);
    }

    #[test]
    fn test_total_usage() {
        let tracker = UsageTracker::new();

        tracker.record(
            "model1",
            &TokenUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: None,
                cache_write_tokens: None,
            },
        );

        tracker.record(
            "model2",
            &TokenUsage {
                input_tokens: 200,
                output_tokens: 100,
                cache_read_tokens: None,
                cache_write_tokens: None,
            },
        );

        let total = tracker.total_usage();
        assert_eq!(total.input_tokens, 300);
        assert_eq!(total.output_tokens, 150);
        assert_eq!(total.total_tokens(), 450);
    }
}

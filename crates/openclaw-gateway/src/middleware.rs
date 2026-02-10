//! Gateway middleware.

use governor::{Quota, RateLimiter, clock::DefaultClock, state::keyed::DefaultKeyedStateStore};
use std::num::NonZeroU32;

/// Rate limiter for the gateway.
pub struct GatewayRateLimiter {
    /// Per-client rate limiter.
    client_limiter: RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>,
}

impl GatewayRateLimiter {
    /// Create a new rate limiter.
    #[must_use]
    pub fn new(requests_per_minute: u32) -> Self {
        let quota =
            Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::MIN));
        Self {
            client_limiter: RateLimiter::keyed(quota),
        }
    }

    /// Check if a request is allowed.
    #[must_use]
    pub fn check(&self, client_id: &str) -> bool {
        self.client_limiter
            .check_key(&client_id.to_string())
            .is_ok()
    }
}

impl Default for GatewayRateLimiter {
    fn default() -> Self {
        Self::new(100) // 100 requests per minute
    }
}

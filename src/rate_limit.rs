// ABOUTME: Token bucket rate limiter for provider API requests
// ABOUTME: Per-provider rate limiting with configurable capacity and refill rate
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::time::{Duration, Instant};

/// Token bucket rate limiter for controlling API request rates.
///
/// Each provider has its own bucket with configurable capacity and refill rate.
#[derive(Debug)]
pub struct TokenBucket {
    /// Maximum number of tokens the bucket can hold.
    capacity: u32,
    /// Current number of available tokens.
    available: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// When tokens were last consumed or refilled.
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate.
    ///
    /// The bucket starts full.
    #[must_use]
    pub fn new(capacity: u32, refill_rate_per_second: f64) -> Self {
        Self {
            capacity,
            available: f64::from(capacity),
            refill_rate: refill_rate_per_second,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume one token. Returns `true` if the request is allowed.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.available >= 1.0 {
            self.available -= 1.0;
            true
        } else {
            false
        }
    }

    /// Return the estimated wait time until a token becomes available.
    #[must_use]
    pub fn wait_duration(&self) -> Duration {
        if self.available >= 1.0 {
            return Duration::ZERO;
        }
        let deficit = 1.0 - self.available;
        let seconds = deficit / self.refill_rate;
        Duration::from_secs_f64(seconds)
    }

    /// Return the number of currently available tokens.
    #[must_use]
    pub fn available_tokens(&self) -> u32 {
        self.available as u32
    }

    /// Return the bucket capacity.
    #[must_use]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Refill tokens based on elapsed time since the last refill.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.available = elapsed
            .mul_add(self.refill_rate, self.available)
            .min(f64::from(self.capacity));
        self.last_refill = now;
    }
}

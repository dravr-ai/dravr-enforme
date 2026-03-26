// ABOUTME: Tests for token bucket rate limiter behavior
// ABOUTME: Verifies acquisition, refill, wait duration, and capacity
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::time::Duration;

use dravr_enforme::rate_limit::TokenBucket;

#[test]
fn new_bucket_starts_full() {
    let bucket = TokenBucket::new(10, 1.0);
    assert_eq!(bucket.capacity(), 10);
    assert_eq!(bucket.available_tokens(), 10);
}

#[test]
fn acquire_consumes_token() {
    let mut bucket = TokenBucket::new(5, 1.0);
    assert!(bucket.try_acquire());
    assert_eq!(bucket.available_tokens(), 4);
}

#[test]
fn acquire_all_tokens() {
    let mut bucket = TokenBucket::new(3, 0.0);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(!bucket.try_acquire());
}

#[test]
fn wait_duration_zero_when_available() {
    let bucket = TokenBucket::new(5, 1.0);
    assert_eq!(bucket.wait_duration(), Duration::ZERO);
}

#[test]
fn wait_duration_nonzero_when_empty() {
    let mut bucket = TokenBucket::new(1, 1.0);
    bucket.try_acquire();
    let wait = bucket.wait_duration();
    assert!(wait > Duration::ZERO);
}

#[test]
fn capacity_returns_max_tokens() {
    let bucket = TokenBucket::new(100, 10.0);
    assert_eq!(bucket.capacity(), 100);
}

#[test]
fn bucket_is_debug() {
    let bucket = TokenBucket::new(5, 1.0);
    let debug = format!("{bucket:?}");
    assert!(debug.contains("TokenBucket"));
}

#[test]
fn zero_capacity_bucket() {
    let mut bucket = TokenBucket::new(0, 0.0);
    assert!(!bucket.try_acquire());
    assert_eq!(bucket.available_tokens(), 0);
}

#[test]
fn high_refill_rate_bucket() {
    let mut bucket = TokenBucket::new(1000, 1000.0);
    for _ in 0..1000 {
        assert!(bucket.try_acquire());
    }
}

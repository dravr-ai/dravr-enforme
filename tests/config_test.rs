// ABOUTME: Tests for SyncConfig environment variable parsing and defaults
// ABOUTME: Verifies default values and environment variable override behavior
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]

use dravr_enforme::config::SyncConfig;
use dravr_enforme::models::deletion::DeletionStrategy;

#[test]
fn sync_config_defaults() {
    let config = SyncConfig::default();
    assert_eq!(config.backfill_days, 30);
    assert_eq!(config.poll_interval_secs, 900);
    assert_eq!(config.max_concurrent_syncs, 3);
    assert!(config.webhook_base_url.is_none());
    assert_eq!(config.deletion_strategy, DeletionStrategy::SoftDelete);
    assert_eq!(config.tombstone_retention_days, 90);
}

#[test]
fn sync_config_deletion_policy() {
    let config = SyncConfig::default();
    let policy = config.deletion_policy();
    assert!(policy.is_soft_delete());
    assert_eq!(policy.tombstone_retention_days, 90);
}

#[test]
fn sync_config_hard_delete_policy() {
    let config = SyncConfig {
        deletion_strategy: DeletionStrategy::HardDelete,
        tombstone_retention_days: 0,
        ..SyncConfig::default()
    };
    let policy = config.deletion_policy();
    assert!(!policy.is_soft_delete());
}

#[test]
fn sync_config_is_debug() {
    let config = SyncConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("SyncConfig"));
}

#[test]
fn sync_config_is_clone() {
    let config = SyncConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.backfill_days, config.backfill_days);
    assert_eq!(cloned.poll_interval_secs, config.poll_interval_secs);
}

#[test]
fn sync_config_custom_values() {
    let config = SyncConfig {
        backfill_days: 7,
        poll_interval_secs: 300,
        max_concurrent_syncs: 5,
        webhook_base_url: Some("https://example.com/webhooks".to_owned()),
        deletion_strategy: DeletionStrategy::HardDelete,
        tombstone_retention_days: 0,
    };
    assert_eq!(config.backfill_days, 7);
    assert_eq!(config.poll_interval_secs, 300);
    assert_eq!(config.max_concurrent_syncs, 5);
    assert_eq!(
        config.webhook_base_url.as_deref(),
        Some("https://example.com/webhooks")
    );
}

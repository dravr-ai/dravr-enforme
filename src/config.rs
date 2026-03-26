// ABOUTME: Sync configuration loaded from environment variables
// ABOUTME: Controls backfill depth, polling intervals, concurrency, and deletion policy
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use crate::models::deletion::DeletionStrategy;

/// Default number of days to backfill on initial connect.
const DEFAULT_BACKFILL_DAYS: u32 = 30;

/// Default polling interval in seconds (15 minutes).
const DEFAULT_POLL_INTERVAL_SECS: u64 = 900;

/// Default maximum concurrent sync operations.
const DEFAULT_MAX_CONCURRENT_SYNCS: usize = 3;

/// Default tombstone retention in days for soft deletes.
const DEFAULT_TOMBSTONE_RETENTION_DAYS: u32 = 90;

/// Configuration for the sync orchestrator, loaded from environment variables.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Number of days to backfill on initial connect (`ENFORME_BACKFILL_DAYS`).
    pub backfill_days: u32,
    /// Polling interval in seconds (`ENFORME_POLL_INTERVAL_SECS`).
    pub poll_interval_secs: u64,
    /// Maximum concurrent sync operations (`ENFORME_MAX_CONCURRENT_SYNCS`).
    pub max_concurrent_syncs: usize,
    /// Base URL for webhook registration (`ENFORME_WEBHOOK_BASE_URL`).
    pub webhook_base_url: Option<String>,
    /// Deletion strategy: soft or hard (`ENFORME_DELETION_STRATEGY`).
    pub deletion_strategy: DeletionStrategy,
    /// Days to retain tombstones for soft deletes (`ENFORME_TOMBSTONE_RETENTION_DAYS`).
    pub tombstone_retention_days: u32,
}

impl SyncConfig {
    /// Load configuration from environment variables with defaults.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            backfill_days: parse_env("ENFORME_BACKFILL_DAYS", DEFAULT_BACKFILL_DAYS),
            poll_interval_secs: parse_env("ENFORME_POLL_INTERVAL_SECS", DEFAULT_POLL_INTERVAL_SECS),
            max_concurrent_syncs: parse_env(
                "ENFORME_MAX_CONCURRENT_SYNCS",
                DEFAULT_MAX_CONCURRENT_SYNCS,
            ),
            webhook_base_url: std::env::var("ENFORME_WEBHOOK_BASE_URL").ok(),
            deletion_strategy: parse_deletion_strategy(),
            tombstone_retention_days: parse_env(
                "ENFORME_TOMBSTONE_RETENTION_DAYS",
                DEFAULT_TOMBSTONE_RETENTION_DAYS,
            ),
        }
    }

    /// Build a `DeletionPolicy` from the current config.
    #[must_use]
    pub fn deletion_policy(&self) -> crate::models::deletion::DeletionPolicy {
        crate::models::deletion::DeletionPolicy {
            strategy: self.deletion_strategy.clone(),
            tombstone_retention_days: self.tombstone_retention_days,
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            backfill_days: DEFAULT_BACKFILL_DAYS,
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            max_concurrent_syncs: DEFAULT_MAX_CONCURRENT_SYNCS,
            webhook_base_url: None,
            deletion_strategy: DeletionStrategy::SoftDelete,
            tombstone_retention_days: DEFAULT_TOMBSTONE_RETENTION_DAYS,
        }
    }
}

/// Parse an environment variable with a default fallback.
fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Parse the deletion strategy from the `ENFORME_DELETION_STRATEGY` env var.
fn parse_deletion_strategy() -> DeletionStrategy {
    match std::env::var("ENFORME_DELETION_STRATEGY")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "hard" => DeletionStrategy::HardDelete,
        _ => DeletionStrategy::SoftDelete,
    }
}

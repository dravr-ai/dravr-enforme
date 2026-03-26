// ABOUTME: Write-path trait for storing and deleting sleep sessions
// ABOUTME: Accepts StoredSleepSession from dravr-equilibre
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;
use dravr_equilibre::StoredSleepSession;

use crate::error::EnformeResult;
use crate::models::deletion::DeletionPolicy;

/// Write-path trait for sleep session persistence.
#[async_trait]
pub trait SleepStore: Send + Sync {
    /// Store one or more sleep sessions, returning the number of records written.
    async fn store_sleep_sessions(&self, sessions: &[StoredSleepSession]) -> EnformeResult<u64>;

    /// Delete a sleep session by ID according to the deletion policy.
    async fn delete_sleep_session(&self, id: &str, policy: &DeletionPolicy) -> EnformeResult<()>;
}

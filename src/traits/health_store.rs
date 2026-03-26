// ABOUTME: Write-path trait for storing and deleting health metric snapshots
// ABOUTME: Accepts StoredHealthMetrics from dravr-equilibre
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;
use dravr_equilibre::StoredHealthMetrics;

use crate::error::EnformeResult;
use crate::models::deletion::DeletionPolicy;

/// Write-path trait for health metrics persistence.
#[async_trait]
pub trait HealthStore: Send + Sync {
    /// Store one or more health snapshots, returning the number of records written.
    async fn store_health_snapshots(&self, snapshots: &[StoredHealthMetrics])
        -> EnformeResult<u64>;

    /// Delete a health snapshot by ID according to the deletion policy.
    async fn delete_health_snapshot(&self, id: &str, policy: &DeletionPolicy) -> EnformeResult<()>;
}

// ABOUTME: Write-path trait for storing and deleting recovery metrics
// ABOUTME: Accepts StoredRecoveryMetrics from dravr-equilibre
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;
use dravr_equilibre::StoredRecoveryMetrics;

use crate::error::EnformeResult;
use crate::models::deletion::DeletionPolicy;

/// Write-path trait for recovery metrics persistence.
#[async_trait]
pub trait RecoveryStore: Send + Sync {
    /// Store one or more recovery metrics, returning the number of records written.
    async fn store_recovery_metrics(&self, metrics: &[StoredRecoveryMetrics])
        -> EnformeResult<u64>;

    /// Delete a recovery metric by ID according to the deletion policy.
    async fn delete_recovery_metric(&self, id: &str, policy: &DeletionPolicy) -> EnformeResult<()>;
}

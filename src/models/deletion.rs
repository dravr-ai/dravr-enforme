// ABOUTME: Deletion policy configuration for soft and hard delete strategies
// ABOUTME: Configurable tombstone retention for soft-deleted records
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use serde::{Deserialize, Serialize};

/// Policy governing how records are deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionPolicy {
    /// The deletion strategy to use.
    pub strategy: DeletionStrategy,
    /// Days to retain tombstone records before permanent removal.
    pub tombstone_retention_days: u32,
}

impl DeletionPolicy {
    /// Create a soft delete policy with the given retention period.
    #[must_use]
    pub fn soft(retention_days: u32) -> Self {
        Self {
            strategy: DeletionStrategy::SoftDelete,
            tombstone_retention_days: retention_days,
        }
    }

    /// Create a hard delete policy (immediate removal).
    #[must_use]
    pub fn hard() -> Self {
        Self {
            strategy: DeletionStrategy::HardDelete,
            tombstone_retention_days: 0,
        }
    }

    /// Check if this policy uses soft delete.
    #[must_use]
    pub fn is_soft_delete(&self) -> bool {
        matches!(self.strategy, DeletionStrategy::SoftDelete)
    }
}

impl Default for DeletionPolicy {
    fn default() -> Self {
        Self::soft(90)
    }
}

/// Strategy for handling record deletion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionStrategy {
    /// Mark records as deleted but retain them for the tombstone period.
    SoftDelete,
    /// Permanently remove records immediately.
    HardDelete,
}

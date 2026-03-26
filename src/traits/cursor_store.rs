// ABOUTME: Write-path trait for sync cursor persistence
// ABOUTME: Tracks incremental sync progress per user+provider+data_type
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;

use crate::error::EnformeResult;
use crate::models::cursor::SyncCursor;

/// Write-path trait for sync cursor persistence.
#[async_trait]
pub trait SyncCursorStore: Send + Sync {
    /// Get the cursor for a specific user, provider, and data type.
    async fn get_cursor(
        &self,
        user_id: &str,
        provider: &str,
        data_type: &str,
    ) -> EnformeResult<Option<SyncCursor>>;

    /// Update or create a sync cursor.
    async fn update_cursor(&self, cursor: &SyncCursor) -> EnformeResult<()>;
}

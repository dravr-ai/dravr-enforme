// ABOUTME: Sync cursor for tracking incremental sync progress per user+provider+data_type
// ABOUTME: Batch model for paginated provider API responses
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use chrono::{DateTime, Utc};
use dravr_equilibre::SyncStatus;
use serde::{Deserialize, Serialize};

/// Tracks sync progress for a specific user, provider, and data type combination.
///
/// Each cursor stores the provider-specific pagination token and metadata
/// about the last sync attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCursor {
    /// The user this cursor belongs to.
    pub user_id: String,
    /// The provider being synced (e.g., "whoop", "garmin").
    pub provider: String,
    /// The data type being synced (e.g., "sleep", "recovery", "health").
    pub data_type: String,
    /// Provider-specific cursor token for pagination.
    pub value: String,
    /// When the last sync completed.
    pub last_sync_at: DateTime<Utc>,
    /// Current sync status.
    pub status: SyncStatus,
    /// Total records synced in the last batch.
    pub records_synced: u64,
    /// Error message from the last failed sync, if any.
    pub error_message: Option<String>,
    /// Number of consecutive retry attempts.
    pub retry_count: u32,
    /// When to attempt the next retry, if applicable.
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl SyncCursor {
    /// Create a new cursor for the given user, provider, and data type.
    #[must_use]
    pub fn new(
        user_id: impl Into<String>,
        provider: impl Into<String>,
        data_type: impl Into<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            provider: provider.into(),
            data_type: data_type.into(),
            value: String::new(),
            last_sync_at: Utc::now(),
            status: SyncStatus::Pending,
            records_synced: 0,
            error_message: None,
            retry_count: 0,
            next_retry_at: None,
        }
    }

    /// Mark the cursor as completed with a new cursor value.
    pub fn complete(&mut self, new_value: impl Into<String>, records: u64) {
        self.value = new_value.into();
        self.last_sync_at = Utc::now();
        self.status = SyncStatus::Completed;
        self.records_synced = records;
        self.error_message = None;
        self.retry_count = 0;
        self.next_retry_at = None;
    }

    /// Mark the cursor as failed with an error message.
    pub fn fail(&mut self, message: impl Into<String>) {
        self.status = SyncStatus::Failed;
        self.error_message = Some(message.into());
        self.retry_count += 1;
    }
}

/// A batch of records returned from a provider API with pagination info.
#[derive(Debug, Clone)]
pub struct SyncBatch<T> {
    /// The records fetched in this batch.
    pub records: Vec<T>,
    /// Updated cursor reflecting the new sync position.
    pub cursor: SyncCursor,
    /// Whether more records are available for fetching.
    pub has_more: bool,
}

impl<T> SyncBatch<T> {
    /// Create an empty batch with no more pages.
    #[must_use]
    pub fn empty(cursor: SyncCursor) -> Self {
        Self {
            records: Vec::new(),
            cursor,
            has_more: false,
        }
    }

    /// Return the number of records in this batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Return true if this batch contains no records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

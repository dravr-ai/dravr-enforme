// ABOUTME: Background sync scheduler with jitter and priority queue
// ABOUTME: Periodically triggers sync for all connected users across providers
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::orchestrator::SyncOrchestrator;

/// A scheduled sync task in the priority queue.
#[derive(Debug, Clone)]
pub struct SyncTask {
    /// When this task should execute.
    pub scheduled_at: DateTime<Utc>,
    /// The user to sync.
    pub user_id: String,
    /// The provider to sync from.
    pub provider: String,
    /// Task priority (lower = higher priority).
    pub priority: u8,
}

impl PartialEq for SyncTask {
    fn eq(&self, other: &Self) -> bool {
        self.scheduled_at == other.scheduled_at && self.priority == other.priority
    }
}

impl Eq for SyncTask {}

impl PartialOrd for SyncTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SyncTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering: earliest time + highest priority first
        other
            .scheduled_at
            .cmp(&self.scheduled_at)
            .then(other.priority.cmp(&self.priority))
    }
}

/// A priority queue of sync tasks.
#[derive(Debug, Default)]
pub struct SyncQueue {
    heap: BinaryHeap<SyncTask>,
}

impl SyncQueue {
    /// Create a new empty sync queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Push a task onto the queue.
    pub fn push(&mut self, task: SyncTask) {
        self.heap.push(task);
    }

    /// Pop the highest-priority task that is ready to execute.
    pub fn pop_ready(&mut self) -> Option<SyncTask> {
        let now = Utc::now();
        if self.heap.peek().is_some_and(|t| t.scheduled_at <= now) {
            self.heap.pop()
        } else {
            None
        }
    }

    /// Return the number of tasks in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Check if the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

/// Add jitter to a duration to spread out scheduled tasks.
///
/// Returns a duration between `base * 0.8` and `base * 1.2`.
#[must_use]
pub fn with_jitter(base: Duration) -> Duration {
    let millis = base.as_millis() as u64;
    let jitter_range = millis / 5; // 20% jitter
    let offset = simple_random() % (jitter_range.max(1));
    let jittered = if simple_random().is_multiple_of(2) {
        millis.saturating_add(offset)
    } else {
        millis.saturating_sub(offset)
    };
    Duration::from_millis(jittered)
}

/// Simple pseudo-random number for jitter (not cryptographic).
fn simple_random() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Start the background sync scheduler loop.
pub fn start(orchestrator: Arc<SyncOrchestrator>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let interval = Duration::from_secs(orchestrator.config().poll_interval_secs);

        info!(interval_secs = interval.as_secs(), "Sync scheduler started");

        loop {
            let sleep_duration = with_jitter(interval);
            sleep(sleep_duration).await;

            for provider_name in orchestrator.provider_names() {
                match orchestrator
                    .deps()
                    .connections
                    .list_connected_users(provider_name)
                    .await
                {
                    Ok(users) => {
                        for user in &users {
                            if !user.is_active {
                                continue;
                            }
                            match orchestrator.sync_user(&user.user_id, provider_name).await {
                                Ok(result) => {
                                    info!(
                                        user_id = user.user_id,
                                        provider = provider_name,
                                        created = result.records_created,
                                        "Scheduled sync completed"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        user_id = user.user_id,
                                        provider = provider_name,
                                        error = %e,
                                        "Scheduled sync failed"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            provider = provider_name,
                            error = %e,
                            "Failed to list connected users"
                        );
                    }
                }
            }
        }
    })
}

// ABOUTME: Tests for the sync scheduler priority queue and jitter logic
// ABOUTME: Verifies task ordering, readiness checking, and jitter bounds
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::time::Duration;

use chrono::Utc;
use dravr_enforme::orchestrator::scheduler::{with_jitter, SyncQueue, SyncTask};

#[test]
fn sync_queue_starts_empty() {
    let queue = SyncQueue::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
}

#[test]
fn sync_queue_push_increases_len() {
    let mut queue = SyncQueue::new();
    queue.push(SyncTask {
        scheduled_at: Utc::now(),
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    });
    assert_eq!(queue.len(), 1);
    assert!(!queue.is_empty());
}

#[test]
fn sync_queue_pop_ready_returns_past_tasks() {
    let mut queue = SyncQueue::new();
    queue.push(SyncTask {
        scheduled_at: Utc::now() - chrono::Duration::seconds(10),
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    });
    let task = queue.pop_ready();
    assert!(task.is_some());
    assert_eq!(task.unwrap().user_id, "user-1");
}

#[test]
fn sync_queue_pop_ready_skips_future_tasks() {
    let mut queue = SyncQueue::new();
    queue.push(SyncTask {
        scheduled_at: Utc::now() + chrono::Duration::hours(1),
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    });
    assert!(queue.pop_ready().is_none());
}

#[test]
fn sync_queue_priority_ordering() {
    let mut queue = SyncQueue::new();
    let past = Utc::now() - chrono::Duration::seconds(10);

    queue.push(SyncTask {
        scheduled_at: past,
        user_id: "low-priority".to_owned(),
        provider: "whoop".to_owned(),
        priority: 10,
    });
    queue.push(SyncTask {
        scheduled_at: past,
        user_id: "high-priority".to_owned(),
        provider: "whoop".to_owned(),
        priority: 1,
    });

    // Higher priority (lower number) should come first
    let first = queue.pop_ready().unwrap();
    assert_eq!(first.user_id, "high-priority");
}

#[test]
fn sync_queue_time_ordering() {
    let mut queue = SyncQueue::new();

    queue.push(SyncTask {
        scheduled_at: Utc::now() - chrono::Duration::seconds(20),
        user_id: "earlier".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    });
    queue.push(SyncTask {
        scheduled_at: Utc::now() - chrono::Duration::seconds(5),
        user_id: "later".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    });

    // Earlier task should come first
    let first = queue.pop_ready().unwrap();
    assert_eq!(first.user_id, "earlier");
}

#[test]
fn with_jitter_stays_within_bounds() {
    let base = Duration::from_secs(100);
    for _ in 0..100 {
        let jittered = with_jitter(base);
        // Should be between 80% and 120% of base
        assert!(jittered >= base * 80 / 100);
        assert!(jittered <= base * 120 / 100);
    }
}

#[test]
fn with_jitter_zero_duration() {
    let jittered = with_jitter(Duration::ZERO);
    // Should be zero or very small
    assert!(jittered <= Duration::from_millis(1));
}

#[test]
fn sync_task_equality() {
    let task1 = SyncTask {
        scheduled_at: Utc::now(),
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    };
    let task2 = SyncTask {
        scheduled_at: task1.scheduled_at,
        user_id: "user-2".to_owned(),
        provider: "garmin".to_owned(),
        priority: 0,
    };
    // Equality is based on scheduled_at and priority only
    assert_eq!(task1, task2);
}

#[test]
fn sync_task_is_debug() {
    let task = SyncTask {
        scheduled_at: Utc::now(),
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
        priority: 0,
    };
    let debug = format!("{task:?}");
    assert!(debug.contains("SyncTask"));
}

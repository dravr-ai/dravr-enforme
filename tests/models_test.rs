// ABOUTME: Tests for domain models: SyncCursor, SyncBatch, DeletionPolicy, WebhookConfig
// ABOUTME: Covers serialization, construction, and helper methods
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]

use dravr_enforme::models::connection::{ConnectedUser, ProviderCredentials};
use dravr_enforme::models::cursor::{SyncBatch, SyncCursor};
use dravr_enforme::models::deletion::{DeletionPolicy, DeletionStrategy};
use dravr_enforme::models::webhook::{WebhookAlgorithm, WebhookConfig, WebhookEvent};

// ============================================================================
// SyncCursor tests
// ============================================================================

#[test]
fn sync_cursor_new_defaults() {
    let cursor = SyncCursor::new("user-1", "whoop", "sleep");
    assert_eq!(cursor.user_id, "user-1");
    assert_eq!(cursor.provider, "whoop");
    assert_eq!(cursor.data_type, "sleep");
    assert!(cursor.value.is_empty());
    assert_eq!(cursor.records_synced, 0);
    assert_eq!(cursor.retry_count, 0);
    assert!(cursor.error_message.is_none());
    assert!(cursor.next_retry_at.is_none());
}

#[test]
fn sync_cursor_complete_updates_fields() {
    let mut cursor = SyncCursor::new("user-1", "whoop", "sleep");
    cursor.complete("next-token-abc", 42);
    assert_eq!(cursor.value, "next-token-abc");
    assert_eq!(cursor.records_synced, 42);
    assert!(cursor.error_message.is_none());
    assert_eq!(cursor.retry_count, 0);
}

#[test]
fn sync_cursor_fail_increments_retry() {
    let mut cursor = SyncCursor::new("user-1", "whoop", "sleep");
    cursor.fail("timeout");
    assert_eq!(cursor.retry_count, 1);
    assert_eq!(cursor.error_message.as_deref(), Some("timeout"));

    cursor.fail("timeout again");
    assert_eq!(cursor.retry_count, 2);
}

#[test]
fn sync_cursor_serialization_roundtrip() {
    let cursor = SyncCursor::new("user-1", "whoop", "sleep");
    let json = serde_json::to_string(&cursor).unwrap();
    let deserialized: SyncCursor = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, cursor.user_id);
    assert_eq!(deserialized.provider, cursor.provider);
    assert_eq!(deserialized.data_type, cursor.data_type);
}

// ============================================================================
// SyncBatch tests
// ============================================================================

#[test]
fn sync_batch_empty_has_no_records() {
    let cursor = SyncCursor::new("user-1", "whoop", "sleep");
    let batch: SyncBatch<String> = SyncBatch::empty(cursor);
    assert!(batch.is_empty());
    assert_eq!(batch.len(), 0);
    assert!(!batch.has_more);
}

#[test]
fn sync_batch_with_records() {
    let cursor = SyncCursor::new("user-1", "whoop", "sleep");
    let batch = SyncBatch {
        records: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        cursor,
        has_more: true,
    };
    assert_eq!(batch.len(), 3);
    assert!(!batch.is_empty());
    assert!(batch.has_more);
}

// ============================================================================
// DeletionPolicy tests
// ============================================================================

#[test]
fn deletion_policy_soft_default() {
    let policy = DeletionPolicy::default();
    assert!(policy.is_soft_delete());
    assert_eq!(policy.tombstone_retention_days, 90);
}

#[test]
fn deletion_policy_soft_custom_retention() {
    let policy = DeletionPolicy::soft(30);
    assert!(policy.is_soft_delete());
    assert_eq!(policy.tombstone_retention_days, 30);
}

#[test]
fn deletion_policy_hard() {
    let policy = DeletionPolicy::hard();
    assert!(!policy.is_soft_delete());
    assert_eq!(policy.tombstone_retention_days, 0);
}

#[test]
fn deletion_strategy_serialization() {
    let soft = serde_json::to_string(&DeletionStrategy::SoftDelete).unwrap();
    assert!(soft.contains("SoftDelete"));
    let hard = serde_json::to_string(&DeletionStrategy::HardDelete).unwrap();
    assert!(hard.contains("HardDelete"));
}

#[test]
fn deletion_policy_serialization_roundtrip() {
    let policy = DeletionPolicy::soft(45);
    let json = serde_json::to_string(&policy).unwrap();
    let deserialized: DeletionPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tombstone_retention_days, 45);
    assert!(deserialized.is_soft_delete());
}

// ============================================================================
// WebhookEvent tests
// ============================================================================

#[test]
fn webhook_event_creation() {
    let event = WebhookEvent::new("whoop", "sleep.updated", "user-1", "resource-123");
    assert_eq!(event.provider, "whoop");
    assert_eq!(event.event_type, "sleep.updated");
    assert_eq!(event.user_id, "user-1");
    assert_eq!(event.resource_id, "resource-123");
}

#[test]
fn webhook_event_is_deletion() {
    let deletion = WebhookEvent::new("whoop", "sleep.deleted", "user-1", "123");
    assert!(deletion.is_deletion());
    assert!(!deletion.is_update());

    let update = WebhookEvent::new("whoop", "sleep.updated", "user-1", "123");
    assert!(!update.is_deletion());
    assert!(update.is_update());
}

#[test]
fn webhook_event_data_type_extraction() {
    let event = WebhookEvent::new("whoop", "sleep.updated", "user-1", "123");
    assert_eq!(event.data_type(), Some("sleep"));

    let event = WebhookEvent::new("whoop", "recovery.deleted", "user-1", "456");
    assert_eq!(event.data_type(), Some("recovery"));
}

#[test]
fn webhook_event_serialization_roundtrip() {
    let event = WebhookEvent::new("whoop", "sleep.updated", "user-1", "123");
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: WebhookEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.provider, "whoop");
    assert_eq!(deserialized.event_type, "sleep.updated");
}

#[test]
fn webhook_algorithm_equality() {
    assert_eq!(WebhookAlgorithm::HmacSha256, WebhookAlgorithm::HmacSha256);
    assert_ne!(WebhookAlgorithm::HmacSha256, WebhookAlgorithm::HmacSha1);
}

#[test]
fn webhook_config_construction() {
    let config = WebhookConfig {
        signature_header: "X-Whoop-Signature",
        algorithm: WebhookAlgorithm::HmacSha256,
        needs_verification: true,
    };
    assert_eq!(config.signature_header, "X-Whoop-Signature");
    assert!(config.needs_verification);
}

// ============================================================================
// ConnectedUser tests
// ============================================================================

#[test]
fn connected_user_creation() {
    let user = ConnectedUser::new("user-1", "whoop");
    assert_eq!(user.user_id, "user-1");
    assert_eq!(user.provider, "whoop");
    assert!(user.is_active);
}

#[test]
fn connected_user_serialization_roundtrip() {
    let user = ConnectedUser::new("user-1", "garmin");
    let json = serde_json::to_string(&user).unwrap();
    let deserialized: ConnectedUser = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, "user-1");
    assert_eq!(deserialized.provider, "garmin");
}

// ============================================================================
// ProviderCredentials tests
// ============================================================================

#[test]
fn credentials_not_expired_when_no_expiry() {
    let creds = ProviderCredentials {
        access_token: "token".to_owned(),
        refresh_token: None,
        expires_at: None,
        scopes: vec![],
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
    };
    assert!(!creds.is_expired());
}

#[test]
fn credentials_expired_when_past() {
    let creds = ProviderCredentials {
        access_token: "token".to_owned(),
        refresh_token: None,
        expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        scopes: vec![],
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
    };
    assert!(creds.is_expired());
}

#[test]
fn credentials_can_refresh_with_refresh_token() {
    let creds = ProviderCredentials {
        access_token: "token".to_owned(),
        refresh_token: Some("refresh".to_owned()),
        expires_at: None,
        scopes: vec![],
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
    };
    assert!(creds.can_refresh());
}

#[test]
fn credentials_cannot_refresh_without_refresh_token() {
    let creds = ProviderCredentials {
        access_token: "token".to_owned(),
        refresh_token: None,
        expires_at: None,
        scopes: vec![],
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
    };
    assert!(!creds.can_refresh());
}

// ABOUTME: Tests for SyncOrchestrator construction and configuration
// ABOUTME: Verifies provider registration, rate limiting, and config access
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dravr_equilibre::{
    ContinuousMetricBatch, DataSource, StoredHealthMetrics, StoredRecoveryMetrics,
    StoredSleepSession,
};

use dravr_enforme::config::SyncConfig;
use dravr_enforme::error::EnformeResult;
use dravr_enforme::models::connection::{ConnectedUser, ProviderCredentials};
use dravr_enforme::models::cursor::{SyncBatch, SyncCursor};
use dravr_enforme::models::deletion::DeletionPolicy;
use dravr_enforme::models::webhook::{WebhookAlgorithm, WebhookConfig, WebhookEvent};
use dravr_enforme::orchestrator::SyncOrchestrator;
use dravr_enforme::traits::connect_hook::OnConnectHook;
use dravr_enforme::traits::connection_store::UserConnectionStore;
use dravr_enforme::traits::credential_store::CredentialStore;
use dravr_enforme::traits::cursor_store::SyncCursorStore;
use dravr_enforme::traits::data_source_store::DataSourceStore;
use dravr_enforme::traits::health_store::HealthStore;
use dravr_enforme::traits::recovery_store::RecoveryStore;
use dravr_enforme::traits::sleep_store::SleepStore;
use dravr_enforme::traits::sync_provider::{DataType, SyncProvider};
use dravr_enforme::traits::timeseries_store::TimeSeriesPointStore;
use dravr_enforme::traits::SyncDeps;

// ============================================================================
// Mock implementations
// ============================================================================

struct MockSleepStore;
#[async_trait]
impl SleepStore for MockSleepStore {
    async fn store_sleep_sessions(&self, sessions: &[StoredSleepSession]) -> EnformeResult<u64> {
        Ok(sessions.len() as u64)
    }
    async fn delete_sleep_session(&self, _id: &str, _policy: &DeletionPolicy) -> EnformeResult<()> {
        Ok(())
    }
}

struct MockRecoveryStore;
#[async_trait]
impl RecoveryStore for MockRecoveryStore {
    async fn store_recovery_metrics(
        &self,
        metrics: &[StoredRecoveryMetrics],
    ) -> EnformeResult<u64> {
        Ok(metrics.len() as u64)
    }
    async fn delete_recovery_metric(
        &self,
        _id: &str,
        _policy: &DeletionPolicy,
    ) -> EnformeResult<()> {
        Ok(())
    }
}

struct MockHealthStore;
#[async_trait]
impl HealthStore for MockHealthStore {
    async fn store_health_snapshots(
        &self,
        snapshots: &[StoredHealthMetrics],
    ) -> EnformeResult<u64> {
        Ok(snapshots.len() as u64)
    }
    async fn delete_health_snapshot(
        &self,
        _id: &str,
        _policy: &DeletionPolicy,
    ) -> EnformeResult<()> {
        Ok(())
    }
}

struct MockTimeSeriesStore;
#[async_trait]
impl TimeSeriesPointStore for MockTimeSeriesStore {
    async fn store_continuous_metrics(
        &self,
        _source_id: &str,
        batches: &[ContinuousMetricBatch],
    ) -> EnformeResult<u64> {
        Ok(batches.len() as u64)
    }
}

struct MockDataSourceStore;
#[async_trait]
impl DataSourceStore for MockDataSourceStore {
    async fn upsert_data_source(&self, source: &DataSource) -> EnformeResult<String> {
        Ok(source.id.clone())
    }
}

struct MockCursorStore;
#[async_trait]
impl SyncCursorStore for MockCursorStore {
    async fn get_cursor(
        &self,
        user_id: &str,
        provider: &str,
        data_type: &str,
    ) -> EnformeResult<Option<SyncCursor>> {
        Ok(Some(SyncCursor::new(user_id, provider, data_type)))
    }
    async fn update_cursor(&self, _cursor: &SyncCursor) -> EnformeResult<()> {
        Ok(())
    }
}

struct MockCredentialStore;
#[async_trait]
impl CredentialStore for MockCredentialStore {
    async fn get_credentials(
        &self,
        user_id: &str,
        provider: &str,
    ) -> EnformeResult<Option<ProviderCredentials>> {
        Ok(Some(ProviderCredentials {
            access_token: "mock-token".to_owned(),
            refresh_token: None,
            expires_at: None,
            scopes: vec![],
            user_id: user_id.to_owned(),
            provider: provider.to_owned(),
        }))
    }
    async fn refresh_credentials(
        &self,
        user_id: &str,
        provider: &str,
    ) -> EnformeResult<ProviderCredentials> {
        Ok(ProviderCredentials {
            access_token: "refreshed-token".to_owned(),
            refresh_token: None,
            expires_at: None,
            scopes: vec![],
            user_id: user_id.to_owned(),
            provider: provider.to_owned(),
        })
    }
}

struct MockConnectionStore;
#[async_trait]
impl UserConnectionStore for MockConnectionStore {
    async fn list_connected_users(&self, provider: &str) -> EnformeResult<Vec<ConnectedUser>> {
        Ok(vec![ConnectedUser::new("user-1", provider)])
    }
}

struct MockProvider;
#[async_trait]
impl SyncProvider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }
    fn supported_data_types(&self) -> &[DataType] {
        &[DataType::Sleep]
    }

    async fn fetch_sleep(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredSleepSession>> {
        let cursor = SyncCursor::new(&creds.user_id, "mock", "sleep");
        Ok(SyncBatch::empty(cursor))
    }
    async fn fetch_recovery(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredRecoveryMetrics>> {
        let cursor = SyncCursor::new(&creds.user_id, "mock", "recovery");
        Ok(SyncBatch::empty(cursor))
    }
    async fn fetch_health(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredHealthMetrics>> {
        let cursor = SyncCursor::new(&creds.user_id, "mock", "health");
        Ok(SyncBatch::empty(cursor))
    }
    async fn fetch_continuous(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>> {
        let cursor = SyncCursor::new(&creds.user_id, "mock", "continuous");
        Ok(SyncBatch::empty(cursor))
    }
    async fn on_connected(
        &self,
        _creds: &ProviderCredentials,
        _webhook_url: &str,
    ) -> EnformeResult<()> {
        Ok(())
    }
    async fn on_disconnected(&self, _creds: &ProviderCredentials) -> EnformeResult<()> {
        Ok(())
    }
    fn webhook_config(&self) -> Option<WebhookConfig> {
        Some(WebhookConfig {
            signature_header: "x-mock-sig",
            algorithm: WebhookAlgorithm::HmacSha256,
            needs_verification: false,
        })
    }
    async fn validate_webhook(
        &self,
        _headers: &http::HeaderMap,
        _body: &[u8],
    ) -> EnformeResult<bool> {
        Ok(true)
    }
    async fn parse_webhook(&self, _body: &[u8]) -> EnformeResult<Vec<WebhookEvent>> {
        Ok(vec![])
    }
}

fn build_mock_deps() -> Arc<SyncDeps> {
    Arc::new(SyncDeps {
        sleep: Arc::new(MockSleepStore),
        recovery: Arc::new(MockRecoveryStore),
        health: Arc::new(MockHealthStore),
        time_series: Arc::new(MockTimeSeriesStore),
        data_sources: Arc::new(MockDataSourceStore),
        cursors: Arc::new(MockCursorStore),
        credentials: Arc::new(MockCredentialStore),
        connections: Arc::new(MockConnectionStore),
    })
}

fn build_orchestrator() -> SyncOrchestrator {
    let deps = build_mock_deps();
    let mut providers: HashMap<String, Box<dyn SyncProvider>> = HashMap::new();
    providers.insert("mock".to_owned(), Box::new(MockProvider));
    SyncOrchestrator::new(deps, providers, SyncConfig::default())
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn orchestrator_lists_providers() {
    let orch = build_orchestrator();
    let names = orch.provider_names();
    assert_eq!(names, vec!["mock"]);
}

#[test]
fn orchestrator_config_returns_defaults() {
    let orch = build_orchestrator();
    assert_eq!(orch.config().backfill_days, 30);
    assert_eq!(orch.config().poll_interval_secs, 900);
}

#[test]
fn orchestrator_is_debug() {
    let orch = build_orchestrator();
    let debug = format!("{orch:?}");
    assert!(debug.contains("SyncOrchestrator"));
}

#[tokio::test]
async fn orchestrator_sync_user_with_mock() {
    let orch = build_orchestrator();
    let result = orch.sync_user("user-1", "mock").await.unwrap();
    assert_eq!(result.provider, "mock");
    assert_eq!(result.user_id, "user-1");
}

#[tokio::test]
async fn orchestrator_sync_unknown_provider_errors() {
    let orch = build_orchestrator();
    let result = orch.sync_user("user-1", "nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn orchestrator_backfill_with_mock() {
    let orch = build_orchestrator();
    let result = orch.backfill("user-1", "mock", 30).await.unwrap();
    assert_eq!(result.provider, "mock");
}

#[test]
fn orchestrator_check_rate_limit() {
    let mut orch = build_orchestrator();
    assert!(orch.check_rate_limit("mock"));
    assert!(orch.check_rate_limit("unknown"));
}

#[test]
fn sync_deps_is_debug() {
    let deps = build_mock_deps();
    let debug = format!("{deps:?}");
    assert!(debug.contains("SyncDeps"));
}

#[test]
fn mock_on_connect_hook_is_send_sync() {
    struct MockHook;
    #[async_trait]
    impl OnConnectHook for MockHook {
        async fn on_provider_connected(
            &self,
            _user_id: &str,
            _provider: &str,
            _credentials: &ProviderCredentials,
        ) -> EnformeResult<()> {
            Ok(())
        }
        async fn on_provider_disconnected(
            &self,
            _user_id: &str,
            _provider: &str,
        ) -> EnformeResult<()> {
            Ok(())
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockHook>();
}

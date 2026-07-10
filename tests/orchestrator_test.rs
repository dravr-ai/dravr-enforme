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

// ============================================================================
// Data-source stamping and credential-refresh behavior
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use chrono::{Duration, Utc};
use dravr_equilibre::SyncStatus;

fn sample_sleep_session(user_id: &str) -> StoredSleepSession {
    StoredSleepSession {
        id: "sleep-1".to_owned(),
        user_id: user_id.to_owned(),
        data_source_id: "mock-default".to_owned(),
        is_nap: false,
        start_datetime: Utc::now() - Duration::hours(8),
        end_datetime: Utc::now(),
        total_sleep_seconds: Some(7 * 3600),
        deep_sleep_seconds: None,
        light_sleep_seconds: None,
        rem_sleep_seconds: None,
        awake_seconds: None,
        sleep_efficiency: None,
        avg_heart_rate: None,
        min_heart_rate: None,
        avg_hrv: None,
        sleep_score: None,
        stages: Vec::new(),
        source_name: "mock".to_owned(),
    }
}

/// Sleep store that captures stored sessions for assertions.
struct CapturingSleepStore {
    stored: Arc<Mutex<Vec<StoredSleepSession>>>,
}
#[async_trait]
impl SleepStore for CapturingSleepStore {
    async fn store_sleep_sessions(&self, sessions: &[StoredSleepSession]) -> EnformeResult<u64> {
        self.stored.lock().unwrap().extend_from_slice(sessions);
        Ok(sessions.len() as u64)
    }
    async fn delete_sleep_session(&self, _id: &str, _policy: &DeletionPolicy) -> EnformeResult<()> {
        Ok(())
    }
}

/// Data source store returning a fixed persisted id, as the platform's
/// upsert does once the row exists.
struct FixedDataSourceStore;
#[async_trait]
impl DataSourceStore for FixedDataSourceStore {
    async fn upsert_data_source(&self, _source: &DataSource) -> EnformeResult<String> {
        Ok("ds-real-1".to_owned())
    }
}

/// Credential store handing out a stale token until refreshed.
struct StaleCredentialStore {
    refresh_calls: Arc<AtomicUsize>,
    stored_token_expired: bool,
    refresh_yields_fresh_token: bool,
}
#[async_trait]
impl CredentialStore for StaleCredentialStore {
    async fn get_credentials(
        &self,
        user_id: &str,
        provider: &str,
    ) -> EnformeResult<Option<ProviderCredentials>> {
        Ok(Some(ProviderCredentials {
            access_token: "stale-token".to_owned(),
            refresh_token: Some("refresh-token".to_owned()),
            expires_at: self
                .stored_token_expired
                .then(|| Utc::now() - Duration::hours(1)),
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
        self.refresh_calls.fetch_add(1, Ordering::SeqCst);
        let token = if self.refresh_yields_fresh_token {
            "fresh-token"
        } else {
            "stale-token"
        };
        Ok(ProviderCredentials {
            access_token: token.to_owned(),
            refresh_token: Some("refresh-token".to_owned()),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            scopes: vec![],
            user_id: user_id.to_owned(),
            provider: provider.to_owned(),
        })
    }
}

/// Provider that rejects stale tokens the way a live API's 401 does after
/// status-code mapping, and returns one sleep record otherwise.
struct AuthAwareProvider;
#[async_trait]
impl SyncProvider for AuthAwareProvider {
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
        if creds.access_token == "stale-token" {
            return Err(dravr_enforme::EnformeError::CredentialsExpired {
                user_id: creds.user_id.clone(),
                provider: "mock".to_owned(),
            });
        }
        let mut cursor = SyncCursor::new(&creds.user_id, "mock", "sleep");
        cursor.records_synced = 1;
        Ok(SyncBatch {
            records: vec![sample_sleep_session(&creds.user_id)],
            cursor,
            has_more: false,
        })
    }
    async fn fetch_recovery(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredRecoveryMetrics>> {
        Ok(SyncBatch::empty(SyncCursor::new(
            &creds.user_id,
            "mock",
            "recovery",
        )))
    }
    async fn fetch_health(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredHealthMetrics>> {
        Ok(SyncBatch::empty(SyncCursor::new(
            &creds.user_id,
            "mock",
            "health",
        )))
    }
    async fn fetch_continuous(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>> {
        Ok(SyncBatch::empty(SyncCursor::new(
            &creds.user_id,
            "mock",
            "continuous",
        )))
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
        None
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

struct RefreshScenario {
    orchestrator: SyncOrchestrator,
    stored: Arc<Mutex<Vec<StoredSleepSession>>>,
    refresh_calls: Arc<AtomicUsize>,
}

fn build_refresh_scenario(
    stored_token_expired: bool,
    refresh_yields_fresh_token: bool,
) -> RefreshScenario {
    let stored = Arc::new(Mutex::new(Vec::new()));
    let refresh_calls = Arc::new(AtomicUsize::new(0));
    let deps = Arc::new(SyncDeps {
        sleep: Arc::new(CapturingSleepStore {
            stored: Arc::clone(&stored),
        }),
        recovery: Arc::new(MockRecoveryStore),
        health: Arc::new(MockHealthStore),
        time_series: Arc::new(MockTimeSeriesStore),
        data_sources: Arc::new(FixedDataSourceStore),
        cursors: Arc::new(MockCursorStore),
        credentials: Arc::new(StaleCredentialStore {
            refresh_calls: Arc::clone(&refresh_calls),
            stored_token_expired,
            refresh_yields_fresh_token,
        }),
        connections: Arc::new(MockConnectionStore),
    });
    let mut providers: HashMap<String, Box<dyn SyncProvider>> = HashMap::new();
    providers.insert("mock".to_owned(), Box::new(AuthAwareProvider));
    RefreshScenario {
        orchestrator: SyncOrchestrator::new(deps, providers, SyncConfig::default()),
        stored,
        refresh_calls,
    }
}

#[tokio::test]
async fn sync_stamps_records_with_upserted_data_source_id() {
    let scenario = build_refresh_scenario(true, true);
    let result = scenario
        .orchestrator
        .sync_user("user-1", "mock")
        .await
        .unwrap();
    assert_eq!(result.records_created, 1);

    let stored = scenario.stored.lock().unwrap();
    assert_eq!(stored.len(), 1);
    // The provider stamped "mock-default"; the orchestrator must replace it
    // with the persisted data-source id so store-side foreign keys resolve.
    assert_eq!(stored[0].data_source_id, "ds-real-1");
}

#[tokio::test]
async fn sync_refreshes_and_retries_when_fetch_reports_expiry() {
    // Stored expiry looks valid, so no proactive refresh happens; the
    // provider's CredentialsExpired must trigger refresh-and-retry-once.
    let scenario = build_refresh_scenario(false, true);
    let result = scenario
        .orchestrator
        .sync_user("user-1", "mock")
        .await
        .unwrap();

    assert_eq!(scenario.refresh_calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.records_created, 1);
    assert_eq!(result.records_errored, 0);
    assert!(matches!(result.status, SyncStatus::Completed));
}

#[tokio::test]
async fn sync_proactively_refreshes_expired_stored_token() {
    // Stored token is already past expiry: refresh happens before any fetch.
    let scenario = build_refresh_scenario(true, true);
    let result = scenario
        .orchestrator
        .sync_user("user-1", "mock")
        .await
        .unwrap();

    assert_eq!(scenario.refresh_calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.records_created, 1);
    assert!(matches!(result.status, SyncStatus::Completed));
}

#[tokio::test]
async fn sync_refreshes_only_once_when_token_stays_invalid() {
    // Refresh keeps returning a rejected token: the sync must fail the data
    // type after a single retry instead of refreshing in a loop.
    let scenario = build_refresh_scenario(false, false);
    let result = scenario
        .orchestrator
        .sync_user("user-1", "mock")
        .await
        .unwrap();

    assert_eq!(scenario.refresh_calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.records_created, 0);
    assert_eq!(result.records_errored, 1);
    assert!(matches!(result.status, SyncStatus::Failed));
    assert!(scenario.stored.lock().unwrap().is_empty());
}

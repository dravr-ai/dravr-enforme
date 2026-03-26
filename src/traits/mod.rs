// ABOUTME: Store traits and provider abstractions for the sync orchestrator
// ABOUTME: Eight granular write-path traits following Interface Segregation principle
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

/// Hook called when a provider is connected or disconnected
pub mod connect_hook;
/// User connection storage trait
pub mod connection_store;
/// Provider credential storage trait
pub mod credential_store;
/// Sync cursor storage trait
pub mod cursor_store;
/// Data source storage trait
pub mod data_source_store;
/// Health metrics storage trait
pub mod health_store;
/// Recovery metrics storage trait
pub mod recovery_store;
/// Sleep session storage trait
pub mod sleep_store;
/// Sync provider trait for per-provider implementations
pub mod sync_provider;
/// Time series point storage trait
pub mod timeseries_store;

/// Dependency container holding all store trait objects.
///
/// Uses `Arc<dyn Trait>` to avoid generic parameter explosion.
/// Pierre-core will construct this and pass it to the `SyncOrchestrator`.
pub struct SyncDeps {
    /// Sleep session store.
    pub sleep: Arc<dyn sleep_store::SleepStore>,
    /// Recovery metrics store.
    pub recovery: Arc<dyn recovery_store::RecoveryStore>,
    /// Health metrics store.
    pub health: Arc<dyn health_store::HealthStore>,
    /// Time series point store.
    pub time_series: Arc<dyn timeseries_store::TimeSeriesPointStore>,
    /// Data source store.
    pub data_sources: Arc<dyn data_source_store::DataSourceStore>,
    /// Sync cursor store.
    pub cursors: Arc<dyn cursor_store::SyncCursorStore>,
    /// Provider credential store.
    pub credentials: Arc<dyn credential_store::CredentialStore>,
    /// User connection store.
    pub connections: Arc<dyn connection_store::UserConnectionStore>,
}

impl std::fmt::Debug for SyncDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncDeps").finish_non_exhaustive()
    }
}

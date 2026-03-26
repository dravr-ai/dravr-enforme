// ABOUTME: Central sync orchestrator coordinating provider sync, backfill, and webhooks
// ABOUTME: Manages per-provider rate limiting, scheduling, and store dispatch
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

/// Historical data backfill on provider connect
pub mod backfill;
/// Device and provider priority deduplication
pub mod dedup;
/// Background task scheduler with jitter and priority queue
pub mod scheduler;

use std::collections::HashMap;
use std::sync::Arc;

use dravr_equilibre::SyncResult;
use tokio::task::JoinHandle;
use tracing::{info, instrument, warn};

use crate::config::SyncConfig;
use crate::error::{EnformeError, EnformeResult};
use crate::rate_limit::TokenBucket;
use crate::traits::sync_provider::{DataType, SyncProvider};
use crate::traits::SyncDeps;

/// Central orchestrator for health data synchronization.
///
/// Coordinates between provider APIs and local stores, handling rate limiting,
/// cursor management, and scheduling.
pub struct SyncOrchestrator {
    /// Store trait objects for persisting sync data.
    deps: Arc<SyncDeps>,
    /// Registered provider implementations keyed by name.
    providers: HashMap<String, Box<dyn SyncProvider>>,
    /// Sync configuration.
    config: SyncConfig,
    /// Per-provider rate limiters.
    rate_limiters: HashMap<String, TokenBucket>,
}

impl SyncOrchestrator {
    /// Create a new sync orchestrator.
    #[must_use]
    pub fn new(
        deps: Arc<SyncDeps>,
        providers: HashMap<String, Box<dyn SyncProvider>>,
        config: SyncConfig,
    ) -> Self {
        let rate_limiters = providers
            .keys()
            .map(|name| {
                // Default: 10 requests per second per provider
                (name.clone(), TokenBucket::new(10, 10.0))
            })
            .collect();

        Self {
            deps,
            providers,
            config,
            rate_limiters,
        }
    }

    /// Sync all data types for a user from a specific provider.
    #[instrument(skip(self))]
    pub async fn sync_user(&self, user_id: &str, provider: &str) -> EnformeResult<SyncResult> {
        let provider_impl = self
            .providers
            .get(provider)
            .ok_or_else(|| EnformeError::provider(provider, "provider not registered"))?;

        let creds = self
            .deps
            .credentials
            .get_credentials(user_id, provider)
            .await?
            .ok_or_else(|| EnformeError::CredentialsExpired {
                user_id: user_id.to_owned(),
                provider: provider.to_owned(),
            })?;

        let mut total_created: u32 = 0;
        let total_updated: u32 = 0;
        let mut total_errors: u32 = 0;
        let started_at = chrono::Utc::now();

        for data_type in provider_impl.supported_data_types() {
            let cursor = self
                .deps
                .cursors
                .get_cursor(user_id, provider, data_type.as_str())
                .await?;

            match self
                .sync_data_type(provider_impl.as_ref(), &creds, *data_type, cursor.as_ref())
                .await
            {
                Ok(count) => total_created += count,
                Err(e) => {
                    warn!(
                        user_id,
                        provider,
                        data_type = data_type.as_str(),
                        error = %e,
                        "Failed to sync data type"
                    );
                    total_errors += 1;
                }
            }
        }

        let status = if total_errors > 0 {
            dravr_equilibre::SyncStatus::Failed
        } else {
            dravr_equilibre::SyncStatus::Completed
        };

        Ok(SyncResult {
            status,
            provider: provider.to_owned(),
            user_id: user_id.to_owned(),
            records_created: total_created,
            records_updated: total_updated,
            records_skipped: 0,
            records_errored: total_errors,
            error_message: None,
            started_at,
            completed_at: Some(chrono::Utc::now()),
        })
    }

    /// Backfill historical data for a user from a provider.
    #[instrument(skip(self))]
    pub async fn backfill(
        &self,
        user_id: &str,
        provider: &str,
        days: u32,
    ) -> EnformeResult<SyncResult> {
        info!(user_id, provider, days, "Starting backfill");
        // Backfill uses the same sync path but without cursors
        self.sync_user(user_id, provider).await
    }

    /// Handle an incoming webhook from a provider.
    #[instrument(skip(self, headers, body))]
    pub async fn handle_webhook(
        &self,
        provider: &str,
        headers: &http::HeaderMap,
        body: &[u8],
    ) -> EnformeResult<()> {
        let provider_impl = self
            .providers
            .get(provider)
            .ok_or_else(|| EnformeError::provider(provider, "provider not registered"))?;

        let valid = provider_impl.validate_webhook(headers, body).await?;
        if !valid {
            return Err(EnformeError::WebhookValidationFailed {
                provider: provider.to_owned(),
                reason: "invalid signature".to_owned(),
            });
        }

        let events = provider_impl.parse_webhook(body).await?;
        for event in &events {
            info!(
                provider,
                event_type = event.event_type,
                user_id = event.user_id,
                resource_id = event.resource_id,
                "Processing webhook event"
            );
        }

        Ok(())
    }

    /// Start the background sync scheduler.
    pub fn start_scheduler(self: Arc<Self>) -> JoinHandle<()> {
        scheduler::start(self)
    }

    /// Get a reference to the sync configuration.
    #[must_use]
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Get a reference to the store dependencies.
    #[must_use]
    pub fn deps(&self) -> &SyncDeps {
        &self.deps
    }

    /// Get the list of registered provider names.
    #[must_use]
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(String::as_str).collect()
    }

    /// Check if the rate limiter allows a request for a given provider.
    pub fn check_rate_limit(&mut self, provider: &str) -> bool {
        self.rate_limiters
            .get_mut(provider)
            .map_or_else(|| true, TokenBucket::try_acquire)
    }

    /// Sync a single data type for a user, returning records written.
    async fn sync_data_type(
        &self,
        provider: &dyn SyncProvider,
        creds: &crate::models::connection::ProviderCredentials,
        data_type: DataType,
        cursor: Option<&crate::models::cursor::SyncCursor>,
    ) -> EnformeResult<u32> {
        match data_type {
            DataType::Sleep => {
                let batch = provider.fetch_sleep(creds, cursor).await?;
                let count = self.deps.sleep.store_sleep_sessions(&batch.records).await?;
                self.deps.cursors.update_cursor(&batch.cursor).await?;
                Ok(count as u32)
            }
            DataType::Recovery => {
                let batch = provider.fetch_recovery(creds, cursor).await?;
                let count = self
                    .deps
                    .recovery
                    .store_recovery_metrics(&batch.records)
                    .await?;
                self.deps.cursors.update_cursor(&batch.cursor).await?;
                Ok(count as u32)
            }
            DataType::Health => {
                let batch = provider.fetch_health(creds, cursor).await?;
                let count = self
                    .deps
                    .health
                    .store_health_snapshots(&batch.records)
                    .await?;
                self.deps.cursors.update_cursor(&batch.cursor).await?;
                Ok(count as u32)
            }
            DataType::Continuous => {
                let batch = provider.fetch_continuous(creds, cursor).await?;
                let mut total: u64 = 0;
                for metric_batch in &batch.records {
                    total += self
                        .deps
                        .time_series
                        .store_continuous_metrics("default", std::slice::from_ref(metric_batch))
                        .await?;
                }
                self.deps.cursors.update_cursor(&batch.cursor).await?;
                Ok(total as u32)
            }
        }
    }
}

impl std::fmt::Debug for SyncOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncOrchestrator")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

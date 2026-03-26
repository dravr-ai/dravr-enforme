// ABOUTME: Per-provider sync trait defining fetch, webhook, and lifecycle operations
// ABOUTME: Feature-gated implementations for each wearable provider (WHOOP, Garmin, etc.)
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;
use dravr_equilibre::{
    ContinuousMetricBatch, StoredHealthMetrics, StoredRecoveryMetrics, StoredSleepSession,
};
use serde::{Deserialize, Serialize};

use crate::error::EnformeResult;
use crate::models::connection::ProviderCredentials;
use crate::models::cursor::{SyncBatch, SyncCursor};
use crate::models::webhook::{WebhookConfig, WebhookEvent};

/// Data types that a provider can sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    /// Sleep session data.
    Sleep,
    /// Recovery and readiness metrics.
    Recovery,
    /// Health and body measurement data.
    Health,
    /// Continuous time series metrics (heart rate, HRV, etc.).
    Continuous,
}

impl DataType {
    /// Return the string representation used in cursors and APIs.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sleep => "sleep",
            Self::Recovery => "recovery",
            Self::Health => "health",
            Self::Continuous => "continuous",
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Trait for provider-specific sync implementations.
///
/// Each wearable provider (WHOOP, Garmin, Fitbit, Oura) implements this trait
/// behind a feature flag. The `SyncOrchestrator` dispatches to the appropriate
/// provider based on the user's connection.
#[async_trait]
pub trait SyncProvider: Send + Sync {
    /// The provider's unique name (e.g., "whoop", "garmin").
    fn name(&self) -> &'static str;

    /// Data types this provider can supply.
    fn supported_data_types(&self) -> &[DataType];

    /// Fetch sleep sessions from the provider API.
    async fn fetch_sleep(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredSleepSession>>;

    /// Fetch recovery metrics from the provider API.
    async fn fetch_recovery(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredRecoveryMetrics>>;

    /// Fetch health/body metrics from the provider API.
    async fn fetch_health(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredHealthMetrics>>;

    /// Fetch continuous time series data from the provider API.
    async fn fetch_continuous(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>>;

    /// Called when a user connects this provider (register webhooks, etc.).
    async fn on_connected(
        &self,
        creds: &ProviderCredentials,
        webhook_url: &str,
    ) -> EnformeResult<()>;

    /// Called when a user disconnects this provider (unregister webhooks, etc.).
    async fn on_disconnected(&self, creds: &ProviderCredentials) -> EnformeResult<()>;

    /// Return the webhook configuration for this provider, if webhooks are supported.
    fn webhook_config(&self) -> Option<WebhookConfig>;

    /// Validate an incoming webhook request's signature.
    async fn validate_webhook(&self, headers: &http::HeaderMap, body: &[u8])
        -> EnformeResult<bool>;

    /// Parse a webhook body into structured events.
    async fn parse_webhook(&self, body: &[u8]) -> EnformeResult<Vec<WebhookEvent>>;
}

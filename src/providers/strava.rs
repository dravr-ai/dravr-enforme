// ABOUTME: Strava health data provider via dravr-sciotte browser scraping
// ABOUTME: Maps scraped DailySummary (fitness/fatigue/form TSB) to StoredRecoveryMetrics
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use dravr_equilibre::{
    ContinuousMetricBatch, StoredHealthMetrics, StoredRecoveryMetrics, StoredSleepSession,
    SyncStatus,
};
use dravr_sciotte::config::{CacheConfig, ScraperConfig};
use dravr_sciotte::models::{AuthSession, DailySummary, HealthParams};
use dravr_sciotte::provider::ProviderConfig;
use dravr_sciotte::scraper::ChromeScraper;
use dravr_sciotte::types::ActivityScraper;
use dravr_sciotte::CachedScraper;
use http::HeaderMap;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::error::{EnformeError, EnformeResult};
use crate::models::connection::ProviderCredentials;
use crate::models::cursor::{SyncBatch, SyncCursor};
use crate::models::webhook::{WebhookConfig, WebhookEvent};
use crate::traits::sync_provider::{DataType, SyncProvider};

/// Strava health data provider via sciotte browser scraping.
///
/// Strava provides Fitness & Freshness (TSB) data: fitness score (CTL),
/// fatigue score (ATL), form score (TSB), and FTP. No sleep or body composition.
pub struct StravaSciotteProvider {
    scraper: Arc<CachedScraper<ChromeScraper>>,
    session: RwLock<Option<AuthSession>>,
}

impl StravaSciotteProvider {
    /// Create a new Strava provider with default sciotte configuration.
    #[must_use]
    pub fn new() -> Self {
        let scraper_config = ScraperConfig::default();
        let provider_config = ProviderConfig::strava_default();
        let chrome = ChromeScraper::new(scraper_config, provider_config);
        let cached = CachedScraper::new(chrome, &CacheConfig::default());

        Self {
            scraper: Arc::new(cached),
            session: RwLock::new(None),
        }
    }

    /// Restore the browser session from credentials.
    fn restore_session(creds: &ProviderCredentials) -> EnformeResult<AuthSession> {
        serde_json::from_str(&creds.access_token).map_err(|e| EnformeError::ProviderError {
            provider: "strava".to_owned(),
            message: format!("Failed to deserialize sciotte session: {e}"),
        })
    }
}

impl Default for StravaSciotteProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for StravaSciotteProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StravaSciotteProvider").finish()
    }
}

#[async_trait]
impl SyncProvider for StravaSciotteProvider {
    fn name(&self) -> &'static str {
        "strava"
    }

    fn supported_data_types(&self) -> &[DataType] {
        &[DataType::Recovery] // Strava provides TSB (fitness/fatigue/form), mapped as recovery
    }

    async fn fetch_sleep(
        &self,
        _creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredSleepSession>> {
        // Strava does not provide sleep data
        Ok(SyncBatch {
            records: Vec::new(),
            cursor: SyncCursor {
                user_id: String::new(),
                provider: "strava".to_owned(),
                data_type: "sleep".to_owned(),
                value: Utc::now().date_naive().to_string(),
                last_sync_at: Utc::now(),
                status: SyncStatus::Completed,
                records_synced: 0,
                error_message: None,
                retry_count: 0,
                next_retry_at: None,
            },
            has_more: false,
        })
    }

    #[instrument(skip(self, creds), fields(provider = "strava"))]
    async fn fetch_recovery(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredRecoveryMetrics>> {
        let session = Self::restore_session(creds)?;
        *self.session.write().await = Some(session.clone());

        let date = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| Utc::now().date_naive());

        let params = HealthParams { date };
        let summary = self
            .scraper
            .get_daily_summary(&session, &params)
            .await
            .map_err(|e| EnformeError::ProviderError {
                provider: "strava".to_owned(),
                message: format!("Failed to scrape Strava daily summary: {e}"),
            })?;

        let records = if summary.fitness_score.is_some() || summary.form_score.is_some() {
            vec![summary_to_recovery(&summary)]
        } else {
            Vec::new()
        };

        let new_cursor = SyncCursor {
            user_id: String::new(),
            provider: "strava".to_owned(),
            data_type: "recovery".to_owned(),
            value: date.to_string(),
            last_sync_at: Utc::now(),
            status: SyncStatus::Completed,
            records_synced: records.len() as u64,
            error_message: None,
            retry_count: 0,
            next_retry_at: None,
        };

        Ok(SyncBatch {
            records,
            cursor: new_cursor,
            has_more: false,
        })
    }

    async fn fetch_health(
        &self,
        _creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredHealthMetrics>> {
        // Strava does not provide body composition or health metrics
        Ok(SyncBatch {
            records: Vec::new(),
            cursor: SyncCursor {
                user_id: String::new(),
                provider: "strava".to_owned(),
                data_type: "health".to_owned(),
                value: Utc::now().date_naive().to_string(),
                last_sync_at: Utc::now(),
                status: SyncStatus::Completed,
                records_synced: 0,
                error_message: None,
                retry_count: 0,
                next_retry_at: None,
            },
            has_more: false,
        })
    }

    async fn fetch_continuous(
        &self,
        _creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>> {
        Ok(SyncBatch {
            records: Vec::new(),
            cursor: SyncCursor {
                user_id: String::new(),
                provider: "strava".to_owned(),
                data_type: "continuous".to_owned(),
                value: Utc::now().date_naive().to_string(),
                last_sync_at: Utc::now(),
                status: SyncStatus::Completed,
                records_synced: 0,
                error_message: None,
                retry_count: 0,
                next_retry_at: None,
            },
            has_more: false,
        })
    }

    #[instrument(skip(self, _creds), fields(provider = "strava"))]
    async fn on_connected(
        &self,
        _creds: &ProviderCredentials,
        _webhook_url: &str,
    ) -> EnformeResult<()> {
        tracing::info!("Strava connected via sciotte — sync via scheduler, no webhooks");
        Ok(())
    }

    #[instrument(skip(self, _creds), fields(provider = "strava"))]
    async fn on_disconnected(&self, _creds: &ProviderCredentials) -> EnformeResult<()> {
        *self.session.write().await = None;
        tracing::info!("Strava disconnected — session cleared");
        Ok(())
    }

    fn webhook_config(&self) -> Option<WebhookConfig> {
        None
    }

    async fn validate_webhook(&self, _headers: &HeaderMap, _body: &[u8]) -> EnformeResult<bool> {
        Err(EnformeError::ProviderError {
            provider: "strava".to_owned(),
            message: "Strava sciotte provider does not support webhooks".to_owned(),
        })
    }

    async fn parse_webhook(&self, _body: &[u8]) -> EnformeResult<Vec<WebhookEvent>> {
        Err(EnformeError::ProviderError {
            provider: "strava".to_owned(),
            message: "Strava sciotte provider does not support webhooks".to_owned(),
        })
    }
}

/// Map Strava TSB data to recovery metrics.
fn summary_to_recovery(s: &DailySummary) -> StoredRecoveryMetrics {
    // Map form_score (TSB, can be negative i32) to recovery/readiness (Option<u32>)
    // Clamp negative values to 0 since recovery scores are unsigned.
    #[allow(clippy::cast_sign_loss)]
    let form_as_u32 = s.form_score.map(|v| v.max(0) as u32);

    StoredRecoveryMetrics {
        id: format!("strava-recovery-{}", s.date),
        user_id: String::new(),
        data_source_id: String::new(),
        date: s.date,
        // Map Strava TSB: form_score (TSB) as recovery indicator
        recovery_score: form_as_u32,
        readiness_score: form_as_u32,
        hrv_ms: None,
        hrv_rmssd: None,
        resting_heart_rate: None,
        stress_score: s.fatigue_score,
        body_battery: None,
        spo2: None,
        respiratory_rate: None,
        skin_temp_deviation: None,
        source_name: "strava".to_owned(),
        recorded_at: Utc::now(),
    }
}

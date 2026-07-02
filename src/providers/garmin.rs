// ABOUTME: Garmin health data provider via dravr-sciotte browser scraping
// ABOUTME: Maps scraped DailySummary to StoredRecoveryMetrics, StoredHealthMetrics, and StoredSleepSession
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, NaiveDate, Utc};
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
use tokio::sync::{OnceCell, RwLock};
use tracing::instrument;

use crate::error::{EnformeError, EnformeResult};
use crate::models::connection::ProviderCredentials;
use crate::models::cursor::{SyncBatch, SyncCursor};
use crate::models::webhook::{WebhookConfig, WebhookEvent};
use crate::traits::sync_provider::{DataType, SyncProvider};

/// Garmin Connect health data provider via sciotte browser scraping.
///
/// Authenticates using a serialized `AuthSession` (browser cookies) stored
/// in the `ProviderCredentials.access_token` field. Scrapes the daily-summary
/// and sleep pages to extract health metrics.
pub struct GarminSciotteProvider {
    scraper: OnceCell<Arc<CachedScraper<ChromeScraper>>>,
    session: RwLock<Option<AuthSession>>,
}

impl GarminSciotteProvider {
    /// Create a new Garmin provider with default sciotte configuration.
    ///
    /// The scraper is built lazily on first use so construction stays
    /// infallible: the embedded provider config now parses to a `Result`
    /// (sciotte denies `expect`/`unwrap`), and that parse is deferred to
    /// [`Self::scraper`] where the error joins the fallible fetch path.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scraper: OnceCell::new(),
            session: RwLock::new(None),
        }
    }

    /// Lazily build (once) and return the shared sciotte scraper.
    ///
    /// # Errors
    ///
    /// Returns a `ProviderError` if the embedded Garmin scraper config fails to
    /// parse — a compile-time constant validated by sciotte's own tests, so
    /// this is a should-never-happen surfaced rather than panicked.
    async fn scraper(&self) -> EnformeResult<&Arc<CachedScraper<ChromeScraper>>> {
        self.scraper
            .get_or_try_init(|| async {
                let provider_config =
                    ProviderConfig::garmin_default().map_err(|e| EnformeError::ProviderError {
                        provider: "garmin".to_owned(),
                        message: format!("Failed to build sciotte provider config: {e}"),
                    })?;
                let chrome = ChromeScraper::new(ScraperConfig::default(), provider_config);
                Ok(Arc::new(CachedScraper::new(
                    chrome,
                    &CacheConfig::default(),
                )))
            })
            .await
    }

    /// Restore the browser session from credentials, or `None` when the stored
    /// blob isn't a usable sciotte session.
    ///
    /// Garmin health is scraped, so it needs a sciotte browser session. A user
    /// connected only via OAuth has an OAuth token (not a session JSON) in this
    /// slot; deserialization then fails on every sync cycle. Treat that as "not
    /// connected via the scraper" and skip rather than erroring repeatedly — a
    /// genuine scrape/login failure surfaces elsewhere.
    fn restore_session(creds: &ProviderCredentials) -> Option<AuthSession> {
        match serde_json::from_str(&creds.access_token) {
            Ok(session) => Some(session),
            Err(e) => {
                tracing::debug!(
                    provider = "garmin",
                    error = %e,
                    "No usable sciotte session for Garmin health sync — skipping"
                );
                None
            }
        }
    }

    /// An empty, completed sync batch for `data_type` — returned when there is
    /// no usable sciotte session so the orchestrator records a clean skip
    /// instead of a recurring error.
    fn empty_skip_batch<T>(user_id: &str, data_type: &str) -> SyncBatch<T> {
        SyncBatch {
            records: Vec::new(),
            cursor: SyncCursor {
                user_id: user_id.to_owned(),
                provider: "garmin".to_owned(),
                data_type: data_type.to_owned(),
                value: Utc::now().date_naive().to_string(),
                last_sync_at: Utc::now(),
                status: SyncStatus::Completed,
                records_synced: 0,
                error_message: None,
                retry_count: 0,
                next_retry_at: None,
            },
            has_more: false,
        }
    }

    /// Fetch daily summaries for a date range, one day at a time.
    async fn fetch_date_range(
        &self,
        session: &AuthSession,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<DailySummary> {
        let mut summaries = Vec::new();
        let mut date = start;

        let scraper = match self.scraper().await {
            Ok(scraper) => scraper,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to build Garmin scraper — skipping date range");
                return summaries;
            }
        };

        while date <= end {
            let params = HealthParams { date };
            match scraper.get_daily_summary(session, &params).await {
                Ok(summary) => summaries.push(summary),
                Err(e) => {
                    tracing::warn!(date = %date, error = %e, "Failed to scrape Garmin daily summary");
                }
            }
            date += Duration::days(1);
        }

        summaries
    }
}

impl Default for GarminSciotteProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GarminSciotteProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GarminSciotteProvider").finish()
    }
}

#[async_trait]
impl SyncProvider for GarminSciotteProvider {
    fn name(&self) -> &'static str {
        "garmin"
    }

    fn supported_data_types(&self) -> &[DataType] {
        &[DataType::Sleep, DataType::Recovery, DataType::Health]
    }

    #[instrument(skip(self, creds), fields(provider = "garmin"))]
    async fn fetch_sleep(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredSleepSession>> {
        let Some(session) = Self::restore_session(creds) else {
            return Ok(Self::empty_skip_batch(&creds.user_id, "sleep"));
        };
        *self.session.write().await = Some(session.clone());

        let start = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| (Utc::now() - Duration::days(1)).date_naive());
        let end = Utc::now().date_naive();

        let summaries = self.fetch_date_range(&session, start, end).await;
        let records: Vec<StoredSleepSession> = summaries
            .iter()
            .filter(|s| s.sleep_duration_seconds.is_some())
            .map(|s| summary_to_sleep(s, &creds.user_id))
            .collect();

        let new_cursor = SyncCursor {
            user_id: creds.user_id.clone(),
            provider: "garmin".to_owned(),
            data_type: "sleep".to_owned(),
            value: end.to_string(),
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

    #[instrument(skip(self, creds), fields(provider = "garmin"))]
    async fn fetch_recovery(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredRecoveryMetrics>> {
        let Some(session) = Self::restore_session(creds) else {
            return Ok(Self::empty_skip_batch(&creds.user_id, "recovery"));
        };
        *self.session.write().await = Some(session.clone());

        let start = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| (Utc::now() - Duration::days(1)).date_naive());
        let end = Utc::now().date_naive();

        let summaries = self.fetch_date_range(&session, start, end).await;
        let records: Vec<StoredRecoveryMetrics> = summaries
            .iter()
            .map(|s| summary_to_recovery(s, &creds.user_id))
            .collect();

        let new_cursor = SyncCursor {
            user_id: creds.user_id.clone(),
            provider: "garmin".to_owned(),
            data_type: "recovery".to_owned(),
            value: end.to_string(),
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

    #[instrument(skip(self, creds), fields(provider = "garmin"))]
    async fn fetch_health(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredHealthMetrics>> {
        let Some(session) = Self::restore_session(creds) else {
            return Ok(Self::empty_skip_batch(&creds.user_id, "health"));
        };
        *self.session.write().await = Some(session.clone());

        let start = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| (Utc::now() - Duration::days(1)).date_naive());
        let end = Utc::now().date_naive();

        let summaries = self.fetch_date_range(&session, start, end).await;
        let records: Vec<StoredHealthMetrics> = summaries
            .iter()
            .filter(|s| s.vo2_max.is_some() || s.weight_kg.is_some())
            .map(|s| summary_to_health(s, &creds.user_id))
            .collect();

        let new_cursor = SyncCursor {
            user_id: creds.user_id.clone(),
            provider: "garmin".to_owned(),
            data_type: "health".to_owned(),
            value: end.to_string(),
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

    #[instrument(skip(self, creds), fields(provider = "garmin"))]
    async fn fetch_continuous(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>> {
        // Garmin continuous metrics (HR, steps) are in the daily summary
        // but mapped as recovery/health, not time-series points
        Ok(SyncBatch {
            records: Vec::new(),
            cursor: SyncCursor {
                user_id: creds.user_id.clone(),
                provider: "garmin".to_owned(),
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

    #[instrument(skip(self, _creds), fields(provider = "garmin"))]
    async fn on_connected(
        &self,
        _creds: &ProviderCredentials,
        _webhook_url: &str,
    ) -> EnformeResult<()> {
        // Garmin has no webhook registration — poll-only via scheduler
        tracing::info!("Garmin connected via sciotte — sync via scheduler, no webhooks");
        Ok(())
    }

    #[instrument(skip(self, _creds), fields(provider = "garmin"))]
    async fn on_disconnected(&self, _creds: &ProviderCredentials) -> EnformeResult<()> {
        *self.session.write().await = None;
        tracing::info!("Garmin disconnected — session cleared");
        Ok(())
    }

    fn webhook_config(&self) -> Option<WebhookConfig> {
        None // Garmin has no webhook support
    }

    async fn validate_webhook(&self, _headers: &HeaderMap, _body: &[u8]) -> EnformeResult<bool> {
        Err(EnformeError::ProviderError {
            provider: "garmin".to_owned(),
            message: "Garmin does not support webhooks".to_owned(),
        })
    }

    async fn parse_webhook(&self, _body: &[u8]) -> EnformeResult<Vec<WebhookEvent>> {
        Err(EnformeError::ProviderError {
            provider: "garmin".to_owned(),
            message: "Garmin does not support webhooks".to_owned(),
        })
    }
}

// ============================================================================
// DailySummary → stored type conversions
// ============================================================================

fn summary_to_sleep(s: &DailySummary, user_id: &str) -> StoredSleepSession {
    let date_midnight = s
        .date
        .and_hms_opt(0, 0, 0)
        .map_or_else(Utc::now, |dt| dt.and_utc());

    let total_secs = s.sleep_duration_seconds.unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    let total_sleep_secs = total_secs as u32;

    #[allow(clippy::cast_possible_wrap)]
    let total_secs_signed = total_secs as i64;

    StoredSleepSession {
        id: format!("garmin-sleep-{}-{}", user_id, s.date),
        user_id: user_id.to_owned(),
        data_source_id: "garmin-default".to_owned(),
        is_nap: false,
        start_datetime: date_midnight,
        end_datetime: date_midnight + Duration::seconds(total_secs_signed),
        total_sleep_seconds: Some(total_sleep_secs),
        #[allow(clippy::cast_possible_truncation)]
        deep_sleep_seconds: s.sleep_deep_seconds.map(|v| v as u32),
        #[allow(clippy::cast_possible_truncation)]
        light_sleep_seconds: s.sleep_light_seconds.map(|v| v as u32),
        #[allow(clippy::cast_possible_truncation)]
        rem_sleep_seconds: s.sleep_rem_seconds.map(|v| v as u32),
        #[allow(clippy::cast_possible_truncation)]
        awake_seconds: s.sleep_awake_seconds.map(|v| v as u32),
        sleep_efficiency: None,
        avg_heart_rate: None,
        min_heart_rate: None,
        avg_hrv: s.hrv_value.map(f64::from),
        sleep_score: s.sleep_score,
        stages: Vec::new(),
        source_name: "garmin".to_owned(),
    }
}

fn summary_to_recovery(s: &DailySummary, user_id: &str) -> StoredRecoveryMetrics {
    StoredRecoveryMetrics {
        id: format!("garmin-recovery-{}-{}", user_id, s.date),
        user_id: user_id.to_owned(),
        data_source_id: "garmin-default".to_owned(),
        date: s.date,
        recovery_score: s.body_battery,
        readiness_score: s.body_battery,
        hrv_ms: s.hrv_value.map(f64::from),
        hrv_rmssd: None,
        resting_heart_rate: s.resting_heart_rate,
        stress_score: s.stress_level,
        body_battery: s.body_battery,
        spo2: None,
        respiratory_rate: None,
        skin_temp_deviation: None,
        source_name: "garmin".to_owned(),
        recorded_at: Utc::now(),
    }
}

fn summary_to_health(s: &DailySummary, user_id: &str) -> StoredHealthMetrics {
    StoredHealthMetrics {
        id: format!("garmin-health-{}-{}", user_id, s.date),
        user_id: user_id.to_owned(),
        data_source_id: "garmin-default".to_owned(),
        date: s.date,
        weight_kg: s.weight_kg.map(f64::from),
        body_fat_pct: s.body_fat_percent.map(f64::from),
        muscle_mass_kg: None,
        bmi: None,
        bone_mass_kg: None,
        water_pct: None,
        systolic_bp: None,
        diastolic_bp: None,
        blood_glucose: None,
        source_name: "garmin".to_owned(),
        recorded_at: Utc::now(),
    }
}

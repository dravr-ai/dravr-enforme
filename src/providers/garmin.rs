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
use tokio::sync::RwLock;
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
    scraper: Arc<CachedScraper<ChromeScraper>>,
    session: RwLock<Option<AuthSession>>,
}

impl GarminSciotteProvider {
    /// Create a new Garmin provider with default sciotte configuration.
    #[must_use]
    pub fn new() -> Self {
        let scraper_config = ScraperConfig::default();
        let provider_config = ProviderConfig::garmin_default();
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
            provider: "garmin".to_owned(),
            message: format!("Failed to deserialize sciotte session: {e}"),
        })
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

        while date <= end {
            let params = HealthParams { date };
            match self.scraper.get_daily_summary(session, &params).await {
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
        let session = Self::restore_session(creds)?;
        *self.session.write().await = Some(session.clone());

        let start = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| (Utc::now() - Duration::days(1)).date_naive());
        let end = Utc::now().date_naive();

        let summaries = self.fetch_date_range(&session, start, end).await;
        let records: Vec<StoredSleepSession> = summaries
            .iter()
            .filter(|s| s.sleep_duration_seconds.is_some())
            .map(|s| summary_to_sleep(s, &creds.access_token))
            .collect();

        let new_cursor = SyncCursor {
            user_id: String::new(),
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
        let session = Self::restore_session(creds)?;
        *self.session.write().await = Some(session.clone());

        let start = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| (Utc::now() - Duration::days(1)).date_naive());
        let end = Utc::now().date_naive();

        let summaries = self.fetch_date_range(&session, start, end).await;
        let records: Vec<StoredRecoveryMetrics> =
            summaries.iter().map(summary_to_recovery).collect();

        let new_cursor = SyncCursor {
            user_id: String::new(),
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
        let session = Self::restore_session(creds)?;
        *self.session.write().await = Some(session.clone());

        let start = cursor
            .and_then(|c| c.value.parse::<NaiveDate>().ok())
            .unwrap_or_else(|| (Utc::now() - Duration::days(1)).date_naive());
        let end = Utc::now().date_naive();

        let summaries = self.fetch_date_range(&session, start, end).await;
        let records: Vec<StoredHealthMetrics> = summaries
            .iter()
            .filter(|s| s.vo2_max.is_some() || s.weight_kg.is_some())
            .map(summary_to_health)
            .collect();

        let new_cursor = SyncCursor {
            user_id: String::new(),
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

    #[instrument(skip(self, _creds), fields(provider = "garmin"))]
    async fn fetch_continuous(
        &self,
        _creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>> {
        // Garmin continuous metrics (HR, steps) are in the daily summary
        // but mapped as recovery/health, not time-series points
        Ok(SyncBatch {
            records: Vec::new(),
            cursor: SyncCursor {
                user_id: String::new(),
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

fn summary_to_sleep(s: &DailySummary, data_source_id: &str) -> StoredSleepSession {
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
        id: format!("garmin-sleep-{}", s.date),
        user_id: String::new(),
        data_source_id: data_source_id.to_owned(),
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

fn summary_to_recovery(s: &DailySummary) -> StoredRecoveryMetrics {
    StoredRecoveryMetrics {
        id: format!("garmin-recovery-{}", s.date),
        user_id: String::new(),
        data_source_id: String::new(),
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

fn summary_to_health(s: &DailySummary) -> StoredHealthMetrics {
    StoredHealthMetrics {
        id: format!("garmin-health-{}", s.date),
        user_id: String::new(),
        data_source_id: String::new(),
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

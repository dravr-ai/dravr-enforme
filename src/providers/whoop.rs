// ABOUTME: WHOOP API v2 provider implementation for sleep, recovery, and health data
// ABOUTME: Supports paginated fetch, webhook registration, and HMAC-SHA256 validation
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::env;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use dravr_equilibre::{
    ContinuousMetricBatch, StoredHealthMetrics, StoredRecoveryMetrics, StoredSleepSession,
    SyncStatus,
};
use http::HeaderMap;
use serde::Deserialize;
use tracing::instrument;

use crate::error::{EnformeError, EnformeResult};
use crate::models::connection::ProviderCredentials;
use crate::models::cursor::{SyncBatch, SyncCursor};
use crate::models::webhook::{WebhookAlgorithm, WebhookConfig, WebhookEvent};
use crate::traits::sync_provider::{DataType, SyncProvider};
use crate::webhook::validation;

/// WHOOP API base URL (v2). The v1 API is no longer supported by WHOOP.
const WHOOP_API_BASE: &str = "https://api.prod.whoop.com/developer/v2";

/// WHOOP webhook signature header.
const WHOOP_SIGNATURE_HEADER: &str = "x-whoop-signature";

/// Parse the leading `YYYY-MM-DD` date from a WHOOP `created_at` timestamp.
///
/// Returns `None` for strings shorter than 10 bytes, strings whose 10th byte
/// is not a char boundary, or prefixes that are not a valid date. Uses
/// `str::get` rather than slicing so a malformed provider payload yields
/// `None` instead of panicking and unwinding the sync scheduler.
#[must_use]
pub fn parse_whoop_date(created_at: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(created_at.get(..10)?, "%Y-%m-%d").ok()
}

/// WHOOP API v2 provider implementation.
#[derive(Debug)]
pub struct WhoopProvider {
    client: reqwest::Client,
    webhook_secret: Option<String>,
}

impl WhoopProvider {
    /// Create a new WHOOP provider instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            webhook_secret: env::var("WHOOP_WEBHOOK_SECRET").ok(),
        }
    }

    /// Create a WHOOP provider with a custom HTTP client and webhook secret.
    #[must_use]
    pub fn with_config(client: reqwest::Client, webhook_secret: Option<String>) -> Self {
        Self {
            client,
            webhook_secret,
        }
    }

    /// Build an authorized request to the WHOOP API.
    fn authorized_get(&self, path: &str, creds: &ProviderCredentials) -> reqwest::RequestBuilder {
        self.client
            .get(format!("{WHOOP_API_BASE}{path}"))
            .bearer_auth(&creds.access_token)
    }

    /// Parse a WHOOP sleep response into stored sleep sessions.
    fn parse_sleep_response(
        response: &WhoopPaginatedResponse<WhoopSleep>,
        user_id: &str,
        data_source_id: &str,
    ) -> Vec<StoredSleepSession> {
        response
            .records
            .iter()
            .filter_map(|s| {
                let start = s.start.parse::<DateTime<Utc>>().ok()?;
                let end = s.end.parse::<DateTime<Utc>>().ok()?;

                let stage_summary = s.score.as_ref().and_then(|sc| sc.stage_summary.as_ref());

                Some(StoredSleepSession {
                    id: s.id.clone(),
                    user_id: user_id.to_owned(),
                    data_source_id: data_source_id.to_owned(),
                    is_nap: s.nap,
                    start_datetime: start,
                    end_datetime: end,
                    total_sleep_seconds: stage_summary
                        .and_then(|sum| sum.in_bed.map(|ms| (ms / 1000) as u32)),
                    deep_sleep_seconds: stage_summary
                        .and_then(|sum| sum.slow_wave.map(|ms| (ms / 1000) as u32)),
                    light_sleep_seconds: stage_summary
                        .and_then(|sum| sum.light.map(|ms| (ms / 1000) as u32)),
                    rem_sleep_seconds: stage_summary
                        .and_then(|sum| sum.rem.map(|ms| (ms / 1000) as u32)),
                    awake_seconds: stage_summary
                        .and_then(|sum| sum.awake.map(|ms| (ms / 1000) as u32)),
                    sleep_efficiency: s
                        .score
                        .as_ref()
                        .and_then(|sc| sc.sleep_efficiency_percentage),
                    avg_heart_rate: None,
                    min_heart_rate: None,
                    avg_hrv: None,
                    sleep_score: s
                        .score
                        .as_ref()
                        .and_then(|sc| sc.sleep_performance_percentage.map(|v| v as u32)),
                    stages: Vec::new(),
                    source_name: "whoop".to_owned(),
                })
            })
            .collect()
    }

    /// Parse a WHOOP recovery response into stored recovery metrics.
    fn parse_recovery_response(
        response: &WhoopPaginatedResponse<WhoopRecovery>,
        user_id: &str,
        data_source_id: &str,
    ) -> Vec<StoredRecoveryMetrics> {
        response
            .records
            .iter()
            .filter_map(|r| {
                let date = parse_whoop_date(&r.created_at)?;
                let recorded_at = r.created_at.parse::<DateTime<Utc>>().ok()?;

                Some(StoredRecoveryMetrics {
                    id: r.cycle_id.to_string(),
                    user_id: user_id.to_owned(),
                    data_source_id: data_source_id.to_owned(),
                    date,
                    recovery_score: r
                        .score
                        .as_ref()
                        .and_then(|s| s.recovery_score.map(|v| v as u32)),
                    readiness_score: None,
                    hrv_ms: r.score.as_ref().and_then(|s| s.hrv_rmssd_milli),
                    hrv_rmssd: r.score.as_ref().and_then(|s| s.hrv_rmssd_milli),
                    resting_heart_rate: r
                        .score
                        .as_ref()
                        .and_then(|s| s.resting_heart_rate.map(|v| v as u32)),
                    stress_score: None,
                    body_battery: None,
                    spo2: r.score.as_ref().and_then(|s| s.spo2_percentage),
                    respiratory_rate: None,
                    skin_temp_deviation: r.score.as_ref().and_then(|s| s.skin_temp_celsius),
                    source_name: "whoop".to_owned(),
                    recorded_at,
                })
            })
            .collect()
    }
}

impl Default for WhoopProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SyncProvider for WhoopProvider {
    fn name(&self) -> &'static str {
        "whoop"
    }

    fn supported_data_types(&self) -> &[DataType] {
        &[DataType::Sleep, DataType::Recovery, DataType::Health]
    }

    #[instrument(skip(self, creds, cursor), fields(provider = "whoop"))]
    async fn fetch_sleep(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredSleepSession>> {
        let mut request = self.authorized_get("/activity/sleep", creds);

        if let Some(c) = cursor {
            if !c.value.is_empty() {
                request = request.query(&[("nextToken", &c.value)]);
            }
        }

        let response: WhoopPaginatedResponse<WhoopSleep> = request
            .send()
            .await
            .map_err(|e| EnformeError::provider("whoop", e.to_string()))?
            .json()
            .await
            .map_err(|e| EnformeError::serialization(e.to_string()))?;

        let sessions = Self::parse_sleep_response(&response, &creds.user_id, "whoop-default");

        let mut new_cursor = SyncCursor::new(&creds.user_id, "whoop", DataType::Sleep.as_str());
        let has_more = response.next_token.is_some();
        if let Some(token) = &response.next_token {
            new_cursor.value.clone_from(token);
        }
        new_cursor.status = SyncStatus::Completed;
        new_cursor.records_synced = sessions.len() as u64;

        Ok(SyncBatch {
            records: sessions,
            cursor: new_cursor,
            has_more,
        })
    }

    #[instrument(skip(self, creds, cursor), fields(provider = "whoop"))]
    async fn fetch_recovery(
        &self,
        creds: &ProviderCredentials,
        cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredRecoveryMetrics>> {
        let mut request = self.authorized_get("/recovery", creds);

        if let Some(c) = cursor {
            if !c.value.is_empty() {
                request = request.query(&[("nextToken", &c.value)]);
            }
        }

        let response: WhoopPaginatedResponse<WhoopRecovery> = request
            .send()
            .await
            .map_err(|e| EnformeError::provider("whoop", e.to_string()))?
            .json()
            .await
            .map_err(|e| EnformeError::serialization(e.to_string()))?;

        let metrics = Self::parse_recovery_response(&response, &creds.user_id, "whoop-default");

        let mut new_cursor = SyncCursor::new(&creds.user_id, "whoop", DataType::Recovery.as_str());
        let has_more = response.next_token.is_some();
        if let Some(token) = &response.next_token {
            new_cursor.value.clone_from(token);
        }
        new_cursor.status = SyncStatus::Completed;
        new_cursor.records_synced = metrics.len() as u64;

        Ok(SyncBatch {
            records: metrics,
            cursor: new_cursor,
            has_more,
        })
    }

    #[instrument(skip(self, creds, _cursor), fields(provider = "whoop"))]
    async fn fetch_health(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<StoredHealthMetrics>> {
        let request = self.authorized_get("/user/measurement/body", creds);

        let response: WhoopBodyMeasurement = request
            .send()
            .await
            .map_err(|e| EnformeError::provider("whoop", e.to_string()))?
            .json()
            .await
            .map_err(|e| EnformeError::serialization(e.to_string()))?;

        let metrics = vec![StoredHealthMetrics {
            id: format!("whoop-body-{}", creds.user_id),
            user_id: creds.user_id.clone(),
            data_source_id: "whoop-default".to_owned(),
            date: Utc::now().date_naive(),
            weight_kg: response.weight_kilogram,
            body_fat_pct: None,
            muscle_mass_kg: None,
            bmi: None,
            bone_mass_kg: None,
            water_pct: None,
            systolic_bp: None,
            diastolic_bp: None,
            blood_glucose: None,
            source_name: "whoop".to_owned(),
            recorded_at: Utc::now(),
        }];

        let mut new_cursor = SyncCursor::new(&creds.user_id, "whoop", DataType::Health.as_str());
        new_cursor.status = SyncStatus::Completed;
        new_cursor.records_synced = metrics.len() as u64;

        Ok(SyncBatch {
            records: metrics,
            cursor: new_cursor,
            has_more: false,
        })
    }

    #[instrument(skip(self, creds, _cursor), fields(provider = "whoop"))]
    async fn fetch_continuous(
        &self,
        creds: &ProviderCredentials,
        _cursor: Option<&SyncCursor>,
    ) -> EnformeResult<SyncBatch<ContinuousMetricBatch>> {
        // WHOOP does not expose continuous time series via public API
        let new_cursor = SyncCursor::new(&creds.user_id, "whoop", DataType::Continuous.as_str());
        Ok(SyncBatch::empty(new_cursor))
    }

    #[instrument(skip(self, creds), fields(provider = "whoop"))]
    async fn on_connected(
        &self,
        creds: &ProviderCredentials,
        webhook_url: &str,
    ) -> EnformeResult<()> {
        let body = serde_json::json!({
            "url": webhook_url,
        });

        self.client
            .post(format!("{WHOOP_API_BASE}/webhook"))
            .bearer_auth(&creds.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| EnformeError::provider("whoop", e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self, creds), fields(provider = "whoop"))]
    async fn on_disconnected(&self, creds: &ProviderCredentials) -> EnformeResult<()> {
        // Attempt to unregister webhook; best-effort
        let _ = self
            .client
            .delete(format!("{WHOOP_API_BASE}/webhook"))
            .bearer_auth(&creds.access_token)
            .send()
            .await;

        Ok(())
    }

    fn webhook_config(&self) -> Option<WebhookConfig> {
        Some(WebhookConfig {
            signature_header: WHOOP_SIGNATURE_HEADER,
            algorithm: WebhookAlgorithm::HmacSha256,
            needs_verification: true,
        })
    }

    async fn validate_webhook(&self, headers: &HeaderMap, body: &[u8]) -> EnformeResult<bool> {
        let secret = self
            .webhook_secret
            .as_deref()
            .ok_or_else(|| EnformeError::config("WHOOP_WEBHOOK_SECRET not configured"))?;

        let signature_header = headers
            .get(WHOOP_SIGNATURE_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| EnformeError::WebhookValidationFailed {
                provider: "whoop".to_owned(),
                reason: format!("missing {WHOOP_SIGNATURE_HEADER} header"),
            })?;

        let signature = validation::extract_signature(signature_header);
        validation::validate_hmac_sha256(secret.as_bytes(), body, signature)
    }

    async fn parse_webhook(&self, body: &[u8]) -> EnformeResult<Vec<WebhookEvent>> {
        let payload: WhoopWebhookPayload =
            serde_json::from_slice(body).map_err(|e| EnformeError::serialization(e.to_string()))?;

        Ok(vec![WebhookEvent::new(
            "whoop",
            &payload.event_type,
            payload.user_id.to_string(),
            payload.id.clone(),
        )])
    }
}

// ============================================================================
// WHOOP API response types (private)
// ============================================================================

#[derive(Debug, Deserialize)]
struct WhoopPaginatedResponse<T> {
    records: Vec<T>,
    next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WhoopSleep {
    // v2 sleep ids are UUID strings, not the v1 integer ids.
    id: String,
    start: String,
    end: String,
    nap: bool,
    score: Option<WhoopSleepScore>,
}

#[derive(Debug, Deserialize)]
struct WhoopSleepScore {
    // v2 nests per-stage durations under stage_summary with total_-prefixed names.
    stage_summary: Option<WhoopSleepStageSummary>,
    sleep_efficiency_percentage: Option<f64>,
    sleep_performance_percentage: Option<f64>,
}

/// Per-stage sleep durations from a v2 sleep score. All values are totals in
/// milliseconds; the wire names carry the `total_*_time_milli` form.
#[derive(Debug, Deserialize)]
struct WhoopSleepStageSummary {
    #[serde(rename = "total_in_bed_time_milli")]
    in_bed: Option<i64>,
    #[serde(rename = "total_slow_wave_sleep_time_milli")]
    slow_wave: Option<i64>,
    #[serde(rename = "total_light_sleep_time_milli")]
    light: Option<i64>,
    #[serde(rename = "total_rem_sleep_time_milli")]
    rem: Option<i64>,
    #[serde(rename = "total_awake_time_milli")]
    awake: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct WhoopRecovery {
    cycle_id: u64,
    created_at: String,
    score: Option<WhoopRecoveryScore>,
}

#[derive(Debug, Deserialize)]
struct WhoopRecoveryScore {
    recovery_score: Option<f64>,
    resting_heart_rate: Option<f64>,
    hrv_rmssd_milli: Option<f64>,
    spo2_percentage: Option<f64>,
    skin_temp_celsius: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct WhoopBodyMeasurement {
    weight_kilogram: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct WhoopWebhookPayload {
    #[serde(rename = "type")]
    event_type: String,
    user_id: u64,
    // v2 webhook ids are strings: activity (sleep/workout) events and recovery
    // events carry the UUID of the associated sleep.
    id: String,
}

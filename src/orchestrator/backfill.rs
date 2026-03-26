// ABOUTME: Historical data backfill triggered on provider connect
// ABOUTME: Fetches past data using pagination and cursor management
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use tracing::{info, instrument, warn};

use crate::error::EnformeResult;
use crate::models::connection::ProviderCredentials;
use crate::traits::sync_provider::{DataType, SyncProvider};
use crate::traits::SyncDeps;

/// Backfill historical data for a user from a provider.
///
/// Fetches all available pages for each supported data type until
/// no more data is available.
#[instrument(skip(deps, provider, creds))]
pub async fn backfill_user(
    deps: &SyncDeps,
    provider: &dyn SyncProvider,
    creds: &ProviderCredentials,
    user_id: &str,
    max_pages: u32,
) -> EnformeResult<BackfillResult> {
    let mut result = BackfillResult::default();

    for data_type in provider.supported_data_types() {
        match backfill_data_type(deps, provider, creds, user_id, *data_type, max_pages).await {
            Ok(count) => {
                info!(
                    user_id,
                    provider = provider.name(),
                    data_type = data_type.as_str(),
                    records = count,
                    "Backfill completed for data type"
                );
                result.records_by_type.push((*data_type, count));
                result.total_records += count;
            }
            Err(e) => {
                warn!(
                    user_id,
                    provider = provider.name(),
                    data_type = data_type.as_str(),
                    error = %e,
                    "Backfill failed for data type"
                );
                result.errors += 1;
            }
        }
    }

    Ok(result)
}

/// Backfill a single data type with pagination.
async fn backfill_data_type(
    deps: &SyncDeps,
    provider: &dyn SyncProvider,
    creds: &ProviderCredentials,
    user_id: &str,
    data_type: DataType,
    max_pages: u32,
) -> EnformeResult<u64> {
    let mut total: u64 = 0;
    let mut cursor = None;
    let mut pages: u32 = 0;

    loop {
        if pages >= max_pages {
            info!(
                user_id,
                provider = provider.name(),
                data_type = data_type.as_str(),
                pages,
                "Reached max pages for backfill"
            );
            break;
        }

        match data_type {
            DataType::Sleep => {
                let batch = provider.fetch_sleep(creds, cursor.as_ref()).await?;
                let count = deps.sleep.store_sleep_sessions(&batch.records).await?;
                total += count;
                deps.cursors.update_cursor(&batch.cursor).await?;
                if !batch.has_more {
                    break;
                }
                cursor = Some(batch.cursor);
            }
            DataType::Recovery => {
                let batch = provider.fetch_recovery(creds, cursor.as_ref()).await?;
                let count = deps.recovery.store_recovery_metrics(&batch.records).await?;
                total += count;
                deps.cursors.update_cursor(&batch.cursor).await?;
                if !batch.has_more {
                    break;
                }
                cursor = Some(batch.cursor);
            }
            DataType::Health => {
                let batch = provider.fetch_health(creds, cursor.as_ref()).await?;
                let count = deps.health.store_health_snapshots(&batch.records).await?;
                total += count;
                deps.cursors.update_cursor(&batch.cursor).await?;
                if !batch.has_more {
                    break;
                }
                cursor = Some(batch.cursor);
            }
            DataType::Continuous => {
                let batch = provider.fetch_continuous(creds, cursor.as_ref()).await?;
                for metric_batch in &batch.records {
                    total += deps
                        .time_series
                        .store_continuous_metrics("default", std::slice::from_ref(metric_batch))
                        .await?;
                }
                deps.cursors.update_cursor(&batch.cursor).await?;
                if !batch.has_more {
                    break;
                }
                cursor = Some(batch.cursor);
            }
        }

        pages += 1;
    }

    Ok(total)
}

/// Result of a backfill operation.
#[derive(Debug, Default)]
pub struct BackfillResult {
    /// Total records synced across all data types.
    pub total_records: u64,
    /// Records synced per data type.
    pub records_by_type: Vec<(DataType, u64)>,
    /// Number of data types that failed.
    pub errors: u32,
}

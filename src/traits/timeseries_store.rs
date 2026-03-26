// ABOUTME: Write-path trait for storing continuous time series metric batches
// ABOUTME: Accepts ContinuousMetricBatch from dravr-equilibre
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;
use dravr_equilibre::ContinuousMetricBatch;

use crate::error::EnformeResult;

/// Write-path trait for time series point persistence.
#[async_trait]
pub trait TimeSeriesPointStore: Send + Sync {
    /// Store continuous metric batches for a data source, returning the number of points written.
    async fn store_continuous_metrics(
        &self,
        source_id: &str,
        batches: &[ContinuousMetricBatch],
    ) -> EnformeResult<u64>;
}

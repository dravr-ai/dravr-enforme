// ABOUTME: Write-path trait for upserting data source records
// ABOUTME: Accepts DataSource from dravr-equilibre
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;
use dravr_equilibre::DataSource;

use crate::error::EnformeResult;

/// Write-path trait for data source persistence.
#[async_trait]
pub trait DataSourceStore: Send + Sync {
    /// Upsert a data source record, returning the source ID.
    async fn upsert_data_source(&self, source: &DataSource) -> EnformeResult<String>;
}

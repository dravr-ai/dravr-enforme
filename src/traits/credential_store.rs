// ABOUTME: Write-path trait for provider credential management
// ABOUTME: Handles OAuth credential retrieval and refresh
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;

use crate::error::EnformeResult;
use crate::models::connection::ProviderCredentials;

/// Write-path trait for provider credential persistence.
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Get the current credentials for a user and provider.
    async fn get_credentials(
        &self,
        user_id: &str,
        provider: &str,
    ) -> EnformeResult<Option<ProviderCredentials>>;

    /// Refresh and return updated credentials for a user and provider.
    async fn refresh_credentials(
        &self,
        user_id: &str,
        provider: &str,
    ) -> EnformeResult<ProviderCredentials>;
}

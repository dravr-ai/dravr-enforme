// ABOUTME: Callback hook for provider connect and disconnect events
// ABOUTME: Triggers webhook registration and backfill on connect
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;

use crate::error::EnformeResult;
use crate::models::connection::ProviderCredentials;

/// Hook called when a user connects or disconnects a wearable provider.
///
/// Implementations should register webhooks and trigger initial backfill on connect,
/// and clean up webhooks on disconnect.
#[async_trait]
pub trait OnConnectHook: Send + Sync {
    /// Called when a user successfully connects a provider via OAuth.
    async fn on_provider_connected(
        &self,
        user_id: &str,
        provider: &str,
        credentials: &ProviderCredentials,
    ) -> EnformeResult<()>;

    /// Called when a user disconnects a provider.
    async fn on_provider_disconnected(&self, user_id: &str, provider: &str) -> EnformeResult<()>;
}

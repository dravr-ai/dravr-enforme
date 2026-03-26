// ABOUTME: Write-path trait for listing connected users per provider
// ABOUTME: Used by the scheduler to determine which users to sync
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use async_trait::async_trait;

use crate::error::EnformeResult;
use crate::models::connection::ConnectedUser;

/// Write-path trait for user connection persistence.
#[async_trait]
pub trait UserConnectionStore: Send + Sync {
    /// List all users connected to a given provider.
    async fn list_connected_users(&self, provider: &str) -> EnformeResult<Vec<ConnectedUser>>;
}

// ABOUTME: Connected user and provider credential models
// ABOUTME: OAuth credential lifecycle management for provider API access
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A user who has connected a wearable provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedUser {
    /// Unique user identifier.
    pub user_id: String,
    /// Provider name (e.g., "whoop", "garmin").
    pub provider: String,
    /// When the connection was established.
    pub connected_at: DateTime<Utc>,
    /// Whether the connection is currently active.
    pub is_active: bool,
}

impl ConnectedUser {
    /// Create a new connected user record.
    #[must_use]
    pub fn new(user_id: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            provider: provider.into(),
            connected_at: Utc::now(),
            is_active: true,
        }
    }
}

/// OAuth credentials for accessing a provider's API on behalf of a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentials {
    /// OAuth access token.
    pub access_token: String,
    /// OAuth refresh token for obtaining new access tokens.
    pub refresh_token: Option<String>,
    /// When the access token expires.
    pub expires_at: Option<DateTime<Utc>>,
    /// OAuth scopes granted.
    pub scopes: Vec<String>,
    /// The user these credentials belong to.
    pub user_id: String,
    /// The provider these credentials are for.
    pub provider: String,
}

impl ProviderCredentials {
    /// Check if the access token has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|expires| expires <= Utc::now())
    }

    /// Check if the credentials can be refreshed.
    #[must_use]
    pub fn can_refresh(&self) -> bool {
        self.refresh_token.is_some()
    }
}

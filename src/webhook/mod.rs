// ABOUTME: Webhook handler for processing incoming provider notifications
// ABOUTME: Routes webhooks to the correct provider for validation and parsing
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

/// HMAC-SHA256 signature validation utilities
pub mod validation;

use std::collections::HashMap;
use std::fmt;

use http::HeaderMap;
use tracing::instrument;

use crate::error::{EnformeError, EnformeResult};
use crate::models::webhook::WebhookEvent;
use crate::traits::sync_provider::SyncProvider;

/// Handles incoming webhook requests from wearable providers.
///
/// Routes each request to the correct provider for signature validation
/// and event parsing.
pub struct WebhookHandler {
    providers: HashMap<String, Box<dyn SyncProvider>>,
}

impl WebhookHandler {
    /// Create a new webhook handler with the given providers.
    #[must_use]
    pub fn new(providers: HashMap<String, Box<dyn SyncProvider>>) -> Self {
        Self { providers }
    }

    /// Validate and parse an incoming webhook request.
    ///
    /// Returns the parsed events if the signature is valid.
    #[instrument(skip(self, headers, body))]
    pub async fn handle(
        &self,
        provider_name: &str,
        headers: &HeaderMap,
        body: &[u8],
    ) -> EnformeResult<Vec<WebhookEvent>> {
        let provider = self.providers.get(provider_name).ok_or_else(|| {
            EnformeError::WebhookValidationFailed {
                provider: provider_name.to_owned(),
                reason: "unknown provider".to_owned(),
            }
        })?;

        let valid = provider.validate_webhook(headers, body).await?;
        if !valid {
            return Err(EnformeError::WebhookValidationFailed {
                provider: provider_name.to_owned(),
                reason: "invalid signature".to_owned(),
            });
        }

        provider.parse_webhook(body).await
    }

    /// Check if a provider is registered for webhook handling.
    #[must_use]
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }
}

impl fmt::Debug for WebhookHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookHandler")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .finish()
    }
}

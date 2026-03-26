// ABOUTME: Webhook configuration and event models for provider notifications
// ABOUTME: Supports HMAC-SHA256 signature validation per Standard Webhooks spec
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use serde::{Deserialize, Serialize};

/// Configuration for how a provider sends and validates webhooks.
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// HTTP header name containing the signature.
    pub signature_header: &'static str,
    /// Algorithm used for signature verification.
    pub algorithm: WebhookAlgorithm,
    /// Whether the webhook requires signature verification.
    pub needs_verification: bool,
}

/// Signature algorithm used by a webhook provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookAlgorithm {
    /// HMAC-SHA256 (used by WHOOP, most providers).
    HmacSha256,
    /// HMAC-SHA1 (legacy providers).
    HmacSha1,
    /// Simple verification token comparison.
    VerificationToken,
}

/// A parsed webhook event from a provider notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// The provider that sent the webhook (e.g., "whoop").
    pub provider: String,
    /// The type of event (e.g., "sleep.updated", "recovery.deleted").
    pub event_type: String,
    /// The user ID associated with the event.
    pub user_id: String,
    /// Provider-specific resource ID to fetch updated data.
    pub resource_id: String,
}

impl WebhookEvent {
    /// Create a new webhook event.
    #[must_use]
    pub fn new(
        provider: impl Into<String>,
        event_type: impl Into<String>,
        user_id: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            event_type: event_type.into(),
            user_id: user_id.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Check if this is a deletion event.
    #[must_use]
    pub fn is_deletion(&self) -> bool {
        self.event_type.ends_with(".deleted")
    }

    /// Check if this is an update event.
    #[must_use]
    pub fn is_update(&self) -> bool {
        self.event_type.ends_with(".updated")
    }

    /// Extract the data type from the event type (e.g., "sleep" from "sleep.updated").
    #[must_use]
    pub fn data_type(&self) -> Option<&str> {
        self.event_type.split('.').next()
    }
}

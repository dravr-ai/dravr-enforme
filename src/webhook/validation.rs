// ABOUTME: HMAC-SHA256 webhook signature validation following Standard Webhooks spec
// ABOUTME: Uses ring::hmac::verify for constant-time comparison to prevent timing attacks
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use crate::error::{EnformeError, EnformeResult};

/// Validate an HMAC-SHA256 signature against the expected value.
///
/// Uses `ring::hmac::verify()` which performs constant-time comparison internally,
/// preventing timing attacks.
///
/// # Arguments
/// * `secret` - The shared secret key for HMAC computation
/// * `body` - The raw request body to verify
/// * `signature` - The hex-encoded signature from the webhook header (may include prefix)
#[cfg(feature = "webhooks")]
pub fn validate_hmac_sha256(secret: &[u8], body: &[u8], signature: &str) -> EnformeResult<bool> {
    use ring::hmac;

    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    let sig_bytes = hex::decode(extract_signature(signature)).map_err(|e| {
        EnformeError::WebhookValidationFailed {
            provider: "unknown".to_owned(),
            reason: format!("invalid hex signature: {e}"),
        }
    })?;

    hmac::verify(&key, body, &sig_bytes)
        .map(|()| true)
        .map_err(|_| EnformeError::WebhookValidationFailed {
            provider: "unknown".to_owned(),
            reason: "HMAC signature mismatch".to_owned(),
        })
}

/// Validate an HMAC-SHA256 signature (stub when webhooks feature is disabled).
#[cfg(not(feature = "webhooks"))]
pub fn validate_hmac_sha256(_secret: &[u8], _body: &[u8], _signature: &str) -> EnformeResult<bool> {
    Err(EnformeError::ConfigurationError {
        message: "webhook validation requires the 'webhooks' feature".to_owned(),
    })
}

/// Extract signature from a header value, stripping common prefixes.
///
/// Handles formats like:
/// - `sha256=abcdef1234`
/// - `v1=abcdef1234`
/// - `abcdef1234` (raw hex)
pub fn extract_signature(header_value: &str) -> &str {
    header_value
        .strip_prefix("sha256=")
        .or_else(|| header_value.strip_prefix("v1="))
        .unwrap_or(header_value)
}

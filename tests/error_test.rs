// ABOUTME: Tests for EnformeError variants and display formatting
// ABOUTME: Verifies all error variants implement Display and Error traits
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]

use std::error::Error;

use dravr_enforme::EnformeError;

#[test]
fn provider_error_displays_correctly() {
    let err = EnformeError::provider("whoop", "connection timeout");
    assert_eq!(
        err.to_string(),
        "provider 'whoop' error: connection timeout"
    );
}

#[test]
fn webhook_validation_error_displays_correctly() {
    let err = EnformeError::WebhookValidationFailed {
        provider: "whoop".to_owned(),
        reason: "invalid signature".to_owned(),
    };
    assert_eq!(
        err.to_string(),
        "webhook validation failed for 'whoop': invalid signature"
    );
}

#[test]
fn cursor_not_found_error_displays_correctly() {
    let err = EnformeError::CursorNotFound {
        user_id: "user-1".to_owned(),
        provider: "whoop".to_owned(),
        data_type: "sleep".to_owned(),
    };
    assert_eq!(
        err.to_string(),
        "sync cursor not found for user 'user-1', provider 'whoop', type 'sleep'"
    );
}

#[test]
fn rate_limited_error_displays_correctly() {
    let err = EnformeError::RateLimited {
        provider: "garmin".to_owned(),
        retry_after_secs: 60,
    };
    assert_eq!(
        err.to_string(),
        "rate limited by provider 'garmin', retry after 60s"
    );
}

#[test]
fn credentials_expired_error_displays_correctly() {
    let err = EnformeError::CredentialsExpired {
        user_id: "user-2".to_owned(),
        provider: "fitbit".to_owned(),
    };
    assert_eq!(
        err.to_string(),
        "credentials expired for user 'user-2', provider 'fitbit'"
    );
}

#[test]
fn configuration_error_displays_correctly() {
    let err = EnformeError::config("missing WHOOP_API_KEY");
    assert_eq!(
        err.to_string(),
        "configuration error: missing WHOOP_API_KEY"
    );
}

#[test]
fn serialization_error_displays_correctly() {
    let err = EnformeError::serialization("invalid JSON");
    assert_eq!(err.to_string(), "serialization error: invalid JSON");
}

#[test]
fn network_error_displays_correctly() {
    let err = EnformeError::NetworkError {
        message: "DNS resolution failed".to_owned(),
    };
    assert_eq!(err.to_string(), "network error: DNS resolution failed");
}

#[test]
fn store_error_displays_correctly() {
    let err = EnformeError::store("database connection lost");
    assert_eq!(err.to_string(), "store error: database connection lost");
}

#[test]
fn enforme_error_implements_std_error() {
    let err: Box<dyn Error> = Box::new(EnformeError::provider("test", "msg"));
    assert!(!err.to_string().is_empty());
}

#[test]
fn enforme_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EnformeError>();
}

#[test]
fn enforme_error_is_debug() {
    let err = EnformeError::provider("whoop", "test");
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("ProviderError"));
}

#[test]
fn from_http_status_401_maps_to_credentials_expired() {
    let err = EnformeError::from_http_status(401, "whoop", "user-1", None, "unauthorized");
    assert!(matches!(
        err,
        EnformeError::CredentialsExpired { user_id, provider }
            if user_id == "user-1" && provider == "whoop"
    ));
}

#[test]
fn from_http_status_403_maps_to_credentials_expired() {
    let err = EnformeError::from_http_status(403, "whoop", "user-1", None, "forbidden");
    assert!(matches!(err, EnformeError::CredentialsExpired { .. }));
}

#[test]
fn from_http_status_429_maps_to_rate_limited_with_retry_after() {
    let err = EnformeError::from_http_status(429, "whoop", "user-1", Some(120), "slow down");
    assert!(matches!(
        err,
        EnformeError::RateLimited { provider, retry_after_secs }
            if provider == "whoop" && retry_after_secs == 120
    ));
}

#[test]
fn from_http_status_429_defaults_retry_after_to_60() {
    let err = EnformeError::from_http_status(429, "whoop", "user-1", None, "");
    assert!(matches!(
        err,
        EnformeError::RateLimited {
            retry_after_secs: 60,
            ..
        }
    ));
}

#[test]
fn from_http_status_500_maps_to_provider_error_with_status_and_body() {
    let err = EnformeError::from_http_status(500, "whoop", "user-1", None, "internal error");
    let text = err.to_string();
    assert!(text.contains("HTTP 500"));
    assert!(text.contains("internal error"));
    assert!(text.contains("whoop"));
}

#[test]
fn from_http_status_truncates_long_bodies_to_200_chars() {
    let body = "x".repeat(5000);
    let err = EnformeError::from_http_status(502, "whoop", "user-1", None, &body);
    let text = err.to_string();
    assert!(text.contains(&"x".repeat(200)));
    assert!(!text.contains(&"x".repeat(201)));
}

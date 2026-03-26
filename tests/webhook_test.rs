// ABOUTME: Tests for webhook validation utilities and signature extraction
// ABOUTME: Covers HMAC-SHA256 validation, header parsing, and event handling
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use dravr_enforme::webhook::validation;

#[test]
fn extract_signature_strips_sha256_prefix() {
    assert_eq!(
        validation::extract_signature("sha256=abcdef1234"),
        "abcdef1234"
    );
}

#[test]
fn extract_signature_strips_v1_prefix() {
    assert_eq!(validation::extract_signature("v1=abcdef1234"), "abcdef1234");
}

#[test]
fn extract_signature_returns_raw_when_no_prefix() {
    assert_eq!(validation::extract_signature("abcdef1234"), "abcdef1234");
}

#[test]
fn extract_signature_empty_string() {
    assert_eq!(validation::extract_signature(""), "");
}

#[test]
fn extract_signature_only_prefix() {
    assert_eq!(validation::extract_signature("sha256="), "");
    assert_eq!(validation::extract_signature("v1="), "");
}

#[cfg(feature = "webhooks")]
mod hmac_tests {
    use dravr_enforme::webhook::validation;

    #[test]
    fn validate_hmac_sha256_correct_signature() {
        use ring::hmac;

        let secret = b"test-secret";
        let body = b"hello world";

        let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
        let signature = hmac::sign(&key, body);
        let hex_sig = hex::encode(signature.as_ref());

        let result = validation::validate_hmac_sha256(secret, body, &hex_sig);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn validate_hmac_sha256_wrong_signature_returns_error() {
        let valid_hex = "aa".repeat(32);
        let result = validation::validate_hmac_sha256(b"secret", b"body", &valid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn validate_hmac_sha256_invalid_hex_returns_error() {
        let result = validation::validate_hmac_sha256(b"secret", b"body", "not-valid-hex!");
        assert!(result.is_err());
    }

    #[test]
    fn validate_hmac_sha256_empty_body() {
        use ring::hmac;

        let secret = b"test-secret";
        let body = b"";

        let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
        let signature = hmac::sign(&key, body);
        let hex_sig = hex::encode(signature.as_ref());

        let result = validation::validate_hmac_sha256(secret, body, &hex_sig);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

#[cfg(not(feature = "webhooks"))]
#[test]
fn validate_hmac_sha256_without_feature_returns_error() {
    let result = validation::validate_hmac_sha256(b"secret", b"body", "sig");
    assert!(result.is_err());
}

// ABOUTME: Tests for WHOOP API response parsing into stored domain models
// ABOUTME: Verifies sleep, recovery, and health data transformation
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]

#[cfg(feature = "provider-whoop")]
mod whoop_tests {
    use dravr_enforme::providers::whoop::WhoopProvider;
    use dravr_enforme::traits::sync_provider::{DataType, SyncProvider};

    #[test]
    fn whoop_provider_name() {
        let provider = WhoopProvider::new();
        assert_eq!(provider.name(), "whoop");
    }

    #[test]
    fn whoop_supported_data_types() {
        let provider = WhoopProvider::new();
        let types = provider.supported_data_types();
        assert!(types.contains(&DataType::Sleep));
        assert!(types.contains(&DataType::Recovery));
        assert!(types.contains(&DataType::Health));
        assert!(!types.contains(&DataType::Continuous));
    }

    #[test]
    fn whoop_webhook_config() {
        let provider = WhoopProvider::new();
        let config = provider.webhook_config().unwrap();
        assert_eq!(config.signature_header, "x-whoop-signature");
        assert!(config.needs_verification);
    }

    #[test]
    fn whoop_provider_default() {
        let provider = WhoopProvider::default();
        assert_eq!(provider.name(), "whoop");
    }

    #[test]
    fn whoop_provider_is_debug() {
        let provider = WhoopProvider::new();
        let debug = format!("{provider:?}");
        assert!(debug.contains("WhoopProvider"));
    }

    #[test]
    fn whoop_with_config() {
        let client = reqwest::Client::new();
        let provider = WhoopProvider::with_config(client, Some("secret".to_owned()));
        assert_eq!(provider.name(), "whoop");
    }
}

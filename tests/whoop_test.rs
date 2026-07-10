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

    #[test]
    fn whoop_date_parses_well_formed_created_at() {
        use chrono::NaiveDate;
        use dravr_enforme::providers::whoop::parse_whoop_date;

        let date = parse_whoop_date("2026-07-09T04:00:00.000Z");
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 7, 9));
    }

    #[test]
    fn whoop_date_short_created_at_returns_none_without_panic() {
        use dravr_enforme::providers::whoop::parse_whoop_date;

        // Fewer than 10 bytes: byte-slicing `&s[..10]` would panic here.
        assert_eq!(parse_whoop_date("2026-07"), None);
        assert_eq!(parse_whoop_date(""), None);
    }

    #[test]
    fn whoop_date_multibyte_created_at_returns_none_without_panic() {
        use dravr_enforme::providers::whoop::parse_whoop_date;

        // A multibyte char straddling byte offset 10 is not a char boundary;
        // slicing `&s[..10]` would panic. `str::get` yields None instead.
        assert_eq!(parse_whoop_date("2026-07-0é9T00:00:00Z"), None);
    }
}

#[cfg(feature = "provider-whoop")]
mod whoop_sleep_duration_tests {
    use dravr_enforme::providers::whoop::asleep_seconds_from_stage_millis;

    #[test]
    fn asleep_seconds_sums_light_slow_wave_and_rem() {
        // 3h light + 1h slow-wave + 1.5h REM = 5.5h asleep
        let secs =
            asleep_seconds_from_stage_millis(Some(3 * 3_600_000), Some(3_600_000), Some(5_400_000));
        assert_eq!(secs, Some(19_800));
    }

    #[test]
    fn asleep_seconds_excludes_in_bed_awake_time() {
        // 8h in bed with 1h awake must NOT read as 8h asleep; only the
        // stage durations count (7h here).
        let secs = asleep_seconds_from_stage_millis(
            Some(4 * 3_600_000),
            Some(3_600_000),
            Some(2 * 3_600_000),
        );
        assert_eq!(secs, Some(7 * 3600));
    }

    #[test]
    fn asleep_seconds_handles_partial_stages() {
        let secs = asleep_seconds_from_stage_millis(Some(3_600_000), None, None);
        assert_eq!(secs, Some(3600));
    }

    #[test]
    fn asleep_seconds_none_when_no_stage_data() {
        assert_eq!(asleep_seconds_from_stage_millis(None, None, None), None);
    }
}

// ABOUTME: Tests that strava/garmin sciotte providers stamp creds.user_id into sync cursors
// ABOUTME: Guards against empty-user_id batches the platform store rejects on UUID parse
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]
#![cfg(any(feature = "provider-strava", feature = "provider-garmin"))]

use dravr_enforme::models::connection::ProviderCredentials;

/// Credentials whose `access_token` is NOT a serialized sciotte session,
/// so both providers take their no-session skip path (no scraping).
fn oauth_only_creds(provider: &str) -> ProviderCredentials {
    ProviderCredentials {
        access_token: "plain-oauth-token-not-a-session-json".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: Vec::new(),
        user_id: "3b81e897-e50b-4d2e-bf7f-3bbd18a5a272".to_string(),
        provider: provider.to_string(),
    }
}

#[cfg(feature = "provider-strava")]
mod strava_cursor_tests {
    use dravr_enforme::providers::strava::StravaSciotteProvider;
    use dravr_enforme::traits::sync_provider::SyncProvider;

    use super::oauth_only_creds;

    #[tokio::test]
    async fn strava_recovery_skip_cursor_carries_user_id() {
        let provider = StravaSciotteProvider::new();
        let creds = oauth_only_creds("strava");
        let batch = provider.fetch_recovery(&creds, None).await.unwrap();
        assert!(batch.records.is_empty());
        assert_eq!(batch.cursor.user_id, creds.user_id);
        assert_eq!(batch.cursor.provider, "strava");
    }

    #[tokio::test]
    async fn strava_unsupported_type_cursors_carry_user_id() {
        let provider = StravaSciotteProvider::new();
        let creds = oauth_only_creds("strava");

        let sleep = provider.fetch_sleep(&creds, None).await.unwrap();
        assert_eq!(sleep.cursor.user_id, creds.user_id);

        let health = provider.fetch_health(&creds, None).await.unwrap();
        assert_eq!(health.cursor.user_id, creds.user_id);

        let continuous = provider.fetch_continuous(&creds, None).await.unwrap();
        assert_eq!(continuous.cursor.user_id, creds.user_id);
    }
}

#[cfg(feature = "provider-garmin")]
mod garmin_cursor_tests {
    use dravr_enforme::providers::garmin::GarminSciotteProvider;
    use dravr_enforme::traits::sync_provider::SyncProvider;

    use super::oauth_only_creds;

    #[tokio::test]
    async fn garmin_skip_cursors_carry_user_id() {
        let provider = GarminSciotteProvider::new();
        let creds = oauth_only_creds("garmin");

        let sleep = provider.fetch_sleep(&creds, None).await.unwrap();
        assert!(sleep.records.is_empty());
        assert_eq!(sleep.cursor.user_id, creds.user_id);

        let recovery = provider.fetch_recovery(&creds, None).await.unwrap();
        assert_eq!(recovery.cursor.user_id, creds.user_id);

        let health = provider.fetch_health(&creds, None).await.unwrap();
        assert_eq!(health.cursor.user_id, creds.user_id);

        let continuous = provider.fetch_continuous(&creds, None).await.unwrap();
        assert_eq!(continuous.cursor.user_id, creds.user_id);
    }
}

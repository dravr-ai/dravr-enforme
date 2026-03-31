// ABOUTME: Provider registry for managing wearable data providers
// ABOUTME: Feature-gated provider implementations (WHOOP, Garmin, Strava)
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::collections::HashMap;

use crate::traits::sync_provider::SyncProvider;

/// WHOOP API v1 provider implementation
#[cfg(feature = "provider-whoop")]
pub mod whoop;

/// Garmin Connect provider via dravr-sciotte browser scraping
#[cfg(feature = "provider-garmin")]
pub mod garmin;

/// Strava provider via dravr-sciotte browser scraping
#[cfg(feature = "provider-strava")]
pub mod strava;

/// Build a map of all enabled provider implementations.
#[must_use]
pub fn build_provider_registry() -> HashMap<String, Box<dyn SyncProvider>> {
    #[allow(unused_mut)]
    let mut providers: HashMap<String, Box<dyn SyncProvider>> = HashMap::new();

    #[cfg(feature = "provider-whoop")]
    {
        let whoop = whoop::WhoopProvider::new();
        providers.insert(whoop.name().to_owned(), Box::new(whoop));
    }

    #[cfg(feature = "provider-garmin")]
    {
        let garmin = garmin::GarminSciotteProvider::new();
        providers.insert(garmin.name().to_owned(), Box::new(garmin));
    }

    #[cfg(feature = "provider-strava")]
    {
        let strava = strava::StravaSciotteProvider::new();
        providers.insert(strava.name().to_owned(), Box::new(strava));
    }

    providers
}

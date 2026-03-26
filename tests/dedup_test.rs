// ABOUTME: Tests for device and provider priority deduplication logic
// ABOUTME: Verifies correct winner selection based on priority ordering
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use dravr_enforme::orchestrator::dedup::{select_best_source, should_replace};
use dravr_equilibre::data_source::DeviceType;

#[test]
fn watch_replaces_phone() {
    assert!(should_replace(
        &DeviceType::Phone,
        "garmin",
        &DeviceType::Watch,
        "garmin"
    ));
}

#[test]
fn phone_does_not_replace_watch() {
    assert!(!should_replace(
        &DeviceType::Watch,
        "garmin",
        &DeviceType::Phone,
        "garmin"
    ));
}

#[test]
fn same_device_higher_priority_provider_wins() {
    // garmin (priority 2) beats whoop (priority 5)
    assert!(should_replace(
        &DeviceType::Watch,
        "whoop",
        &DeviceType::Watch,
        "garmin"
    ));
}

#[test]
fn same_device_lower_priority_provider_loses() {
    assert!(!should_replace(
        &DeviceType::Watch,
        "garmin",
        &DeviceType::Watch,
        "whoop"
    ));
}

#[test]
fn band_replaces_ring() {
    assert!(should_replace(
        &DeviceType::Ring,
        "oura",
        &DeviceType::Band,
        "fitbit"
    ));
}

#[test]
fn select_best_source_empty_returns_none() {
    assert!(select_best_source(&[]).is_none());
}

#[test]
fn select_best_source_single_returns_zero() {
    let sources = vec![(DeviceType::Watch, "garmin")];
    assert_eq!(select_best_source(&sources), Some(0));
}

#[test]
fn select_best_source_watch_wins_over_phone() {
    let sources = vec![(DeviceType::Phone, "garmin"), (DeviceType::Watch, "garmin")];
    assert_eq!(select_best_source(&sources), Some(1));
}

#[test]
fn select_best_source_higher_priority_provider_wins() {
    let sources = vec![(DeviceType::Watch, "whoop"), (DeviceType::Watch, "garmin")];
    assert_eq!(select_best_source(&sources), Some(1));
}

#[test]
fn select_best_source_multiple_devices() {
    let sources = vec![
        (DeviceType::Scale, "withings"),
        (DeviceType::Ring, "oura"),
        (DeviceType::Watch, "apple"),
        (DeviceType::Band, "fitbit"),
    ];
    // Watch has highest priority
    assert_eq!(select_best_source(&sources), Some(2));
}

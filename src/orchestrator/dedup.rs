// ABOUTME: Deduplication logic using device and provider priority from dravr-equilibre
// ABOUTME: Resolves conflicts when multiple sources report the same metric
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::cmp::Ordering;

use dravr_equilibre::data_source::DeviceType;
use dravr_equilibre::{DevicePriority, ProviderPriority};

/// Determine whether a new data point should replace an existing one.
///
/// Uses device priority first, then provider priority as tiebreaker.
/// Returns `true` if the new source should replace the existing one.
#[must_use]
pub fn should_replace(
    existing_device: &DeviceType,
    existing_provider: &str,
    new_device: &DeviceType,
    new_provider: &str,
) -> bool {
    let existing_device_priority = DevicePriority::priority(existing_device);
    let new_device_priority = DevicePriority::priority(new_device);

    // Lower priority number = higher priority
    match new_device_priority.cmp(&existing_device_priority) {
        Ordering::Less => true,
        Ordering::Greater => false,
        Ordering::Equal => {
            // Tiebreak by provider priority
            let existing_provider_priority = ProviderPriority::priority(existing_provider);
            let new_provider_priority = ProviderPriority::priority(new_provider);
            new_provider_priority < existing_provider_priority
        }
    }
}

/// Select the highest-priority source from a list of `(device_type, provider)` pairs.
///
/// Returns the index of the winning source, or `None` if the list is empty.
#[must_use]
pub fn select_best_source(sources: &[(DeviceType, &str)]) -> Option<usize> {
    if sources.is_empty() {
        return None;
    }

    let mut best_idx = 0;
    for (idx, (device, provider)) in sources.iter().enumerate().skip(1) {
        let (best_device, best_provider) = &sources[best_idx];
        if should_replace(best_device, best_provider, device, provider) {
            best_idx = idx;
        }
    }

    Some(best_idx)
}

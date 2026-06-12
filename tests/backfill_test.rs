// ABOUTME: Tests for backfill result construction and data type tracking
// ABOUTME: Verifies BackfillResult defaults and aggregation
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string
)]

use dravr_enforme::orchestrator::backfill::BackfillResult;
use dravr_enforme::traits::sync_provider::DataType;

#[test]
fn backfill_result_default() {
    let result = BackfillResult::default();
    assert_eq!(result.total_records, 0);
    assert!(result.records_by_type.is_empty());
    assert_eq!(result.errors, 0);
}

#[test]
fn backfill_result_with_records() {
    let result = BackfillResult {
        total_records: 100,
        records_by_type: vec![
            (DataType::Sleep, 50),
            (DataType::Recovery, 30),
            (DataType::Health, 20),
        ],
        errors: 0,
    };
    assert_eq!(result.total_records, 100);
    assert_eq!(result.records_by_type.len(), 3);
}

#[test]
fn backfill_result_with_errors() {
    let result = BackfillResult {
        total_records: 50,
        records_by_type: vec![(DataType::Sleep, 50)],
        errors: 2,
    };
    assert_eq!(result.errors, 2);
}

#[test]
fn backfill_result_is_debug() {
    let result = BackfillResult::default();
    let debug = format!("{result:?}");
    assert!(debug.contains("BackfillResult"));
}

#[test]
fn data_type_as_str() {
    assert_eq!(DataType::Sleep.as_str(), "sleep");
    assert_eq!(DataType::Recovery.as_str(), "recovery");
    assert_eq!(DataType::Health.as_str(), "health");
    assert_eq!(DataType::Continuous.as_str(), "continuous");
}

#[test]
fn data_type_display() {
    assert_eq!(format!("{}", DataType::Sleep), "sleep");
    assert_eq!(format!("{}", DataType::Recovery), "recovery");
    assert_eq!(format!("{}", DataType::Health), "health");
    assert_eq!(format!("{}", DataType::Continuous), "continuous");
}

#[test]
fn data_type_equality() {
    assert_eq!(DataType::Sleep, DataType::Sleep);
    assert_ne!(DataType::Sleep, DataType::Recovery);
}

#[test]
fn data_type_serialization() {
    let json = serde_json::to_string(&DataType::Sleep).unwrap();
    let deserialized: DataType = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, DataType::Sleep);
}

#[test]
fn data_type_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(DataType::Sleep);
    set.insert(DataType::Recovery);
    set.insert(DataType::Sleep); // duplicate
    assert_eq!(set.len(), 2);
}

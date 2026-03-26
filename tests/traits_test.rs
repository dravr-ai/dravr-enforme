// ABOUTME: Compile-time Send+Sync checks for all traits
// ABOUTME: Ensures all store traits and provider traits are object-safe
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use dravr_enforme::traits::connect_hook::OnConnectHook;
use dravr_enforme::traits::connection_store::UserConnectionStore;
use dravr_enforme::traits::credential_store::CredentialStore;
use dravr_enforme::traits::cursor_store::SyncCursorStore;
use dravr_enforme::traits::data_source_store::DataSourceStore;
use dravr_enforme::traits::health_store::HealthStore;
use dravr_enforme::traits::recovery_store::RecoveryStore;
use dravr_enforme::traits::sleep_store::SleepStore;
use dravr_enforme::traits::sync_provider::SyncProvider;
use dravr_enforme::traits::timeseries_store::TimeSeriesPointStore;

fn assert_send_sync<T: Send + Sync + ?Sized>() {}

#[test]
fn sleep_store_is_send_sync() {
    assert_send_sync::<dyn SleepStore>();
}

#[test]
fn recovery_store_is_send_sync() {
    assert_send_sync::<dyn RecoveryStore>();
}

#[test]
fn health_store_is_send_sync() {
    assert_send_sync::<dyn HealthStore>();
}

#[test]
fn timeseries_store_is_send_sync() {
    assert_send_sync::<dyn TimeSeriesPointStore>();
}

#[test]
fn data_source_store_is_send_sync() {
    assert_send_sync::<dyn DataSourceStore>();
}

#[test]
fn cursor_store_is_send_sync() {
    assert_send_sync::<dyn SyncCursorStore>();
}

#[test]
fn credential_store_is_send_sync() {
    assert_send_sync::<dyn CredentialStore>();
}

#[test]
fn connection_store_is_send_sync() {
    assert_send_sync::<dyn UserConnectionStore>();
}

#[test]
fn sync_provider_is_send_sync() {
    assert_send_sync::<dyn SyncProvider>();
}

#[test]
fn on_connect_hook_is_send_sync() {
    assert_send_sync::<dyn OnConnectHook>();
}

#[test]
fn sync_deps_is_send_sync() {
    assert_send_sync::<dravr_enforme::traits::SyncDeps>();
}

// Verify trait objects can be stored in Arc
#[test]
fn traits_are_object_safe() {
    use std::sync::Arc;

    fn take_sleep(_: Arc<dyn SleepStore>) {}
    fn take_recovery(_: Arc<dyn RecoveryStore>) {}
    fn take_health(_: Arc<dyn HealthStore>) {}
    fn take_timeseries(_: Arc<dyn TimeSeriesPointStore>) {}
    fn take_data_source(_: Arc<dyn DataSourceStore>) {}
    fn take_cursor(_: Arc<dyn SyncCursorStore>) {}
    fn take_credential(_: Arc<dyn CredentialStore>) {}
    fn take_connection(_: Arc<dyn UserConnectionStore>) {}
    fn take_provider(_: Box<dyn SyncProvider>) {}
    fn take_hook(_: Box<dyn OnConnectHook>) {}

    // These compile = traits are object-safe
    let _ = take_sleep;
    let _ = take_recovery;
    let _ = take_health;
    let _ = take_timeseries;
    let _ = take_data_source;
    let _ = take_cursor;
    let _ = take_credential;
    let _ = take_connection;
    let _ = take_provider;
    let _ = take_hook;
}

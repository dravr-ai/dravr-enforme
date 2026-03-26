// ABOUTME: Domain models for sync operations
// ABOUTME: Cursors, webhooks, deletion policies, and provider connections
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

/// Provider connection and credential models
pub mod connection;
/// Sync cursor and batch models for incremental sync
pub mod cursor;
/// Deletion policy and strategy configuration
pub mod deletion;
/// Webhook event and configuration models
pub mod webhook;

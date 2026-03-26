// ABOUTME: Health data sync orchestrator for the Dravr platform
// ABOUTME: Defines sync traits, models, and provider abstractions for wearable health data
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

#![deny(unsafe_code)]

//! # dravr-enforme
//!
//! Health data sync orchestrator — webhook-driven provider sync with cursor-based
//! CDC for the Dravr platform.
//!
//! ## Architecture
//!
//! - **8 granular store traits** (Interface Segregation) for write-path operations
//! - **`SyncProvider`** trait — per-provider, feature-gated implementations (WHOOP first)
//! - **Orchestrator** — central coordinator for sync, backfill, and webhooks
//! - **CDC cursors** — incremental sync per user, provider, and data type
//!
//! ## Features
//!
//! - `webhooks` — webhook handling with axum types
//! - `provider-whoop` — WHOOP API v1 provider implementation
//! - `provider-garmin` — Garmin API provider (planned)
//! - `all-providers` — enable all provider implementations

// ============================================================================
// Core modules
// ============================================================================

/// Sync configuration from environment variables
pub mod config;
/// Structured error types for sync operations
pub mod error;

// ============================================================================
// Domain models
// ============================================================================

/// Sync models: cursors, webhooks, deletion policy, connections
pub mod models;

// ============================================================================
// Store traits (write path, Interface Segregation)
// ============================================================================

/// Store traits and provider abstractions
pub mod traits;

// ============================================================================
// Orchestration
// ============================================================================

/// Sync orchestrator, scheduler, backfill, and deduplication
pub mod orchestrator;

// ============================================================================
// Webhook handling
// ============================================================================

/// Webhook validation and event processing
pub mod webhook;

// ============================================================================
// Rate limiting
// ============================================================================

/// Token bucket rate limiter per provider
pub mod rate_limit;

// ============================================================================
// Provider implementations (feature-gated)
// ============================================================================

/// Provider registry and implementations
pub mod providers;

// ============================================================================
// Re-exports
// ============================================================================

pub use config::SyncConfig;
pub use error::{EnformeError, EnformeResult};
pub use models::connection::{ConnectedUser, ProviderCredentials};
pub use models::cursor::{SyncBatch, SyncCursor};
pub use models::deletion::{DeletionPolicy, DeletionStrategy};
pub use models::webhook::{WebhookAlgorithm, WebhookConfig, WebhookEvent};
pub use orchestrator::SyncOrchestrator;
pub use rate_limit::TokenBucket;
pub use traits::connect_hook::OnConnectHook;
pub use traits::connection_store::UserConnectionStore;
pub use traits::credential_store::CredentialStore;
pub use traits::cursor_store::SyncCursorStore;
pub use traits::data_source_store::DataSourceStore;
pub use traits::health_store::HealthStore;
pub use traits::recovery_store::RecoveryStore;
pub use traits::sleep_store::SleepStore;
pub use traits::sync_provider::{DataType, SyncProvider};
pub use traits::timeseries_store::TimeSeriesPointStore;
pub use traits::SyncDeps;

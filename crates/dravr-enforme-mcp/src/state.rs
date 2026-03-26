// ABOUTME: Shared server state for the MCP server
// ABOUTME: Thread-safe state container holding sync orchestrator configuration
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use tokio::sync::RwLock;

/// Thread-safe shared state type alias.
pub type SharedState = Arc<RwLock<ServerState>>;

/// Server state holding sync orchestrator configuration.
#[derive(Debug)]
pub struct ServerState {}

impl ServerState {
    /// Create a new server state.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new shared state instance.
#[must_use]
pub fn create_shared_state() -> SharedState {
    Arc::new(RwLock::new(ServerState::new()))
}

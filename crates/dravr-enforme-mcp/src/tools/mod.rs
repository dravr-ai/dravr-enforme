// ABOUTME: MCP tool registry for sync orchestrator tools
// ABOUTME: Pluggable tool architecture using dravr-tronc shared McpTool trait and ToolRegistry
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use dravr_tronc::ToolRegistry;

use crate::state::ServerState;

/// Build the default tool registry with all available sync tools.
#[must_use]
pub fn build_tool_registry() -> ToolRegistry<ServerState> {
    ToolRegistry::new()
}

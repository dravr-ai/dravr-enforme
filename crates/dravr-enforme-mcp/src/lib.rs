// ABOUTME: MCP server library exposing sync orchestrator tools via Model Context Protocol
// ABOUTME: JSON-RPC 2.0 protocol with stdio and HTTP transports via dravr-tronc
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

//! # dravr-enforme-mcp
//!
//! MCP (Model Context Protocol) server for the dravr-enforme sync orchestrator.
//! Exposes sync tools as MCP tools via JSON-RPC 2.0.

/// Shared server state
pub mod state;
/// MCP tool definitions and registry
pub mod tools;

pub use dravr_tronc::McpServer;
pub use state::{ServerState, SharedState};
pub use tools::build_tool_registry;

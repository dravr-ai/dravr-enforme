## Git Workflow: NO Pull Requests


## Mandatory Session Setup (ALL AI Agents)

**Run these commands at the START OF EVERY SESSION:**

```bash
# 1. Initialize shared build config (required for validation)
git submodule update --init --recursive

# 2. Set git hooks
git config core.hooksPath .build/hooks
```

## Mandatory Pre-Push Validation

**Before EVERY push, run:**

```bash
# 1. Format
cargo fmt --all

# 2. Clippy with warnings as errors
cargo clippy --workspace --all-targets -- -D warnings

# 3. Architectural validation (MUST exit 0)
.build/validation/validate.sh
```

**DO NOT push if `.build/validation/validate.sh` fails.** Fix all reported issues first.

The validation checks: placeholder code, forbidden anyhow usage, problematic unwraps/expects/panics,
underscore-prefixed names, unauthorized clippy allows, dead code annotations, test integrity, and more.

**CRITICAL: NEVER create Pull Requests. All merges happen locally via squash merge.**

### Rules
- **NEVER use `gh pr create`** or any PR creation command
- **NEVER suggest creating a PR**
- Feature branches are merged via **local squash merge**

### Workflow for Features
1. Create feature branch: `git checkout -b feature/my-feature`
2. Make commits, push to remote: `git push -u origin feature/my-feature`
3. When ready, squash merge locally (from main worktree):
   ```bash
   git checkout main
   git fetch origin
   git merge --squash origin/feature/my-feature
   git commit
   git push
   ```

### Bug Fixes
- Bug fixes go directly to `main` branch (no feature branch needed)
- Commit and push directly: `git push origin main`

## Rust Workspace Architecture

The backend is a Cargo workspace with 3 crates:

| Crate | Description |
|-------|-------------|
| `dravr-enforme` | Core library with sync traits, models, orchestrator, and providers |
| `dravr-enforme-mcp` | MCP server exposing sync tools via Model Context Protocol |
| `dravr-enforme-server` | Unified REST API + MCP server + CLI binary |

## Project Overview

**dravr-enforme** is a health data sync orchestrator with webhook-driven provider sync and cursor-based CDC for the Dravr platform.

### Key Concepts
- **8 store traits** — Interface Segregation for write-path: SleepStore, RecoveryStore, HealthStore, TimeSeriesPointStore, DataSourceStore, SyncCursorStore, CredentialStore, UserConnectionStore
- **SyncProvider trait** — per-provider, feature-gated implementations (WHOOP first)
- **SyncOrchestrator** — central coordinator for sync, backfill, webhooks, rate limiting
- **SyncDeps** — Arc-based dependency injection container (avoids generic explosion)
- **CDC cursors** — incremental sync per user+provider+data_type
- **HMAC-SHA256 webhooks** — Standard Webhooks spec
- **Soft delete default** — configurable via ENFORME_DELETION_STRATEGY env var

### Design Decisions
- Providers return `StoredSleepSession` directly from dravr-equilibre (no intermediate types)
- CQRS: enforme defines write-path traits; read-path stays in Pierre
- Last Writer Wins with timestamp for conflict resolution
- Token bucket rate limiting per provider

## Git Hooks - MANDATORY for ALL AI Agents

**MANDATORY - Run this at the START OF EVERY SESSION:**
```bash
git config core.hooksPath .githooks
```

**NEVER use `--no-verify` when committing or pushing.** The hooks enforce:
- SPDX license headers on all source files
- Commit message format (max 2 lines, conventional commits)
- No AI-generated commit signatures
- No unauthorized root markdown files

## Pre-Push Validation Workflow

1. **Make your changes and commit**
2. **Run validation before pushing:**
   ```bash
   ./scripts/pre-push-validate.sh
   ```
3. **Push:**
   ```bash
   git push
   ```

# Writing code

- CRITICAL: NEVER USE --no-verify WHEN COMMITTING CODE
- Every .rs file: 2 ABOUTME lines + SPDX header (`// SPDX-License-Identifier: Apache-2.0` + `// Copyright (c) 2026 dravr.ai`)
- NEVER use anyhow — use `EnformeError` with thiserror
- NEVER use unwrap()/expect() in production code
- NEVER use `#[allow(clippy::...)]` except for cast truncation/loss/precision
- Observability: `tracing = "0.1"` only. `#[instrument]` on key methods. NEVER import tracing-subscriber/opentelemetry.
- Commits: conventional format, max 2 lines
- Avoid #[cfg(test)] in the src code. Only in tests

## Error Handling Requirements

### Acceptable Error Handling
- `?` operator for error propagation
- `Result<T, E>` for all fallible operations
- Custom error types implementing `std::error::Error`

### Prohibited Error Handling
- `unwrap()` except in test code
- `expect()` - Acceptable ONLY for documenting invariants that should never fail
- `panic!()` - Only in test assertions
- `anyhow!()` - FORBIDDEN entirely

## Required Pre-Commit Validation

### Tiered Validation Approach

#### Tier 1: Quick Iteration (during development)
```bash
cargo fmt
cargo check --quiet
cargo test --test <test_file> <test_name_pattern> -- --nocapture
```

#### Tier 2: Pre-Commit (before committing)
```bash
cargo fmt
cargo clippy -p dravr-enforme
cargo test --test <test_file> <test_pattern> -- --nocapture
```

#### Tier 3: Full Validation (before merge only)
```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

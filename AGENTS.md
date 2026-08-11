## Git Workflow: NO Pull Requests

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

## After Pushing — MANDATORY CI MONITORING

The Agent MUST monitor CI on every push and not consider work complete until the relevant workflows reach a terminal success status. If CI fails, fix and re-push in the same session. The shared pre-push hook (`.build/hooks/pre-push`) prints this reminder on success.

**Tool priority** (to preserve GitHub PAT rate-limit quota):
1. **WebFetch** the branch's Actions page at `https://github.com/dravr-ai/dravr-enforme/actions?query=branch%3A<branch>` — no PAT quota cost
2. `gh run list --branch <branch>` or single targeted `gh run view <id>` — costs one core slot each
3. GitHub MCP tools (`mcp__github__*`) for non-read operations

**Forbidden:** `gh run watch`, background polling loops, sub-60s polling cadence.

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

### Tiered Validation (During Development)

| Tier | When | Commands |
|------|------|----------|
| Quick | During dev iteration | `cargo check --quiet && cargo test --test <file> <pattern>` |
| Pre-commit | Before each commit | `cargo fmt --all && cargo clippy -p <changed-crate>` |
| Full | Before push (see above) | `cargo fmt + clippy + .build/validation/validate.sh` |

## Mandatory Session Startup Checklist

Before touching any code in a new session, run in this order:

```bash
# 1. Pull shared build config (provides .build/hooks, .build/validation, etc.)
git submodule update --init --recursive

# 2. Set canonical git hooks path — ALWAYS .build/hooks, NEVER .githooks
git config core.hooksPath .build/hooks

# 3. Scan recent history for context
git log --oneline -10

# 4. Check CI health on main
gh run list --branch main --limit 10 --json workflowName,conclusion

# 5. See uncommitted work
git status
```

**If any workflow on main has been red for 2+ runs, STOP and surface it to the user** before starting the requested task. Ask: "Should I investigate CI before doing X?"

The canonical hooks/validation live in the `.build/` git submodule from
https://github.com/dravr-ai/dravr-build-config — never use a local `.githooks/`.

## Architectural Discipline

### Single Source of Truth (SSOT)
Before adding a new abstraction (registry, manager, factory, handler, schema module):
1. Grep for existing abstractions with similar purposes
2. If one exists, USE IT or DOCUMENT WHY it's being replaced + DELETE the old in the same commit
3. Never leave two systems doing the same job "for compat"

### No Orphan Migrations
If you introduce a "v2" of something:
- Migrate ALL callers in the same session, OR
- Record remaining work in memory (`type: project`) with explicit list of what's left
- NEVER leave "for compat" code without a tracked deletion date

### When Adding, Remove
Every commit that adds a new abstraction must identify what it replaces and delete that. If nothing is replaced, the commit message must justify why the new abstraction is needed.

### Complete Deletion, Not Deprecation
Don't mark code `// DEPRECATED` or `// TODO remove later`. Delete it. If deletion is blocked, file an issue and link it from the code.

## Pushback Triggers — When to Stop and Ask

STOP and ask the user before proceeding when you find:

1. **Duplication** — two systems/modules doing similar things
   → "Is this intentional? Should I consolidate before adding my feature?"
2. **Stale state** — `TODO`, `FIXME`, `for compat`, `temporary`, `v2` comments in code you're touching
   → "Is this still needed? Should I resolve it first?"
3. **Red CI** — workflows failing on main
   → "Should I fix CI first before doing the task?"
4. **Version drift** — two versions of the same dependency in Cargo.lock
   → "Is this intentional or should it be consolidated?"
5. **Request conflicts with architecture** — user asks you to add X but X exists differently
   → Surface the existing thing, ask which to use
6. **Half-finished migrations** — both old and new paths still live
   → "Finish migration first, or add feature on top?"

Default behavior is to complete the requested task. These triggers override that.

## Limitation register (org-wide)

- A genuine, documented limitation in code carries `LIMITATION(registre#<issue>):` on the marker line, naming the limited item, backed by an issue in the **private** `dravr-ai/dravr-registre` tracker (labels `limitation` + this repo's name). Most dravr repos are PUBLIC — internal gaps and security residuals never go on this repo's own tracker.
- Deferral/confession prose ("for now", "not yet implemented", "is the follow-up", "in a follow-up commit", "not yet wired", "not threaded through") is CI-gated by `.build/validation/limitation-gates.sh` (invoked by `validate.sh`); a registered marker line is the only exemption. Implement the real thing, or register the gap — never document it unregistered.
- A capability declared but consumed only by tests is a phantom surface: wire a production consumer in the same change, or register it with a marker naming the item.
- A feature shipped disarmed (flag off, shadow/observe mode, log-only phase) gets a `feature-phases.yaml` entry (name/surface/current/advance_when/review_by); dravr-build-config's reusable `feature-phase-review` workflow opens a registre issue when the review date passes.

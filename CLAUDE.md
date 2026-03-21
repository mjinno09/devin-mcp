# CLAUDE.md

## Project Overview

devin-mcp is a Rust MCP server that wraps the Devin API. It enables Claude Code, Cursor, and other MCP clients to create and manage Devin AI sessions via stdio transport.

## Tech Stack

- Rust (edition 2021)
- rmcp (official MCP SDK) — `#[tool]` macro for tool definitions
- reqwest — HTTP client for Devin API
- tokio — async runtime
- wiremock — HTTP mocking for tests

## Architecture

- `src/main.rs` — Entry point. stdio transport. All logging to stderr.
- `src/server.rs` — MCP ServerHandler with 4 tools: create_session, get_session, list_sessions, send_message
- `src/devin_client.rs` — Devin API v1 client. Stateless, all methods take `&self`.
- `tests/integration.rs` — E2E tests spawning the binary as a child process.

See `docs/architecture.md` for the full architecture overview.

## Design Principles

- **Fire and Forget**: create_session returns a URL immediately. No polling.
- **Stateless**: Server holds no state between tool calls. Pure API passthrough.
- **stdout is sacred**: Only JSON-RPC goes to stdout. Everything else to stderr.
- **No unwrap() in production code**: Use `anyhow::Result` or `McpError`. `unwrap()` is only for tests.

## Commands

```sh
cargo build                    # Build
cargo test --bin devin-mcp     # Unit tests (wiremock, no API key needed)
cargo test --test integration  # Integration tests (mock only)
DEVIN_API_KEY=xxx cargo test   # All tests including live
cargo fmt --all --check        # Format check
cargo clippy --all-targets     # Lint
```

## Code Conventions

- Derive `schemars::JsonSchema` on all tool parameter structs
- Doc comments on struct fields become MCP tool parameter descriptions for the LLM
- Error handling: `anyhow::Result` for app errors, `McpError` for MCP protocol errors
- Logging: use tracing macros (`info!`, `error!`), never `println!`
- Tests: use wiremock for HTTP mocking, actual binary spawn for integration tests

## Workflow

- Before starting a task, create a plan in `docs/plan/`
- For architecture changes, write an ADR in `docs/adr/`
- Run `cargo fmt` and `cargo clippy` before committing
- Keep CHANGELOG.md updated for user-facing changes

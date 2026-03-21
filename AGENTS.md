# AGENTS.md

<!-- This file is read by Cursor's Agent mode. Keep in sync with CLAUDE.md. -->

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
- `src/devin_client.rs` — Devin API v1 client. Stateless.
- `tests/integration.rs` — E2E tests spawning the binary as a child process.

## Design Principles

- **Fire and Forget**: create_session returns a URL immediately. No polling.
- **Stateless**: Server holds no state between tool calls.
- **stdout is sacred**: Only JSON-RPC goes to stdout. Everything else to stderr.

## Commands

- `cargo build` — Build
- `cargo test --lib` — Unit tests
- `cargo test --test integration` — Integration tests
- `cargo fmt --all --check` — Format check
- `cargo clippy --all-targets` — Lint

## Code Conventions

- Derive `schemars::JsonSchema` on all tool parameter structs
- Doc comments on struct fields become MCP tool parameter descriptions
- No `unwrap()` in production code
- No `println!()` — use `tracing` macros, output to stderr

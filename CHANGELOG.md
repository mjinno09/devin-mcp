# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- Release workflow failing when tag is created via GitHub UI

## [0.1.0] - 2026-03-21

### Added

- `create_session` tool — Create a Devin session and return the session URL
- `get_session` tool — Get status and details of an existing session
- `list_sessions` tool — List recent Devin sessions
- `send_message` tool — Send follow-up instructions to a session
- stdio transport for Claude Code, Cursor, and other MCP clients
- GitHub Actions CI (fmt, clippy, unit tests, integration tests)

# devin-mcp

[![CI](https://github.com/mjinno09/devin-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/mjinno09/devin-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

An MCP (Model Context Protocol) server for creating and managing [Devin AI](https://devin.ai) sessions. Use it from [Claude Code](https://code.claude.com/), [Cursor](https://cursor.com/), or any MCP-compatible client.

## Features

- **create_session** — Ask Devin to work on a task (returns session URL, fire-and-forget)
- **get_session** — Check the status of an existing session
- **list_sessions** — Browse recent sessions
- **send_message** — Send follow-up instructions to a running session

## Install

### Homebrew (recommended)

```sh
brew install mjinno09/tap/devin-mcp
```

### Shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/mjinno09/devin-mcp/releases/latest/download/devin-mcp-installer.sh | sh
```

**macOS:** If the binary is blocked by Gatekeeper after installation, run:

```sh
xattr -c ~/.cargo/bin/devin-mcp
```

## Setup

### Prerequisites

- A Devin account and API key

### Claude Code

```sh
claude mcp add --transport stdio --scope user \
  --env DEVIN_API_KEY="${DEVIN_API_KEY}" \
  devin-mcp -- devin-mcp
```

### Cursor

Create `.cursor/mcp.json` in your project root:

```json
{
  "mcpServers": {
    "devin-mcp": {
      "command": "devin-mcp",
      "env": {
        "DEVIN_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

### Other MCP clients

Any client that supports stdio transport:

```json
{
  "mcpServers": {
    "devin-mcp": {
      "command": "devin-mcp",
      "env": {
        "DEVIN_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

## Usage

From Claude Code:

```
> Ask Devin to fix the login bug in issue #42

Devin session created.
- Session: https://app.devin.ai/sessions/devin-abc123
- Status: running
Devin is working on it. A PR will be created when done.
```

## Configuration

| Environment Variable | Required | Description |
|---|---|---|
| `DEVIN_API_KEY` | Yes | Devin API key (get from Settings) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT — see [LICENSE](LICENSE) for details.

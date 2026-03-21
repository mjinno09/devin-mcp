# Architecture

## Overview

```mermaid
graph LR
    Client["Claude Code / Cursor / etc<br/>(MCP Client)"]
    Server["devin-mcp<br/>(MCP Server)"]
    API["Devin API<br/>(v1)"]
    GitHub["GitHub<br/>(PR)"]

    Client -- "stdio (JSON-RPC)" --> Server
    Server -- "HTTPS" --> API
    API -. "非同期で作業後" .-> GitHub

    style Server fill:#f5f5f5,stroke:#333
    style API fill:#e8f4fd,stroke:#333
    style GitHub fill:#e6ffe6,stroke:#333
```

## Components

### MCP Server (`src/server.rs`)

4 つのツールを提供する MCP サーバー。

| Tool | Description | Devin API |
|------|-------------|-----------|
| `create_session` | タスクを依頼。URL を返して即完了 | `POST /v1/sessions` |
| `get_session` | セッション状態を確認 | `GET /v1/sessions/{id}` |
| `list_sessions` | セッション一覧を取得 | `GET /v1/sessions` |
| `send_message` | 追加指示を送信 | `POST /v1/sessions/{id}` |

### Devin API Client (`src/devin_client.rs`)

Devin REST API v1 の薄いラッパー。ステートレスで、全メソッドが `&self` を取る。
内部状態は持たず、リクエストごとに完結する。

### Transport

stdio（標準入出力）を使用。MCP クライアントが本バイナリを子プロセスとして起動し、
stdin/stdout で JSON-RPC 2.0 メッセージをやりとりする。

**鉄則: stdout には JSON-RPC のみ。ログは全て stderr へ。**

## Data Flow

### create_session

```mermaid
sequenceDiagram
    participant Client as MCP Client
    participant Server as devin-mcp
    participant API as Devin API
    participant GH as GitHub

    Client->>Server: tools/call create_session { prompt: "..." }
    Server->>API: POST /v1/sessions { prompt: "..." }
    API-->>Server: { session_id: "devin-xxx", status: "running" }
    Server-->>Client: Session URL: https://app.devin.ai/sessions/devin-xxx

    Note over API,GH: Devin が非同期で作業
    API--)GH: PR を作成
```

## Design Decisions

設計上の意思決定は `docs/adr/` に記録する。

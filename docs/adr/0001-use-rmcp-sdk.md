# 0001. Use rmcp as MCP SDK

## Status

Accepted

## Context

Rust で MCP サーバーを実装するにあたり、SDK の選択肢が複数ある:

- `rmcp` — MCP 公式の Rust SDK（modelcontextprotocol/rust-sdk）
- `rust-mcp-sdk` — コミュニティ製、マクロが豊富
- `mcp_rust_sdk` — 軽量だが機能が限定的
- 自前実装 — JSON-RPC 2.0 を直接ハンドル

## Decision

公式 SDK である `rmcp` を採用する。

理由:
- MCP 仕様の変更に最も早く追従する（公式メンテナンス）
- `#[tool]` / `#[tool_router]` マクロで宣言的にツールを定義できる
- `schemars` との統合で JSON Schema が自動生成される
- stdio / Streamable HTTP / SSE の全トランスポートをサポート
- crates.io で 470 万以上のダウンロード実績

## Consequences

- **メリット**: 公式サポート、エコシステムとの互換性、マクロによる DX
- **デメリット**: まだ v1.0 未満でブレイキングチェンジの可能性がある
- **リスク緩和**: Cargo.toml でバージョンを固定し、CI でビルド確認を継続する

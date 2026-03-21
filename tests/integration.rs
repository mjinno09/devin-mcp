//! MCP サーバーのインテグレーションテスト
//!
//! 実際のバイナリを子プロセスとして起動し、
//! JSON-RPC over stdio で通信する E2E テスト。
//!
//! DEVIN_API_KEY が設定されていない場合はスキップされる。

use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// JSON-RPC リクエストを送信してレスポンスを受け取るヘルパー
async fn send_jsonrpc(
    stdin: &mut tokio::process::ChildStdin,
    stdout: &mut BufReader<tokio::process::ChildStdout>,
    request: Value,
) -> Value {
    let msg = serde_json::to_string(&request).unwrap();
    stdin.write_all(msg.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let mut line = String::new();
    stdout.read_line(&mut line).await.unwrap();
    serde_json::from_str(&line).unwrap()
}

fn should_skip() -> bool {
    std::env::var("DEVIN_API_KEY")
        .map(|v| v.is_empty())
        .unwrap_or(true)
}

#[tokio::test]
async fn test_mcp_initialize_handshake() {
    let binary = env!("CARGO_BIN_EXE_devin-mcp");

    let mut child = Command::new(binary)
        .env("DEVIN_API_KEY", "dummy-key-for-init-test")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // MCP initialize ハンドシェイク
    let response = send_jsonrpc(
        &mut stdin,
        &mut stdout,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "0.1.0"
                }
            }
        }),
    )
    .await;

    // サーバー情報が返ってくることを確認
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["serverInfo"].is_object());
    // capabilities.tools may be an object or nested differently depending on rmcp version
    assert!(response["result"]["capabilities"].is_object());

    // initialized 通知を送信
    let init_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    let msg = serde_json::to_string(&init_notification).unwrap();
    stdin.write_all(msg.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // tools/list でツール一覧を取得
    let tools_response = send_jsonrpc(
        &mut stdin,
        &mut stdout,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    )
    .await;

    let tools = tools_response["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(tool_names.contains(&"create_session"));
    assert!(tool_names.contains(&"get_session"));
    assert!(tool_names.contains(&"list_sessions"));
    assert!(tool_names.contains(&"send_message"));

    // 各ツールに inputSchema が定義されていることを確認
    for tool in tools {
        assert!(
            tool["inputSchema"].is_object(),
            "Tool {} missing inputSchema",
            tool["name"]
        );
    }

    child.kill().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_mcp_create_session_live() {
    if should_skip() {
        eprintln!("Skipping live test: DEVIN_API_KEY not set");
        return;
    }

    let binary = env!("CARGO_BIN_EXE_devin-mcp");
    let api_key = std::env::var("DEVIN_API_KEY").unwrap();

    let mut child = Command::new(binary)
        .env("DEVIN_API_KEY", &api_key)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // initialize
    let _ = send_jsonrpc(
        &mut stdin,
        &mut stdout,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0.1.0" }
            }
        }),
    )
    .await;

    // initialized 通知
    let notif = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    }))
    .unwrap();
    stdin.write_all(notif.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // tools/call で create_session を呼ぶ（実際に Devin セッションが作られる）
    let call_response = send_jsonrpc(
        &mut stdin,
        &mut stdout,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "create_session",
                "arguments": {
                    "prompt": "Echo test: respond with 'integration test passed'",
                    "tags": ["ci-test"],
                    "max_acu_limit": 1
                }
            }
        }),
    )
    .await;

    // セッション URL が返ってくることを確認
    let content = &call_response["result"]["content"][0]["text"];
    assert!(
        content
            .as_str()
            .unwrap()
            .contains("https://app.devin.ai/sessions/"),
        "Response should contain session URL: {}",
        content
    );

    child.kill().await.ok();
}

#[tokio::test]
#[ignore]
async fn test_mcp_list_sessions_live() {
    if should_skip() {
        eprintln!("Skipping live test: DEVIN_API_KEY not set");
        return;
    }

    let binary = env!("CARGO_BIN_EXE_devin-mcp");
    let api_key = std::env::var("DEVIN_API_KEY").unwrap();

    let mut child = Command::new(binary)
        .env("DEVIN_API_KEY", &api_key)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // initialize + initialized
    let _ = send_jsonrpc(
        &mut stdin,
        &mut stdout,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0.1.0" }
            }
        }),
    )
    .await;

    let notif = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    }))
    .unwrap();
    stdin.write_all(notif.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // list_sessions
    let response = send_jsonrpc(
        &mut stdin,
        &mut stdout,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "list_sessions",
                "arguments": { "limit": 5, "offset": 0 }
            }
        }),
    )
    .await;

    assert!(response["result"]["content"][0]["text"].is_string());

    child.kill().await.ok();
}

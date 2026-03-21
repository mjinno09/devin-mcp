//! MCP サーバーのインテグレーションテスト
//!
//! 実際のバイナリを子プロセスとして起動し、
//! JSON-RPC over stdio で通信する E2E テスト。
//! wiremock で Devin API をモックする。

use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

/// MCP サーバーを起動し initialize ハンドシェイクを完了するヘルパー
async fn spawn_and_initialize(
    mock_server: &MockServer,
) -> (
    tokio::process::Child,
    tokio::process::ChildStdin,
    BufReader<tokio::process::ChildStdout>,
) {
    let binary = env!("CARGO_BIN_EXE_devin-mcp");

    let mut child = Command::new(binary)
        .env("DEVIN_API_KEY", "test-api-key")
        .env("DEVIN_API_BASE_URL", mock_server.uri())
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
                "clientInfo": { "name": "test-client", "version": "0.1.0" }
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

    (child, stdin, stdout)
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
async fn test_mcp_create_session() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "session_id": "devin-integ123",
            "status": "running",
            "title": "Integration test session",
            "created_at": "2025-01-01T00:00:00.000000+00:00",
            "updated_at": "2025-01-01T00:00:00.000000+00:00",
            "messages": []
        })))
        .mount(&mock_server)
        .await;

    let (mut child, mut stdin, mut stdout) = spawn_and_initialize(&mock_server).await;

    // tools/call で create_session を呼ぶ
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
            .contains("https://app.devin.ai/sessions/devin-integ123"),
        "Response should contain session URL: {}",
        content
    );

    child.kill().await.ok();
}

#[tokio::test]
async fn test_mcp_list_sessions() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "session_id": "devin-list001",
                    "status": "finished",
                    "title": "List test session",
                    "created_at": "2025-01-01T00:00:00.000000+00:00",
                    "updated_at": "2025-01-01T00:00:00.000000+00:00",
                    "messages": []
                }
            ],
            "total": 1
        })))
        .mount(&mock_server)
        .await;

    let (mut child, mut stdin, mut stdout) = spawn_and_initialize(&mock_server).await;

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

    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("devin-list001"),
        "Response should contain session ID: {}",
        text
    );

    child.kill().await.ok();
}

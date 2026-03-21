use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData, ServerHandler,
};
use serde::Deserialize;

use crate::devin_client::{CreateSessionRequest, DevinClient};

// ============================================================
// Tool パラメータ定義
// ============================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSessionParams {
    /// タスクの説明。Devin に実行させたい作業内容を具体的に記述する
    pub prompt: String,
    /// セッションに付けるタグ（オプション）
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// ACU 上限（オプション、デフォルトなし）
    #[serde(default)]
    pub max_acu_limit: Option<u32>,
    /// Playbook ID（オプション）
    #[serde(default)]
    pub playbook_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetSessionParams {
    /// 取得するセッションの ID
    pub session_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListSessionsParams {
    /// 取得件数（デフォルト: 10）
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// オフセット（デフォルト: 0）
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendMessageParams {
    /// メッセージを送信するセッションの ID
    pub session_id: String,
    /// 送信するメッセージ内容
    pub message: String,
}

// ============================================================
// MCP サーバー本体
// ============================================================

#[derive(Debug, Clone)]
pub struct DevinMcpServer {
    pub(crate) client: DevinClient,
    tool_router: ToolRouter<DevinMcpServer>,
}

fn internal_error(msg: String) -> ErrorData {
    ErrorData::internal_error(msg, None)
}

#[tool_router]
impl DevinMcpServer {
    pub fn new(api_key: String) -> Self {
        Self {
            client: DevinClient::new(api_key),
            tool_router: Self::tool_router(),
        }
    }

    /// Devin にタスクを依頼する。セッション URL を返して終了（ステートレス）
    #[tool(
        description = "Create a new Devin session. Returns the session URL. Devin works asynchronously and results will appear as a GitHub PR."
    )]
    async fn create_session(
        &self,
        Parameters(params): Parameters<CreateSessionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let session = self
            .client
            .create_session(CreateSessionRequest {
                prompt: params.prompt,
                tags: params.tags,
                max_acu_limit: params.max_acu_limit,
                playbook_id: params.playbook_id,
            })
            .await
            .map_err(|e| internal_error(format!("Failed to create session: {}", e)))?;

        let url = format!("https://app.devin.ai/sessions/{}", session.session_id);
        let title = session.title.unwrap_or_default();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Devin session created.\n\
             - Session: {url}\n\
             - ID: {}\n\
             - Title: {title}\n\
             - Status: {}\n\n\
             Devin is working on it. A PR will be created when done.",
            session.session_id, session.status
        ))]))
    }

    /// 既存の Devin セッションの詳細情報を取得する
    #[tool(
        description = "Get details about an existing Devin session, including status, messages, and pull request info."
    )]
    async fn get_session(
        &self,
        Parameters(params): Parameters<GetSessionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let session = self
            .client
            .get_session(&params.session_id)
            .await
            .map_err(|e| internal_error(format!("Failed to get session: {}", e)))?;

        let url = format!("https://app.devin.ai/sessions/{}", session.session_id);
        let pr_info = session
            .pull_request
            .as_ref()
            .map(|pr| format!("\n- PR: {}", pr.url))
            .unwrap_or_default();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Session: {url}\n\
             - ID: {}\n\
             - Status: {}\n\
             - Title: {}{pr_info}",
            session.session_id,
            session.status,
            session.title.as_deref().unwrap_or("(untitled)"),
        ))]))
    }

    /// Devin セッションの一覧を取得する
    #[tool(description = "List Devin sessions. Returns session IDs, statuses, titles, and URLs.")]
    async fn list_sessions(
        &self,
        Parameters(params): Parameters<ListSessionsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let sessions = self
            .client
            .list_sessions(params.limit, params.offset)
            .await
            .map_err(|e| internal_error(format!("Failed to list sessions: {}", e)))?;

        let lines: Vec<String> = sessions
            .items
            .iter()
            .map(|s| {
                format!(
                    "- [{}] {} — https://app.devin.ai/sessions/{}",
                    s.status,
                    s.title.as_deref().unwrap_or("(untitled)"),
                    s.session_id,
                )
            })
            .collect();

        let total = sessions.total.unwrap_or(sessions.items.len() as u64);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Sessions ({} total):\n{}",
            total,
            lines.join("\n")
        ))]))
    }

    /// 既存の Devin セッションにメッセージを送信する（フォローアップ指示やスリープ解除）
    #[tool(
        description = "Send a follow-up message to an existing Devin session. Use this to provide additional instructions or wake a sleeping session."
    )]
    async fn send_message(
        &self,
        Parameters(params): Parameters<SendMessageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let session = self
            .client
            .send_message(&params.session_id, &params.message)
            .await
            .map_err(|e| internal_error(format!("Failed to send message: {}", e)))?;

        let url = format!("https://app.devin.ai/sessions/{}", session.session_id);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Message sent to session.\n\
             - Session: {url}\n\
             - Status: {}",
            session.status
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for DevinMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Devin AI session manager. Tools:\n\
                 - create_session: Create a new Devin task (returns URL, fire-and-forget)\n\
                 - get_session: Check status of an existing session\n\
                 - list_sessions: Browse recent sessions\n\
                 - send_message: Send follow-up instructions to a session",
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_server() -> (MockServer, DevinMcpServer) {
        let mock_server = MockServer::start().await;
        let mut server = DevinMcpServer::new("test-key".to_string());
        server.client.base_url = mock_server.uri();
        (mock_server, server)
    }

    #[tokio::test]
    async fn test_create_session_tool_returns_url() {
        let (mock_server, server) = setup_server().await;

        Mock::given(method("POST"))
            .and(path("/sessions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "session_id": "devin-new123",
                "status": "running",
                "title": "Fix bug #42",
                "created_at": "2025-01-01T00:00:00.000000+00:00",
                "updated_at": "2025-01-01T00:00:00.000000+00:00",
                "messages": []
            })))
            .mount(&mock_server)
            .await;

        let params = CreateSessionParams {
            prompt: "Write unit tests for auth module".to_string(),
            tags: Some(vec!["testing".to_string()]),
            max_acu_limit: Some(3),
            playbook_id: None,
        };

        let result = server.create_session(Parameters(params)).await.unwrap();

        let text_content = result.content[0].as_text().expect("Expected text content");
        let text = &text_content.text;
        assert!(text.contains("https://app.devin.ai/sessions/devin-new123"));
        assert!(text.contains("running"));
    }

    #[tokio::test]
    async fn test_create_session_tool_handles_api_failure() {
        let (mock_server, server) = setup_server().await;

        Mock::given(method("POST"))
            .and(path("/sessions"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let params = CreateSessionParams {
            prompt: "test".to_string(),
            tags: None,
            max_acu_limit: None,
            playbook_id: None,
        };

        let result = server.create_session(Parameters(params)).await;
        assert!(result.is_err());
    }
}

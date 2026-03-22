use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DevinClient {
    client: Client,
    api_key: String,
    pub(crate) base_url: String,
}

#[derive(Serialize)]
pub struct CreateSessionRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_acu_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook_id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Session {
    pub session_id: String,
    pub status: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub messages: Vec<SessionMessage>,
    pub pull_request: Option<PullRequest>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SessionMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct SessionList {
    #[serde(default)]
    pub sessions: Vec<Session>,
}

impl DevinClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.devin.ai/v1".to_string(),
        }
    }

    /// セッション作成
    pub async fn create_session(&self, req: CreateSessionRequest) -> Result<Session> {
        let resp = self
            .client
            .post(format!("{}/sessions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<Session>()
            .await?;
        Ok(resp)
    }

    /// セッション詳細取得
    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let resp = self
            .client
            .get(format!("{}/sessions/{}", self.base_url, session_id))
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<Session>()
            .await?;
        Ok(resp)
    }

    /// セッション一覧
    pub async fn list_sessions(&self, limit: u32, offset: u32) -> Result<SessionList> {
        let resp = self
            .client
            .get(format!("{}/sessions", self.base_url))
            .bearer_auth(&self.api_key)
            .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
            .send()
            .await?
            .error_for_status()?
            .json::<SessionList>()
            .await?;
        Ok(resp)
    }

    /// メッセージ送信
    pub async fn send_message(&self, session_id: &str, message: &str) -> Result<Session> {
        let resp = self
            .client
            .post(format!("{}/sessions/{}", self.base_url, session_id))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({ "message": message }))
            .send()
            .await?
            .error_for_status()?
            .json::<Session>()
            .await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// テスト用のクライアントを MockServer に向けて生成
    async fn setup() -> (MockServer, DevinClient) {
        let mock_server = MockServer::start().await;
        let client = DevinClient {
            client: Client::new(),
            api_key: "test-api-key".to_string(),
            base_url: mock_server.uri(),
        };
        (mock_server, client)
    }

    #[tokio::test]
    async fn test_create_session() {
        let (mock_server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/sessions"))
            .and(header("Authorization", "Bearer test-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "session_id": "devin-abc123",
                "status": "running",
                "title": "Test session",
                "created_at": "2025-01-01T00:00:00.000000+00:00",
                "updated_at": "2025-01-01T00:00:00.000000+00:00",
                "messages": []
            })))
            .mount(&mock_server)
            .await;

        let session = client
            .create_session(CreateSessionRequest {
                prompt: "Fix bug #42".to_string(),
                tags: Some(vec!["test".to_string()]),
                max_acu_limit: None,
                playbook_id: None,
            })
            .await
            .unwrap();

        assert_eq!(session.session_id, "devin-abc123");
        assert_eq!(session.status, "running");
    }

    #[tokio::test]
    async fn test_get_session() {
        let (mock_server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/sessions/devin-abc123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "session_id": "devin-abc123",
                "status": "finished",
                "title": "Test session",
                "created_at": "2025-01-01T00:00:00.000000+00:00",
                "updated_at": "2025-01-01T00:01:00.000000+00:00",
                "messages": [
                    { "type": "initial_user_message", "message": "Fix bug #42" }
                ],
                "pull_request": { "url": "https://github.com/org/repo/pull/99" }
            })))
            .mount(&mock_server)
            .await;

        let session = client.get_session("devin-abc123").await.unwrap();

        assert_eq!(session.status, "finished");
        assert!(session.pull_request.is_some());
        assert_eq!(
            session.pull_request.unwrap().url,
            "https://github.com/org/repo/pull/99"
        );
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let (mock_server, client) = setup().await;

        Mock::given(method("GET"))
            .and(path("/sessions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "sessions": [
                    {
                        "session_id": "devin-001",
                        "status": "finished",
                        "title": "Session 1",
                        "created_at": "2025-01-01T00:00:00.000000+00:00",
                        "updated_at": "2025-01-01T00:00:00.000000+00:00",
                        "messages": []
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let list = client.list_sessions(10, 0).await.unwrap();

        assert_eq!(list.sessions.len(), 1);
        assert_eq!(list.sessions[0].session_id, "devin-001");
    }

    #[tokio::test]
    async fn test_send_message() {
        let (mock_server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/sessions/devin-abc123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "session_id": "devin-abc123",
                "status": "running",
                "title": "Test session",
                "created_at": "2025-01-01T00:00:00.000000+00:00",
                "updated_at": "2025-01-01T00:02:00.000000+00:00",
                "messages": []
            })))
            .mount(&mock_server)
            .await;

        let session = client
            .send_message("devin-abc123", "Also fix CSS")
            .await
            .unwrap();
        assert_eq!(session.status, "running");
    }

    #[tokio::test]
    async fn test_create_session_api_error() {
        let (mock_server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/sessions"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let result = client
            .create_session(CreateSessionRequest {
                prompt: "test".to_string(),
                tags: None,
                max_acu_limit: None,
                playbook_id: None,
            })
            .await;

        assert!(result.is_err());
    }
}

//! Tests for the Agent API endpoints

use crate::server::apis::agent::{AgentChatRequest, AgentChatMessage, AgentStreamChunk, messages_to_prompt};
use crate::server::test_helpers::test_config;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use axum_test::TestServer;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_chat_request_serialization() {
        let request = AgentChatRequest {
            model: "test-model".to_string(),
            messages: vec![
                AgentChatMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                AgentChatMessage {
                    role: "assistant".to_string(),
                    content: "Hi there!".to_string(),
                },
            ],
            system: Some("You are a helpful assistant".to_string()),
            stream: false,
            tools: Some(vec!["tool1".to_string(), "tool2".to_string()]),
            session_id: Some("session-123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AgentChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "test-model");
        assert_eq!(deserialized.messages.len(), 2);
        assert_eq!(deserialized.messages[0].role, "user");
        assert_eq!(deserialized.messages[0].content, "Hello");
        assert_eq!(deserialized.system, Some("You are a helpful assistant".to_string()));
        assert!(!deserialized.stream);
        assert_eq!(deserialized.tools, Some(vec!["tool1".to_string(), "tool2".to_string()]));
        assert_eq!(deserialized.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_agent_chat_request_defaults() {
        let json = r#"{
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}]
        }"#;

        let request: AgentChatRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 1);
        assert!(request.system.is_none());
        assert!(!request.stream);
        assert!(request.tools.is_none());
        assert!(request.session_id.is_none());
    }

    #[test]
    fn test_agent_chat_response_serialization() {
        use crate::server::apis::agent::{AgentChatResponse, AgentUsage};

        let response = AgentChatResponse {
            response: "Hello! How can I help?".to_string(),
            model: "test-model".to_string(),
            usage: AgentUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            session_id: Some("session-123".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AgentChatResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.response, "Hello! How can I help?");
        assert_eq!(deserialized.model, "test-model");
        assert_eq!(deserialized.usage.prompt_tokens, 10);
        assert_eq!(deserialized.usage.completion_tokens, 5);
        assert_eq!(deserialized.usage.total_tokens, 15);
        assert_eq!(deserialized.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_agent_stream_chunk_serialization() {
        let chunk = AgentStreamChunk {
            content: "Hello".to_string(),
            done: false,
        };

        let json = serde_json::to_string(&chunk).unwrap();
        let deserialized: AgentStreamChunk = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, "Hello");
        assert!(!deserialized.done);

        // Test done chunk
        let done_chunk = AgentStreamChunk {
            content: "".to_string(),
            done: true,
        };

        let json = serde_json::to_string(&done_chunk).unwrap();
        let deserialized: AgentStreamChunk = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, "");
        assert!(deserialized.done);
    }

    #[test]
    fn test_messages_to_prompt() {
        let messages = vec![
            AgentChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            AgentChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            AgentChatMessage {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
            AgentChatMessage {
                role: "user".to_string(),
                content: "How are you?".to_string(),
            },
        ];

        let prompt = messages_to_prompt(&messages);

        assert!(prompt.contains("system: You are a helpful assistant"));
        assert!(prompt.contains("user: Hello"));
        assert!(prompt.contains("assistant: Hi there!"));
        assert!(prompt.contains("user: How are you?"));
        assert_eq!(prompt.matches('\n').count(), 3);
    }

    #[test]
    fn test_messages_to_prompt_empty() {
        let messages = vec![];
        let prompt = messages_to_prompt(&messages);
        assert_eq!(prompt, "");
    }

    #[test]
    fn test_messages_to_prompt_single() {
        let messages = vec![AgentChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let prompt = messages_to_prompt(&messages);
        assert_eq!(prompt, "user: Hello");
    }

    #[test]
    fn test_agent_chat_request_with_stream_true() {
        let request = AgentChatRequest {
            model: "test-model".to_string(),
            messages: vec![AgentChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: None,
            stream: true,
            tools: None,
            session_id: None,
        };

        assert!(request.stream);
    }

    #[test]
    fn test_agent_chat_message_serialization() {
        let message = AgentChatMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: AgentChatMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.content, "Test message");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::server::{AppState, rig_provider::RkllmClient, rkllm_runtime::RkllmRuntime};
    use crate::config::Config;
    use std::path::PathBuf;
    use std::sync::Arc;

    async fn create_test_app_state() -> AppState {
        let config = Arc::new(test_config());
        let models_path = PathBuf::from("./mock_models");
        let runtime = RkllmRuntime::new(models_path);
        let rig_client = RkllmClient::new(Arc::new(runtime.clone()));

        AppState {
            runtime,
            config,
            digest_cache: crate::server::DigestCache::default(),
            rig_client,
        }
    }

    #[tokio::test]
    async fn test_agent_chat_endpoint_structure() {
        let app_state = create_test_app_state().await;

        let app = Router::new()
            .route("/api/agent/chat", post(crate::server::apis::agent::agent_chat))
            .with_state(app_state);

        let server = TestServer::new(app);

        let response = server
            .post("/api/agent/chat")
            .json(&json!({
                "model": "test-model",
                "messages": [
                    {"role": "user", "content": "Hello"}
                ],
                "stream": false
            }))
            .await;

        // Should return 500 because model is not loaded
        assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_agent_stream_endpoint_structure() {
        let app_state = create_test_app_state().await;

        let app = Router::new()
            .route("/api/agent/stream", post(crate::server::apis::agent::agent_stream))
            .with_state(app_state);

        let server = TestServer::new(app);

        let response = server
            .post("/api/agent/stream")
            .json(&json!({
                "model": "test-model",
                "messages": [
                    {"role": "user", "content": "Hello"}
                ],
                "stream": true
            }))
            .await;

        // The stream endpoint may return 200 (starts streaming then fails) or 500 (fails immediately)
        // Both are acceptable for this structural test
        assert!(response.status_code() == StatusCode::OK || response.status_code() == StatusCode::INTERNAL_SERVER_ERROR);
    }
}

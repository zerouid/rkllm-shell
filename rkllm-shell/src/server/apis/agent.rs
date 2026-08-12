//! Agent API endpoints for rig-based agents

use axum::{
    extract::{Json, State},
    response::sse::{Event, Sse},
    response::IntoResponse,
};
use futures::Stream;
use rig::agent::Agent;
use rig::completion::Prompt;
use rig::tool::Tool;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use rig::agent::MultiTurnStreamItem;
use rig::message::Text;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

use crate::config::Config;
use crate::error::Result as RkllmResult;
use crate::server::rig_provider::RkllmClient;
use crate::server::AppState;
use crate::server::api_models::ollama::{ChatCompletionRequestMessage, Role};
use crate::terminal::message::write;

/// Request for agent chat
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AgentChatRequest {
    /// Model to use
    pub model: String,
    /// Conversation messages
    pub messages: Vec<AgentChatMessage>,
    /// Optional system prompt
    #[serde(default)]
    pub system: Option<String>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
    /// Tools to enable (by name)
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    /// Session ID for conversation memory
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Chat message for agent API
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AgentChatMessage {
    pub role: String,
    pub content: String,
}

/// Response from agent chat
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AgentChatResponse {
    pub response: String,
    pub model: String,
    pub usage: AgentUsage,
    pub session_id: Option<String>,
}

/// Usage statistics
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AgentUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Streaming response chunk
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStreamChunk {
    pub content: String,
    pub done: bool,
}

/// Agent chat endpoint (non-streaming)
#[utoipa::path(
    post,
    path = "/api/agent/chat",
    request_body = AgentChatRequest,
    responses(
        (status = 200, description = "Agent response", body = AgentChatResponse),
        (status = 500, description = "Server error")
    ),
    tag = "agent"
)]
pub async fn agent_chat(
    State(state): State<AppState>,
    Json(req): Json<AgentChatRequest>,
) -> RkllmResult<impl IntoResponse> {
    let model_name = req.model.clone();
    let session_id = req.session_id.clone();

    // Build agent with preamble
    let mut agent_builder = state.rig_client.agent(&model_name);
    
    if let Some(system) = &req.system {
        agent_builder = agent_builder.preamble(system);
    }

    // TODO: Add tools based on req.tools
    // For now, add default tools
    // agent_builder = agent_builder.tool(...);

    let agent = agent_builder.build();

    // Convert messages to rig format
    let prompt = messages_to_prompt(&req.messages);

    // Execute agent
    let response = agent.prompt(&prompt).await
        .map_err(|e| crate::error::Error::Server(format!("Agent error: {}", e)))?;

    Ok(Json(AgentChatResponse {
        response,
        model: model_name,
        usage: AgentUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
        session_id,
    }))
}

/// Agent stream endpoint (SSE)
#[utoipa::path(
    post,
    path = "/api/agent/stream",
    request_body = AgentChatRequest,
    responses(
        (status = 200, description = "SSE stream of agent response"),
        (status = 500, description = "Server error")
    ),
    tag = "agent"
)]
pub async fn agent_stream(
    State(state): State<AppState>,
    Json(req): Json<AgentChatRequest>,
) -> RkllmResult<Sse<Pin<Box<dyn Stream<Item = std::result::Result<Event, Infallible>> + Send>>>> {
    let model_name = req.model.clone();
    let session_id = req.session_id.clone();

    // Build agent
    let mut agent_builder = state.rig_client.agent(&model_name);
    
    if let Some(system) = &req.system {
        agent_builder = agent_builder.preamble(system);
    }

    let agent = agent_builder.build();

    // Convert messages to prompt
    let prompt = messages_to_prompt(&req.messages);

    // Create streaming response
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    tokio::spawn(async move {
        // Stream the agent response
        let mut stream = agent.stream_prompt(&prompt).await;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                    let content = text.text;
                    let chunk_json = serde_json::to_string(&AgentStreamChunk {
                        content,
                        done: false,
                    }).unwrap();
                    let event = Event::default().data(chunk_json);
                    if tx.send(Ok(event)).is_err() {
                        break;
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning))) => {
                    let content = reasoning.display_text();
                    let chunk_json = serde_json::to_string(&AgentStreamChunk {
                        content,
                        done: false,
                    }).unwrap();
                    let event = Event::default().data(chunk_json);
                    if tx.send(Ok(event)).is_err() {
                        break;
                    }
                }
                Ok(MultiTurnStreamItem::FinalResponse(_)) => {
                    // Final response received
                }
                Ok(MultiTurnStreamItem::ModelTurnRetried { turn: _ }) => {
                    // Model turn retried, continue
                }
                Ok(_) => {
                    // Other stream items (tool calls, etc.) - ignore for now
                }
                Err(e) => {
                    write::error(format!("Stream error: {}", e)).ok();
                    break;
                }
            }
        }
        // Send done event
        let done_chunk = serde_json::to_string(&AgentStreamChunk {
            content: String::new(),
            done: true,
        }).unwrap();
        let event = Event::default().data(done_chunk);
        let _ = tx.send(Ok(event));
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    Ok(Sse::new(Box::pin(stream)))
}

/// Convert messages to a single prompt string
pub fn messages_to_prompt(messages: &[AgentChatMessage]) -> String {
    messages.iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("
")
}


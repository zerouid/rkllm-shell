//! Translation layer between Ollama and OpenAI request/response types

use chrono::Utc;
use std::time::Duration;

use crate::server::api_models::{
    ollama::{ChatCompletionRequest as OllamaChatRequest, ChatCompletionRequestMessage as OllamaMessage, ChatCompletionResponse as OllamaChatResponse, GenerateRequest as OllamaGenerateRequest, GenerateResponse as OllamaGenerateResponse, EmbedRequest as OllamaEmbedRequest, EmbedResponse as OllamaEmbedResponse, Role},
    openai::{OpenAiChatRequest, OpenAiChatResponse, OpenAiChatChunk, OpenAiStreamChoice, OpenAiDelta, OpenAiChoice, OpenAiUsage, OpenAiMessage},
};

/// Convert OpenAI Chat Request to Ollama Chat Request
impl From<OpenAiChatRequest> for OllamaChatRequest {
    fn from(req: OpenAiChatRequest) -> Self {
        let messages: Vec<OllamaMessage> = req
            .messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: match m.role.as_str() {
                    "system" => Role::System,
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    _ => Role::User,
                },
                content: m.content,
                thunking: None,
                images: None,
            })
            .collect();

        OllamaChatRequest {
            model: req.model,
            messages,
            stream: req.stream,
            temperature: req.temperature,
            top_p: req.top_p,
            max_tokens: req.max_tokens,
            keep_alive: req.keep_alive,
            options: crate::server::defaults::default_model_options(),
        }
    }
}

/// Convert Ollama Chat Response to OpenAI Chat Response
impl From<OllamaChatResponse> for OpenAiChatResponse {
    fn from(resp: OllamaChatResponse) -> Self {
        let completion_id = format!("chatcmpl-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0).abs());
        OpenAiChatResponse {
            id: completion_id,
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: resp.model,
            choices: vec![OpenAiChoice {
                index: 0,
                message: OpenAiMessage {
                    role: format!("{:?}", resp.message.role).to_lowercase(),
                    content: resp.message.content,
                },
                finish_reason: if resp.done { "stop" } else { "length" }.to_string(),
            }],
            usage: OpenAiUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        }
    }
}

/// Convert Ollama Chat Response to OpenAI Streaming Chunk
impl From<OllamaChatResponse> for OpenAiChatChunk {
    fn from(resp: OllamaChatResponse) -> Self {
        let chunk_id = format!("chatcmpl-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0).abs());
        OpenAiChatChunk {
            id: chunk_id,
            object: "chat.completion.chunk".to_string(),
            created: Utc::now().timestamp(),
            model: resp.model,
            choices: vec![OpenAiStreamChoice {
                index: 0,
                delta: OpenAiDelta {
                    role: if resp.done { None } else { Some("assistant".to_string()) },
                    content: if resp.done { None } else { Some(resp.message.content) },
                },
                finish_reason: if resp.done { Some("stop".to_string()) } else { None },
            }],
        }
    }
}

/// Create the final [DONE] sentinel chunk for OpenAI streaming
pub fn openai_done_chunk(model: String, id: String) -> OpenAiChatChunk {
    OpenAiChatChunk {
        id,
        object: "chat.completion.chunk".to_string(),
        created: Utc::now().timestamp(),
        model,
        choices: vec![OpenAiStreamChoice {
            index: 0,
            delta: OpenAiDelta {
                role: None,
                content: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
    }
}

/// Convert Ollama Generate Request to OpenAI-like internal format (for reuse)
impl From<OllamaGenerateRequest> for OpenAiChatRequest {
    fn from(req: OllamaGenerateRequest) -> Self {
        OpenAiChatRequest {
            model: req.model,
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: req.prompt,
            }],
            stream: req.stream,
            temperature: req.temperature,
            top_p: req.top_p,
            max_tokens: req.max_tokens,
            keep_alive: req.keep_alive,
        }
    }
}

/// Convert Ollama Embed Response to OpenAI Embed Response format
/// Note: OpenAI embeddings use a different format; this converts to a compatible structure
pub fn ollama_embed_to_openai(resp: OllamaEmbedResponse) -> serde_json::Value {
    serde_json::json!({
        "object": "list",
        "data": resp.embeddings.into_iter().enumerate().map(|(i, emb)| {
            serde_json::json!({
                "object": "embedding",
                "index": i,
                "embedding": emb,
            })
        }).collect::<Vec<_>>(),
        "model": resp.model,
        "usage": {
            "prompt_tokens": resp.prompt_eval_count,
            "total_tokens": resp.prompt_eval_count,
        }
    })
}

/// Shared role mapping helper
pub fn openai_role_to_ollama(role: &str) -> Role {
    match role {
        "system" => Role::System,
        "user" => Role::User,
        "assistant" => Role::Assistant,
        _ => Role::User,
    }
}

/// Shared duration default
pub fn default_keep_alive() -> Duration {
    std::time::Duration::from_secs(300)
}
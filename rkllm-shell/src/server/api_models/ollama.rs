//! Ollama-compatible API types (used by /api/chat, /api/generate, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

use crate::server::defaults::*;

// ---------------------------------------------------------------------------
// Ollama Chat Completion types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ChatCompletionRequestMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thunking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionRequestMessage>,
    #[serde(default = "default_stream")]
    pub stream: bool,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: Duration,
    #[serde(default = "default_model_options")]
    pub options: crate::server::ollama_models::ModelOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ChatCompletionResponse {
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub message: ChatCompletionRequestMessage,
    pub done_reason: String,
    pub done: bool,
}

// ---------------------------------------------------------------------------
// Ollama Generate types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, ToSchema)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default = "default_stream")]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: Duration,
    #[serde(default = "default_model_options")]
    pub options: crate::server::ollama_models::ModelOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub response: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

// ---------------------------------------------------------------------------
// Ollama Embed types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum EmbedInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmbedRequest {
    pub model: String,
    pub input: EmbedInput,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: Duration,
    #[serde(default = "default_embed_truncation")]
    pub truncate: bool,
    #[serde(default = "default_model_options")]
    pub options: crate::server::ollama_models::ModelOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmbedResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
    pub total_duration: Duration,
    pub load_duration: Duration,
    pub prompt_eval_count: i32,
}

//! OpenAI-compatible request/response types for /v1/chat/completions and /v1/models

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// OpenAI Service Tier
// ---------------------------------------------------------------------------

/// Since this enum's variants do not hold data, we can easily define them as #[repr(C)]
/// which helps with FFI.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub enum ServiceTier {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "flex")]
    Flex,
}

// ---------------------------------------------------------------------------
// OpenAI Chat Completion Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum OpenAiContent {
    Text(String),
    Array(Vec<OpenAiContentPart>),
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(tag = "type")]
pub enum OpenAiContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAiImageUrl },
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: OpenAiContent,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiChatRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
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
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiChoice {
    pub index: u32,
    pub message: OpenAiMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
}

// Streaming types

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiStreamChoice {
    pub index: u32,
    pub delta: OpenAiDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiChatChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAiStreamChoice>,
}

// ---------------------------------------------------------------------------
// OpenAI Models API Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiModel {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OpenAiModelList {
    pub object: String,
    pub data: Vec<OpenAiModel>,
}

impl OpenAiModelList {
    pub fn from_ollama_models(models: &crate::server::api_models::ollama_models::ListResponse) -> Self {
        let data = models
            .models
            .iter()
            .map(|m| OpenAiModel {
                id: m.name.clone(),
                object: "model".to_string(),
                created: m.modified_at.timestamp(),
                owned_by: "rkllm".to_string(),
            })
            .collect();
        Self {
            object: "list".to_string(),
            data,
        }
    }
}

// ---------------------------------------------------------------------------
// Defaults (shared with Ollama types)
// ---------------------------------------------------------------------------

fn default_stream() -> bool {
    false
}

fn default_temperature() -> f32 {
    0.8
}

fn default_top_p() -> f32 {
    0.9
}

fn default_keep_alive() -> Duration {
    std::time::Duration::from_secs(300) // 5 minutes default
}

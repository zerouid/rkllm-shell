//! Ollama Model Management types (used by `/api/pull`, `/api/show`, `/api/tags`, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;

use crate::server::defaults::*;

// ---------------------------------------------------------------------------
// Ollama Model Management types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PullRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ProgressResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DeleteRequest {
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ShowRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
    #[serde(default = "default_model_options")]
    pub options: ModelOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ShowResponse {
    pub license: String,
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
    pub system: String,
    pub details: String,
    #[serde(rename = "modified_at")]
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ListResponse {
    pub models: Vec<ListModelResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ListModelResponse {
    pub name: String,
    pub model: String,
    #[serde(rename = "modified_at")]
    pub modified_at: DateTime<Utc>,
    pub size: i64,
    pub digest: String,
    pub details: ModelDetails,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModelOptions {
    pub num_ctx: i32,
    pub repeat_last_n: i32,
    pub repeat_penalty: f32,
    pub temperature: f32,
    pub seed: i32,
    pub stop: Vec<String>,
    pub num_predict: i32,
    pub top_k: i32,
    pub top_p: f32,
    pub min_p: f32,
}

// Re-export defaults for external use
pub use crate::server::defaults::*;
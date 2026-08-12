//! Runtime abstraction traits for testing
//!
//! This module defines traits that abstract the RKLLM runtime operations,
//! enabling unit testing without requiring actual RKNPU hardware.

use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::error::Result;
use crate::server::rkllm_runtime::CompletionRequest;

/// Error type for runtime operations
#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Failed to load model: {0}")]
    LoadFailed(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Model already loaded: {0}")]
    AlreadyLoaded(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<RuntimeError> for crate::error::Error {
    fn from(e: RuntimeError) -> Self {
        crate::error::Error::Server(e.to_string())
    }
}

/// Information about a loaded model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub key: String,
    pub model_path: String,
    pub quantization: String,
    pub size_bytes: u64,
    pub loaded_at: std::time::SystemTime,
}

/// Trait for model handles that can run inference
#[async_trait]
pub trait ModelHandle: Send + Sync {
    /// Run inference with text-only messages
    fn run_inference(&self, messages: Vec<String>) -> tokio::sync::mpsc::UnboundedReceiver<String>;

    /// Run inference with multimodal input (prompt + images)
    fn run_multimodal_inference(
        &self,
        prompt: String,
        images: Vec<String>,
    ) -> tokio::sync::mpsc::UnboundedReceiver<String>;

    /// Get the keep-alive duration for this model
    fn keep_alive(&self) -> Duration;

    /// Get model info
    fn model_info(&self) -> ModelInfo;
}

/// Trait for the model runtime that manages model loading and caching
#[async_trait]
pub trait ModelRuntime: Send + Sync + 'static {
    /// Get or load a model for the given request
    async fn get_or_load_model(&self, request: &CompletionRequest) -> Result<Arc<dyn ModelHandle>>;

    /// List all currently loaded models
    async fn list_loaded_models(&self) -> Vec<ModelInfo>;

    /// Unload a specific model by key
    async fn unload_model(&self, model_key: &str) -> Result<()>;

    /// Get the models directory path
    fn models_path(&self) -> &Path;

    /// Get the number of loaded models
    fn loaded_model_count(&self) -> usize;

    /// Check if a model is loaded
    fn is_model_loaded(&self, model_key: &str) -> bool;
}
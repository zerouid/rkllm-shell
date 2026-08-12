//! RKLLM rig Provider - Implements rig's CompletionModel for RKLLM runtime

use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt};
use rig::client::CompletionClient;
use rig::completion::{
    CompletionModel, CompletionRequest, CompletionResponse, CompletionError,
    Message, AssistantContent,
    CompletionRequestBuilder, Usage,
};
use rig_core::OneOrMany;
use rig::message::{UserContent, Text};
use rig::streaming::{StreamingCompletionResponse, RawStreamingChoice, PauseControl};
use rig_core::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::server::rkllm_runtime::{RkllmRuntime, CompletionRequest as RkllmRequest};
use crate::server::runtime_trait::{ModelHandle, ModelRuntime};
use crate::error::Result as RkllmResult;

/// Errors specific to RKLLM completion
#[derive(Error, Debug)]
pub enum RkllmCompletionError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Stream error: {0}")]
    StreamError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<RkllmCompletionError> for CompletionError {
    fn from(err: RkllmCompletionError) -> Self {
        CompletionError::ProviderError(err.to_string())
    }
}

/// Configuration for RKLLM completion model
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RkllmCompletionConfig {
    pub model_name: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_tokens: Option<u32>,
    pub keep_alive: Option<Duration>,
    pub system_prompt: Option<String>,
}

impl Default for RkllmCompletionConfig {
    fn default() -> Self {
        Self {
            model_name: "default".to_string(),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            max_tokens: Some(2048),
            keep_alive: Some(Duration::from_secs(300)),
            system_prompt: None,
        }
    }
}

/// Raw response type for RKLLM (empty - we don't have provider-specific raw response)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RkllmResponse;

impl rig::completion::GetTokenUsage for RkllmResponse {
    fn token_usage(&self) -> Usage {
        Usage::new()
    }
}

/// Streaming response type for RKLLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RkllmStreamingResponse {
    pub text: String,
}

impl rig::completion::GetTokenUsage for RkllmStreamingResponse {
    fn token_usage(&self) -> Usage {
        Usage::new()
    }
}

/// RKLLM Completion Model implementing rig's CompletionModel trait
#[derive(Clone)]
pub struct RkllmCompletionModel {
    runtime: Arc<RkllmRuntime>,
    config: RkllmCompletionConfig,
}

impl RkllmCompletionModel {
    pub fn new(runtime: Arc<RkllmRuntime>, config: RkllmCompletionConfig) -> Self {
        Self { runtime, config }
    }

    /// Convert rig messages to a prompt string
    fn messages_to_prompt(&self, messages: Vec<Message>) -> String {
        let mut prompt_parts = Vec::new();

        // Add system prompt if configured
        if let Some(system) = &self.config.system_prompt {
            prompt_parts.push(format!("System: {}", system));
        }

        for msg in messages {
            match msg {
                Message::System { content } => {
                    prompt_parts.push(format!("System: {}", content));
                }
                Message::User { content } => {
                    for item in content {
                        match item {
                            UserContent::Text(text) => {
                                prompt_parts.push(format!("User: {}", text.text));
                            }
                            UserContent::Image(image) => {
                                let size = match &image.data {
                                    rig::message::DocumentSourceKind::Raw(bytes) => bytes.len(),
                                    rig::message::DocumentSourceKind::Base64(s) => s.len(),
                                    rig::message::DocumentSourceKind::Url(url) => url.len(),
                                    rig::message::DocumentSourceKind::FileId(id) => id.len(),
                                    rig::message::DocumentSourceKind::String(s) => s.len(),
                                    rig::message::DocumentSourceKind::Unknown => 0,
                                    _ => 0,
                                };
                                prompt_parts.push(format!("User: [Image: {} bytes]", size));
                            }
                            UserContent::ToolResult(_) => {
                                prompt_parts.push("User: [ToolResult]".to_string());
                            }
                            UserContent::Audio(_) => {
                                prompt_parts.push("User: [Audio]".to_string());
                            }
                            UserContent::Video(_) => {
                                prompt_parts.push("User: [Video]".to_string());
                            }
                            UserContent::Document(_) => {
                                prompt_parts.push("User: [Document]".to_string());
                            }
                        }
                    }
                }
                Message::Assistant { content, id: _ } => {
                    for item in content {
                        match item {
                            AssistantContent::Text(text) => {
                                prompt_parts.push(format!("Assistant: {}", text.text));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        prompt_parts.join("\n")
    }

    async fn get_model_handle(&self) -> RkllmResult<Arc<dyn ModelHandle>> {
        let request = RkllmRequest::Generate(crate::server::api_models::GenerateRequest {
            model: self.config.model_name.clone(),
            prompt: String::new(),
            options: Default::default(),
            keep_alive: self.config.keep_alive.unwrap_or_else(|| Duration::from_secs(300)),
            ..Default::default()
        });
        self.runtime.get_or_load_model(&request).await
    }

    fn build_rkllm_request(&self, prompt: String) -> RkllmRequest {
        RkllmRequest::Generate(crate::server::api_models::GenerateRequest {
            model: self.config.model_name.clone(),
            prompt,
            options: crate::server::api_models::ollama_models::ModelOptions {
                temperature: self.config.temperature.unwrap_or(0.7),
                top_p: self.config.top_p.unwrap_or(0.9),
                top_k: self.config.top_k.unwrap_or(40),
                num_predict: self.config.max_tokens.map(|v| v as i32).unwrap_or(2048),
                ..Default::default()
            },
            keep_alive: self.config.keep_alive.unwrap_or_else(|| Duration::from_secs(300)),
            ..Default::default()
        })
    }
}

impl CompletionModel for RkllmCompletionModel {
    type Response = RkllmResponse;
    type StreamingResponse = RkllmStreamingResponse;
    type Client = RkllmClient;

    fn make(client: &Self::Client, model: impl Into<String>) -> Self {
        let model_name: String = model.into();
        Self::new(
            client.runtime.clone(),
            RkllmCompletionConfig {
                model_name,
                ..Default::default()
            },
        )
    }

    fn completion(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<
        Output = std::result::Result<CompletionResponse<Self::Response>, CompletionError>,
    > + Send {
        let model = self.clone();
        async move {
            let model_handle = model.get_model_handle().await.map_err(|e| CompletionError::ProviderError(e.to_string()))?;

            // Convert rig CompletionRequest to prompt - use chat_history directly
            let prompt = model.messages_to_prompt(request.chat_history.into_iter().collect());

            let receiver = model_handle.run_inference(vec![prompt]);

            // Collect all tokens
            let mut full_response = String::new();
            let mut receiver = receiver;
            while let Some(token) = receiver.recv().await {
                full_response.push_str(&token);
            }

            Ok(CompletionResponse {
                choice: OneOrMany::one(AssistantContent::Text(Text {
                    text: full_response,
                    additional_params: None,
                })),
                usage: Usage::new(),
                raw_response: RkllmResponse,
                message_id: None,
            })
        }
    }

    fn stream(
        &self,
        request: CompletionRequest,
    ) -> impl std::future::Future<
        Output = std::result::Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>,
    > + Send {
        let model = self.clone();
        async move {
            let model_handle = model.get_model_handle().await.map_err(|e| CompletionError::ProviderError(e.to_string()))?;

            // Convert rig CompletionRequest to prompt
            let prompt = model.messages_to_prompt(request.chat_history.into_iter().collect());

            let receiver = model_handle.run_inference(vec![prompt]);
            let stream = UnboundedReceiverStream::new(receiver)
                .map(|token| Ok(RawStreamingChoice::Message(token)));

            let streaming_response = StreamingCompletionResponse::stream(Box::pin(stream));
            Ok(streaming_response)
        }
    }

    fn completion_request(&self, prompt: impl Into<Message>) -> CompletionRequestBuilder<Self> {
        CompletionRequestBuilder::new(self.clone(), prompt.into())
    }

    fn composes_native_output_with_tools(&self) -> bool {
        false
    }
}

/// RKLLM Client for creating completion models and agents
#[derive(Clone)]
pub struct RkllmClient {
    runtime: Arc<RkllmRuntime>,
}

impl RkllmClient {
    pub fn new(runtime: Arc<RkllmRuntime>) -> Self {
        Self { runtime }
    }

    pub fn completion_model(&self, model_name: &str) -> RkllmCompletionModel {
        RkllmCompletionModel::make(self, model_name)
    }

    pub fn completion_model_with_config(&self, config: RkllmCompletionConfig) -> RkllmCompletionModel {
        RkllmCompletionModel::new(self.runtime.clone(), config)
    }

    pub fn agent(&self, model_name: &str) -> rig::agent::AgentBuilder<RkllmCompletionModel> {
        let model = self.completion_model(model_name);
        rig::agent::AgentBuilder::new(model)
    }

    pub fn agent_with_config(&self, config: RkllmCompletionConfig) -> rig::agent::AgentBuilder<RkllmCompletionModel> {
        let model = self.completion_model_with_config(config);
        rig::agent::AgentBuilder::new(model)
    }
}

impl CompletionClient for RkllmClient {
    type CompletionModel = RkllmCompletionModel;
}

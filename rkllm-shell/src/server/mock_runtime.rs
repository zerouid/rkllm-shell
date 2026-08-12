//! Mock runtime implementation for testing
//!
//! This module provides a fully functional mock implementation of the
//! ModelRuntime and ModelHandle traits for unit and integration testing.

use super::runtime_trait::{ModelHandle, ModelInfo, ModelRuntime, RuntimeError};
use crate::error::Result;
use crate::server::rkllm_runtime::CompletionRequest;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

/// Configuration for mock runtime behavior
#[derive(Debug, Clone)]
pub struct MockRuntimeConfig {
    /// Whether model loading should fail
    pub should_fail_load: bool,
    /// Error message when loading fails
    pub load_error_msg: String,
    /// Delay before model loads (simulates loading time)
    pub load_delay: Duration,
    /// Default responses for models
    pub default_responses: Vec<String>,
    /// Whether inference should error
    pub should_error_inference: bool,
    /// Error message for inference errors
    pub inference_error_msg: String,
}

impl Default for MockRuntimeConfig {
    fn default() -> Self {
        Self {
            should_fail_load: false,
            load_error_msg: "Mock load failure".into(),
            load_delay: Duration::from_millis(10),
            default_responses: vec!["Hello! ".into(), "How can I help?".into()],
            should_error_inference: false,
            inference_error_msg: "Mock inference error".into(),
        }
    }
}

/// Mock model entry for tracking loaded models
#[derive(Debug, Clone)]
struct MockModelEntry {
    key: String,
    model_path: String,
    quantization: String,
    size_bytes: u64,
    loaded_at: SystemTime,
    responses: Vec<String>,
    should_error: bool,
    error_msg: String,
}

/// Mock runtime implementation
#[derive(Debug, Clone)]
pub struct MockRuntime {
    config: MockRuntimeConfig,
    models: Arc<Mutex<HashMap<String, MockModelEntry>>>,
    models_path: PathBuf,
}

impl MockRuntime {
    /// Create a new mock runtime with default configuration
    pub fn new() -> Self {
        Self::with_config(MockRuntimeConfig::default())
    }

    /// Create a new mock runtime with custom configuration
    pub fn with_config(config: MockRuntimeConfig) -> Self {
        Self {
            config,
            models: Arc::new(Mutex::new(HashMap::new())),
            models_path: PathBuf::from("./mock_models"),
        }
    }

    /// Create a mock runtime with a specific models path
    pub fn with_models_path(models_path: PathBuf) -> Self {
        let mut runtime = Self::new();
        runtime.models_path = models_path;
        runtime
    }

    /// Add a pre-configured model to the runtime
    pub fn add_model(
        &self,
        key: String,
        model_path: String,
        responses: Vec<String>,
    ) {
        let entry = MockModelEntry {
            key: key.clone(),
            model_path,
            quantization: "W4A16".into(),
            size_bytes: 1024 * 1024 * 500, // 500 MB
            loaded_at: SystemTime::now(),
            responses,
            should_error: false,
            error_msg: String::new(),
        };
        self.models.lock().unwrap().insert(key, entry);
    }

    /// Get the number of loaded models
    pub fn model_count(&self) -> usize {
        self.models.lock().unwrap().len()
    }

    /// Clear all models
    pub fn clear(&self) {
        self.models.lock().unwrap().clear();
    }
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelRuntime for MockRuntime {
    async fn get_or_load_model(&self, request: &CompletionRequest) -> Result<Arc<dyn ModelHandle>> {
        // Simulate load delay
        if self.config.load_delay > Duration::ZERO {
            tokio::time::sleep(self.config.load_delay).await;
        }

        // Check if should fail
        if self.config.should_fail_load {
            return Err(crate::error::Error::Server(
                self.config.load_error_msg.clone(),
            ));
        }

        // Generate model key
        let model_key = Self::generate_model_key(request);

        // Check if already loaded
        {
            let models = self.models.lock().unwrap();
            if let Some(entry) = models.get(&model_key) {
                return Ok(Arc::new(MockModel::from_entry(entry.clone())));
            }
        }

        // Load new model
        let model_path = self.resolve_model_path(request);
        let entry = MockModelEntry {
            key: model_key.clone(),
            model_path: model_path.clone(),
            quantization: detect_quantization(&model_path),
            size_bytes: 1024 * 1024 * 500,
            loaded_at: SystemTime::now(),
            responses: self.config.default_responses.clone(),
            should_error: self.config.should_error_inference,
            error_msg: self.config.inference_error_msg.clone(),
        };

        self.models.lock().unwrap().insert(model_key.clone(), entry.clone());

        Ok(Arc::new(MockModel::from_entry(entry)))
    }

    async fn list_loaded_models(&self) -> Vec<ModelInfo> {
        let models = self.models.lock().unwrap();
        models
            .values()
            .map(|entry| ModelInfo {
                key: entry.key.clone(),
                model_path: entry.model_path.clone(),
                quantization: entry.quantization.clone(),
                size_bytes: entry.size_bytes,
                loaded_at: entry.loaded_at,
            })
            .collect()
    }

    async fn unload_model(&self, model_key: &str) -> Result<()> {
        let mut models = self.models.lock().unwrap();
        if models.remove(model_key).is_some() {
            Ok(())
        } else {
            Err(crate::error::Error::Server(format!(
                "Model not found: {}",
                model_key
            )))
        }
    }

    fn models_path(&self) -> &std::path::Path {
        &self.models_path
    }

    fn loaded_model_count(&self) -> usize {
        self.models.lock().unwrap().len()
    }

    fn is_model_loaded(&self, model_key: &str) -> bool {
        self.models.lock().unwrap().contains_key(model_key)
    }
}

impl MockRuntime {
    /// Generate a model key from a request (matches RkllmRuntime logic)
    fn generate_model_key(request: &CompletionRequest) -> String {
        let (model, options) = match request {
            CompletionRequest::Generate(req) => (&req.model, &req.options),
            CompletionRequest::Chat(req) => (&req.model, &req.options),
        };
        format!(
            "{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
            model,
            options.num_ctx,
            options.repeat_last_n,
            options.repeat_penalty,
            options.temperature,
            options.seed,
            options.num_predict,
            options.top_k,
            options.top_p,
            options.stop.join(","),
            options.min_p
        )
    }

    /// Resolve model path (simplified version)
    fn resolve_model_path(&self, request: &CompletionRequest) -> String {
        let model = match request {
            CompletionRequest::Generate(req) => &req.model,
            CompletionRequest::Chat(req) => &req.model,
        };
        self.models_path
            .join(format!("{}.rkllm", model))
            .to_string_lossy()
            .to_string()
    }
}

/// Mock model handle implementation
#[derive(Debug, Clone)]
pub struct MockModel {
    entry: MockModelEntry,
}

impl MockModel {
    fn from_entry(entry: MockModelEntry) -> Self {
        Self { entry }
    }

    /// Create a mock model with custom responses
    pub fn with_responses(responses: Vec<String>) -> Self {
        let entry = MockModelEntry {
            key: "mock-model".into(),
            model_path: "./mock.rkllm".into(),
            quantization: "W4A16".into(),
            size_bytes: 1024 * 1024 * 500,
            loaded_at: SystemTime::now(),
            responses,
            should_error: false,
            error_msg: String::new(),
        };
        Self { entry }
    }

    /// Create a mock model that errors on inference
    pub fn with_error(error_msg: String) -> Self {
        let entry = MockModelEntry {
            key: "mock-model".into(),
            model_path: "./mock.rkllm".into(),
            quantization: "W4A16".into(),
            size_bytes: 1024 * 1024 * 500,
            loaded_at: SystemTime::now(),
            responses: vec![],
            should_error: true,
            error_msg,
        };
        Self { entry }
    }
}

#[async_trait]
impl ModelHandle for MockModel {
    fn run_inference(&self, _messages: Vec<String>) -> mpsc::UnboundedReceiver<String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let responses = self.entry.responses.clone();
        let should_error = self.entry.should_error;
        let error_msg = self.entry.error_msg.clone();

        tokio::spawn(async move {
            if should_error {
                let _ = tx.send(format!("[ERROR] {}", error_msg));
            } else {
                for chunk in responses {
                    // Small delay to simulate streaming
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    let _ = tx.send(chunk);
                }
            }
            // Drop sender to close channel
        });

        rx
    }

    fn run_multimodal_inference(
        &self,
        _prompt: String,
        _images: Vec<String>,
    ) -> mpsc::UnboundedReceiver<String> {
        // For mock, treat multimodal same as regular inference
        self.run_inference(vec![])
    }

    fn keep_alive(&self) -> Duration {
        Duration::from_secs(300)
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            key: self.entry.key.clone(),
            model_path: self.entry.model_path.clone(),
            quantization: self.entry.quantization.clone(),
            size_bytes: self.entry.size_bytes,
            loaded_at: self.entry.loaded_at,
        }
    }
}

/// Helper function to detect quantization from filename
fn detect_quantization(filename: &str) -> String {
    let upper = filename.to_uppercase();
    for q in &[
        "W4A16", "W8A8", "W4A8", "W8A16", "INT4", "INT8", "FP16", "FP32",
    ] {
        if upper.contains(q) {
            return q.to_string();
        }
    }
    "unknown".to_string()
}

/// Builder for creating mock runtimes with specific configurations
pub struct MockRuntimeBuilder {
    config: MockRuntimeConfig,
    models_path: Option<PathBuf>,
    preloaded_models: Vec<(String, String, Vec<String>)>,
}

impl MockRuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: MockRuntimeConfig::default(),
            models_path: None,
            preloaded_models: vec![],
        }
    }

    pub fn with_load_failure(mut self, should_fail: bool, msg: impl Into<String>) -> Self {
        self.config.should_fail_load = should_fail;
        self.config.load_error_msg = msg.into();
        self
    }

    pub fn with_load_delay(mut self, delay: Duration) -> Self {
        self.config.load_delay = delay;
        self
    }

    pub fn with_default_responses(mut self, responses: Vec<String>) -> Self {
        self.config.default_responses = responses;
        self
    }

    pub fn with_inference_error(mut self, should_error: bool, msg: impl Into<String>) -> Self {
        self.config.should_error_inference = should_error;
        self.config.inference_error_msg = msg.into();
        self
    }

    pub fn with_models_path(mut self, path: PathBuf) -> Self {
        self.models_path = Some(path);
        self
    }

    pub fn with_preloaded_model(
        mut self,
        key: String,
        model_path: String,
        responses: Vec<String>,
    ) -> Self {
        self.preloaded_models.push((key, model_path, responses));
        self
    }

    pub fn build(self) -> MockRuntime {
        let mut runtime = if let Some(path) = self.models_path {
            MockRuntime::with_models_path(path)
        } else {
            MockRuntime::with_config(self.config)
        };

        for (key, model_path, responses) in self.preloaded_models {
            runtime.add_model(key, model_path, responses);
        }

        runtime
    }
}

impl Default for MockRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_runtime_basic() {
        let runtime = MockRuntime::new();
        assert_eq!(runtime.loaded_model_count(), 0);
        assert!(!runtime.is_model_loaded("test"));
    }

    #[tokio::test]
    async fn test_mock_runtime_load_model() {
        let runtime = MockRuntime::new();
        let request = CompletionRequest::Generate(crate::server::api_models::GenerateRequest {
            model: "test-model".into(),
            prompt: "test".into(),
            stream: false,
            system: None,
            template: None,
            context: None,
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: None,
            keep_alive: Duration::from_secs(300),
            options: crate::server::defaults::default_model_options(),
        });

        let model = runtime.get_or_load_model(&request).await.unwrap();
        assert_eq!(runtime.loaded_model_count(), 1);
        assert!(runtime.is_model_loaded(&model.model_info().key));
    }

    #[tokio::test]
    async fn test_mock_runtime_reuse_model() {
        let runtime = MockRuntime::new();
        let request = CompletionRequest::Generate(crate::server::api_models::GenerateRequest {
            model: "test-model".into(),
            prompt: "test".into(),
            stream: false,
            system: None,
            template: None,
            context: None,
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: None,
            keep_alive: Duration::from_secs(300),
            options: crate::server::defaults::default_model_options(),
        });

        let model1 = runtime.get_or_load_model(&request).await.unwrap();
        let model2 = runtime.get_or_load_model(&request).await.unwrap();

        // Should return the same cached model
        assert_eq!(model1.model_info().key, model2.model_info().key);
        assert_eq!(runtime.loaded_model_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_model_inference() {
        let model = MockModel::with_responses(vec!["Hello".into(), " world!".into()]);
        let mut rx = model.run_inference(vec!["test".into()]);

        let mut result = String::new();
        while let Some(chunk) = rx.recv().await {
            result.push_str(&chunk);
        }

        assert_eq!(result, "Hello world!");
    }

    #[tokio::test]
    async fn test_mock_model_inference_error() {
        let model = MockModel::with_error("Something went wrong".into());
        let mut rx = model.run_inference(vec!["test".into()]);

        let mut result = String::new();
        while let Some(chunk) = rx.recv().await {
            result.push_str(&chunk);
        }

        assert!(result.contains("[ERROR]"));
        assert!(result.contains("Something went wrong"));
    }

    #[tokio::test]
    async fn test_mock_runtime_builder() {
        let runtime = MockRuntimeBuilder::new()
            .with_default_responses(vec!["Custom response".into()])
            .with_load_delay(Duration::from_millis(5))
            .with_preloaded_model(
                "preloaded".into(),
                "/path/model.rkllm".into(),
                vec!["Preloaded".into()],
            )
            .build();

        assert_eq!(runtime.loaded_model_count(), 1);
        assert!(runtime.is_model_loaded("preloaded"));
    }

    #[tokio::test]
    async fn test_mock_runtime_unload() {
        let runtime = MockRuntime::new();
        let request = CompletionRequest::Generate(crate::server::api_models::GenerateRequest {
            model: "test-model".into(),
            prompt: "test".into(),
            stream: false,
            system: None,
            template: None,
            context: None,
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: None,
            keep_alive: Duration::from_secs(300),
            options: crate::server::defaults::default_model_options(),
        });

        let model = runtime.get_or_load_model(&request).await.unwrap();
        let key = model.model_info().key.clone();

        assert!(runtime.is_model_loaded(&key));

        runtime.unload_model(&key).await.unwrap();
        assert!(!runtime.is_model_loaded(&key));
        assert_eq!(runtime.loaded_model_count(), 0);
    }

    #[tokio::test]
    async fn test_mock_runtime_list_models() {
        let runtime = MockRuntime::new();
        let request = CompletionRequest::Generate(crate::server::api_models::GenerateRequest {
            model: "model1".into(),
            prompt: "test".into(),
            stream: false,
            system: None,
            template: None,
            context: None,
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: None,
            keep_alive: Duration::from_secs(300),
            options: crate::server::defaults::default_model_options(),
        });

        runtime.get_or_load_model(&request).await.unwrap();

        let models = runtime.list_loaded_models().await;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].key, MockRuntime::generate_model_key(&request));
    }
}
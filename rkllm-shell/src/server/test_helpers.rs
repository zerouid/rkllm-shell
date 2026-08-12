//! Test helpers and utilities
//!
//! This module provides common test utilities, builders, and fixtures
//! for testing the rkllm-shell application.

use crate::server::api_models::{
    ollama::{ChatCompletionRequest, ChatCompletionRequestMessage, GenerateRequest, Role},
    openai::{OpenAiChatRequest, OpenAiMessage, OpenAiContent},
};
use crate::server::defaults::default_model_options;
use std::time::Duration;

/// Builder for creating Ollama chat completion requests
pub struct ChatRequestBuilder {
    request: ChatCompletionRequest,
}

impl ChatRequestBuilder {
    pub fn new(model: &str) -> Self {
        Self {
            request: ChatCompletionRequest {
                model: model.into(),
                messages: vec![],
                stream: false,
                temperature: 0.8,
                top_p: 0.9,
                max_tokens: None,
                keep_alive: Duration::from_secs(300),
                options: default_model_options(),
            },
        }
    }

    pub fn message(mut self, role: Role, content: &str) -> Self {
        self.request.messages.push(ChatCompletionRequestMessage {
            role,
            content: content.into(),
            thunking: None,
            images: None,
        });
        self
    }

    pub fn system(mut self, content: &str) -> Self {
        self.message(Role::System, content)
    }

    pub fn user(mut self, content: &str) -> Self {
        self.message(Role::User, content)
    }

    pub fn assistant(mut self, content: &str) -> Self {
        self.message(Role::Assistant, content)
    }

    pub fn with_image(mut self, base64: &str) -> Self {
        if let Some(last) = self.request.messages.last_mut() {
            last.images = Some(vec![base64.into()]);
        }
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = stream;
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.request.temperature = temp;
        self
    }

    pub fn max_tokens(mut self, tokens: i32) -> Self {
        self.request.max_tokens = Some(tokens);
        self
    }

    pub fn build(self) -> ChatCompletionRequest {
        self.request
    }
}

/// Builder for creating Ollama generate requests
pub struct GenerateRequestBuilder {
    request: GenerateRequest,
}

impl GenerateRequestBuilder {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            request: GenerateRequest {
                model: model.into(),
                prompt: prompt.into(),
                stream: false,
                system: None,
                template: None,
                context: None,
                temperature: 0.8,
                top_p: 0.9,
                max_tokens: None,
                keep_alive: Duration::from_secs(300),
                options: default_model_options(),
            },
        }
    }

    pub fn system(mut self, system: &str) -> Self {
        self.request.system = Some(system.into());
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = stream;
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.request.temperature = temp;
        self
    }

    pub fn max_tokens(mut self, tokens: i32) -> Self {
        self.request.max_tokens = Some(tokens);
        self
    }

    pub fn build(self) -> GenerateRequest {
        self.request
    }
}

/// Builder for creating OpenAI chat requests
pub struct OpenAiChatRequestBuilder {
    request: OpenAiChatRequest,
}

impl OpenAiChatRequestBuilder {
    pub fn new(model: &str) -> Self {
        Self {
            request: OpenAiChatRequest {
                model: model.into(),
                messages: vec![],
                stream: false,
                temperature: 0.8,
                top_p: 0.9,
                max_tokens: None,
                keep_alive: Duration::from_secs(300),
            },
        }
    }

    pub fn message(mut self, role: &str, content: &str) -> Self {
        self.request.messages.push(OpenAiMessage {
            role: role.into(),
            content: OpenAiContent::Text(content.into()),
        });
        self
    }

    pub fn system(mut self, content: &str) -> Self {
        self.message("system", content)
    }

    pub fn user(mut self, content: &str) -> Self {
        self.message("user", content)
    }

    pub fn assistant(mut self, content: &str) -> Self {
        self.message("assistant", content)
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = stream;
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.request.temperature = temp;
        self
    }

    pub fn max_tokens(mut self, tokens: i32) -> Self {
        self.request.max_tokens = Some(tokens);
        self
    }

    pub fn build(self) -> OpenAiChatRequest {
        self.request
    }
}

/// Common test base64 images
pub mod test_images {
    /// 1x1 pixel PNG (base64 encoded)
    pub const TINY_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    /// 1x1 pixel JPEG (base64 encoded)
    pub const TINY_JPEG: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4iLBwoOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozv/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQAAPwDSzyD/2Q==";

    /// Invalid base64
    pub const INVALID_BASE64: &str = "not-valid-base64!!!";

    /// Data URL format PNG
    pub fn data_url_png() -> String {
        format!("data:image/png;base64,{}", TINY_PNG)
    }

    /// Data URL format JPEG
    pub fn data_url_jpeg() -> String {
        format!("data:image/jpeg;base64,{}", TINY_JPEG)
    }
}

/// Assert that a response stream contains expected chunks
pub async fn assert_stream_contains(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    expected: &[&str],
) {
    let mut received = Vec::new();
    while let Some(chunk) = rx.recv().await {
        received.push(chunk);
    }
    let combined = received.join("");
    for exp in expected {
        assert!(
            combined.contains(exp),
            "Expected stream to contain '{}', got: {}",
            exp,
            combined
        );
    }
}

/// Create a temporary directory for testing
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Create a test configuration
pub fn test_config() -> crate::config::Config {
    let dir = temp_dir();
    let mut config = crate::config::Config::default();
    config.dir = dir.path().to_path_buf();
    config.models_path = Some(dir.path().join("models"));
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequestBuilder::new("test-model")
            .system("You are a helpful assistant")
            .user("Hello")
            .assistant("Hi there!")
            .user("How are you?")
            .stream(true)
            .temperature(0.5)
            .build();

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 4);
        assert_eq!(request.messages[0].role, Role::System);
        assert_eq!(request.messages[1].role, Role::User);
        assert_eq!(request.messages[2].role, Role::Assistant);
        assert_eq!(request.messages[3].role, Role::User);
        assert!(request.stream);
        assert_eq!(request.temperature, 0.5);
    }

    #[test]
    fn test_chat_request_builder_with_image() {
        let request = ChatRequestBuilder::new("test-model")
            .user("Look at this image")
            .with_image("base64data")
            .build();

        assert_eq!(request.messages.len(), 1);
        assert!(request.messages[0].images.is_some());
        assert_eq!(request.messages[0].images.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_generate_request_builder() {
        let request = GenerateRequestBuilder::new("test-model", "Once upon a time")
            .system("You are a storyteller")
            .stream(true)
            .temperature(0.9)
            .max_tokens(100)
            .build();

        assert_eq!(request.model, "test-model");
        assert_eq!(request.prompt, "Once upon a time");
        assert_eq!(request.system, Some("You are a storyteller".into()));
        assert!(request.stream);
        assert_eq!(request.temperature, 0.9);
        assert_eq!(request.max_tokens, Some(100));
    }

    #[test]
    fn test_openai_chat_request_builder() {
        let request = OpenAiChatRequestBuilder::new("gpt-3.5-turbo")
            .system("You are helpful")
            .user("Hello")
            .assistant("Hi!")
            .stream(true)
            .build();

        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.messages.len(), 3);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[2].role, "assistant");
        assert!(request.stream);
    }

    #[test]
    fn test_test_images() {
        // Verify the base64 decodes
        let png_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, test_images::TINY_PNG).unwrap();
        assert!(!png_bytes.is_empty());
        assert_eq!(&png_bytes[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG magic

        let jpeg_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, test_images::TINY_JPEG).unwrap();
        assert!(!jpeg_bytes.is_empty());
        assert_eq!(&jpeg_bytes[0..3], &[0xFF, 0xD8, 0xFF]); // JPEG magic
    }
}
//! Test fixtures
//!
//! This module provides pre-configured test fixtures for common test scenarios.

use crate::server::api_models::{
    ollama::{ChatCompletionRequest, ChatCompletionRequestMessage, GenerateRequest, Role},
    openai::{OpenAiChatRequest, OpenAiMessage, OpenAiContent},
};
use crate::server::defaults::default_model_options;
use std::time::Duration;

/// A simple chat request with system and user messages
pub fn simple_chat_request(model: &str) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: model.into(),
        messages: vec![
            ChatCompletionRequestMessage {
                role: Role::System,
                content: "You are a helpful assistant".into(),
                thunking: None,
                images: None,
            },
            ChatCompletionRequestMessage {
                role: Role::User,
                content: "Hello".into(),
                thunking: None,
                images: None,
            },
        ],
        stream: false,
        temperature: 0.8,
        top_p: 0.9,
        max_tokens: None,
        keep_alive: Duration::from_secs(300),
        options: default_model_options(),
    }
}

/// A chat request with an image
pub fn chat_request_with_image(model: &str, image_base64: &str) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: model.into(),
        messages: vec![ChatCompletionRequestMessage {
            role: Role::User,
            content: "Describe this image".into(),
            thunking: None,
            images: Some(vec![image_base64.into()]),
        }],
        stream: false,
        temperature: 0.8,
        top_p: 0.9,
        max_tokens: None,
        keep_alive: Duration::from_secs(300),
        options: default_model_options(),
    }
}

/// A streaming chat request
pub fn streaming_chat_request(model: &str) -> ChatCompletionRequest {
    let mut req = simple_chat_request(model);
    req.stream = true;
    req
}

/// A generate request
pub fn generate_request(model: &str, prompt: &str) -> GenerateRequest {
    GenerateRequest {
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
    }
}

/// A streaming generate request
pub fn streaming_generate_request(model: &str, prompt: &str) -> GenerateRequest {
    let mut req = generate_request(model, prompt);
    req.stream = true;
    req
}

/// An OpenAI chat request
pub fn openai_chat_request(model: &str) -> OpenAiChatRequest {
    OpenAiChatRequest {
        model: model.into(),
        messages: vec![
            OpenAiMessage {
                role: "system".into(),
                content: OpenAiContent::Text("You are a helpful assistant".into()),
            },
            OpenAiMessage {
                role: "user".into(),
                content: OpenAiContent::Text("Hello".into()),
            },
        ],
        stream: false,
        temperature: 0.8,
        top_p: 0.9,
        max_tokens: None,
        keep_alive: Duration::from_secs(300),
    }
}

/// A streaming OpenAI chat request
pub fn streaming_openai_chat_request(model: &str) -> OpenAiChatRequest {
    let mut req = openai_chat_request(model);
    req.stream = true;
    req
}

/// An OpenAI chat request with image
pub fn openai_chat_request_with_image(model: &str, image_base64: &str) -> OpenAiChatRequest {
    OpenAiChatRequest {
        model: model.into(),
        messages: vec![OpenAiMessage {
            role: "user".into(),
            content: OpenAiContent::Text("Describe this image".into()),
        }],
        stream: false,
        temperature: 0.8,
        top_p: 0.9,
        max_tokens: None,
        keep_alive: Duration::from_secs(300),
    }
}

/// Model names for testing
pub mod models {
    pub const LLAMA_3_2_1B: &str = "llama3.2:1b";
    pub const LLAMA_3_2_3B: &str = "llama3.2:3b";
    pub const LLAMA_3_1_8B: &str = "llama3.1:8b";
    pub const QWEN_2_5_0_5B: &str = "qwen2.5:0.5b";
    pub const PHI_3_MINI: &str = "phi3:mini";
    pub const TEST_MODEL: &str = "test-model";
    pub const MULTIMODAL_MODEL: &str = "llava:7b";
}

/// Test image data
pub mod images {
    use super::super::test_helpers::test_images;

    pub const TINY_PNG: &str = test_images::TINY_PNG;
    pub const TINY_JPEG: &str = test_images::TINY_JPEG;
    pub const INVALID_BASE64: &str = test_images::INVALID_BASE64;

    pub fn data_url_png() -> String {
        test_images::data_url_png()
    }

    pub fn data_url_jpeg() -> String {
        test_images::data_url_jpeg()
    }
}

/// Expected response patterns for assertions
pub mod responses {
    pub const HELLO_WORLD: &[&str] = &["Hello", "world"];
    pub const GREETING: &[&str] = &["Hello", "Hi", "Hey"];
    pub const ERROR_PREFIX: &str = "[ERROR]";
    pub const STREAM_CHUNK_PREFIX: &str = "";
}

/// Common test scenarios
pub mod scenarios {
    use super::*;

    /// Basic text generation scenario
    pub fn basic_generation() -> GenerateRequest {
        generate_request(models::TEST_MODEL, "Once upon a time")
    }

    /// Chat with system prompt scenario
    pub fn chat_with_system() -> ChatCompletionRequest {
        simple_chat_request(models::TEST_MODEL)
    }

    /// Multimodal chat scenario
    pub fn multimodal_chat() -> ChatCompletionRequest {
        chat_request_with_image(models::MULTIMODAL_MODEL, images::TINY_PNG)
    }

    /// Streaming generation scenario
    pub fn streaming_generation() -> GenerateRequest {
        streaming_generate_request(models::TEST_MODEL, "Count to 5")
    }

    /// Long context scenario
    pub fn long_context() -> ChatCompletionRequest {
        let mut req = simple_chat_request(models::TEST_MODEL);
        req.messages.push(ChatCompletionRequestMessage {
            role: Role::User,
            content: "x".repeat(10000),
            thunking: None,
            images: None,
        });
        req
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures() {
        let chat = simple_chat_request(models::TEST_MODEL);
        assert_eq!(chat.model, models::TEST_MODEL);
        assert_eq!(chat.messages.len(), 2);

        let gen = generate_request(models::TEST_MODEL, "test prompt");
        assert_eq!(gen.prompt, "test prompt");

        let openai = openai_chat_request(models::TEST_MODEL);
        assert_eq!(openai.model, models::TEST_MODEL);
        assert_eq!(openai.messages.len(), 2);
    }

    #[test]
    fn test_scenarios() {
        let basic = scenarios::basic_generation();
        assert_eq!(basic.model, models::TEST_MODEL);

        let chat = scenarios::chat_with_system();
        assert_eq!(chat.messages.len(), 2);

        let multimodal = scenarios::multimodal_chat();
        assert!(multimodal.messages[0].images.is_some());
    }
}
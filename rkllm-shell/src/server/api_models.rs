//! API Models - Re-exports from separated Ollama and OpenAI modules

pub mod ollama;
pub mod ollama_models;
pub mod openai;
pub mod translate;

// Re-export Ollama types
pub use ollama::{
    ChatCompletionRequest, ChatCompletionRequestMessage, ChatCompletionResponse,
    GenerateRequest, GenerateResponse,
    EmbedRequest, EmbedResponse,
    EmbedInput,
    Role,
};

// Re-export Ollama model management types
pub use ollama_models::{
    PullRequest, ProgressResponse, DeleteRequest, ShowRequest, ShowResponse,
    ListResponse, ListModelResponse, ModelDetails,
    ModelOptions,
};

// Re-export OpenAI types
pub use openai::{
    OpenAiMessage, OpenAiChatRequest, OpenAiChatResponse,
    OpenAiChoice, OpenAiUsage, OpenAiDelta, OpenAiStreamChoice, OpenAiChatChunk,
    OpenAiModel, OpenAiModelList, ServiceTier,
};

// Re-export translation helpers
pub use translate::{
    openai_done_chunk, ollama_embed_to_openai, openai_role_to_ollama, default_keep_alive,
};

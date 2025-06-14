use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::server::defaults::*;

fn default_model_options() -> ModelOptions {
    ModelOptions {
        num_ctx: default_context_window(),
        repeat_last_n: default_repeat_last_n(),
        repeat_penalty: default_repeat_penalty(),
        temperature: default_temperature(),
        seed: default_seed(),
        stop: default_stop(),
        num_predict: default_num_predict(),
        top_k: default_top_k(),
        top_p: default_top_p(),
        min_p: default_min_p(),
    }
}


/// Specifies the latency tier to use for processing the request. This parameter is relevant for customers subscribed to the scale tier service:   - If set to 'auto', and the Project is Scale tier enabled, the system     will utilize scale tier credits until they are exhausted.   - If set to 'auto', and the Project is not Scale tier enabled, the request will be processed using the default service tier with a lower uptime SLA and no latency guarentee.   - If set to 'default', the request will be processed using the default service tier with a lower uptime SLA and no latency guarentee.   - If set to 'flex', the request will be processed with the Flex Processing service tier. [Learn more](/docs/guides/flex-processing).   - When not set, the default behavior is 'auto'.    When this parameter is set, the response body will include the `service_tier` utilized. 
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceTier {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "flex")]
    Flex,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

/// Developer-provided instructions that the model should follow, regardless of messages sent by the user. 
/// With o1 models and newer, `developer` messages replace the previous `system` messages. 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionRequestMessage {
    #[serde(rename = "content")]
    pub content: String,

    /// The role of the messages author, in this case `developer`.
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "role")]
    pub role: Role,

    /// (for thinking models) the model's thinking process
    #[serde(rename = "thinking")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub thunking: Option<String>,

    /// An optional list of images to include in the message (for multimodal models such as llava)
    #[serde(rename = "images")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub images: Option<Vec<Vec<i32>>>,
}




#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelOptions {
    /// Sets the size of the context window used to generate the next token. (Default: 2048)
    #[serde(rename = "num_ctx")]
    #[serde(default = "default_context_window")]
    pub num_ctx: i32,

    /// Sets how far back for the model to look back to prevent repetition. (Default: 64, 0 = disabled, -1 = num_ctx)
    #[serde(rename = "repeat_last_n")]
    #[serde(default = "default_repeat_last_n")]
    pub repeat_last_n: i32,

    /// Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, 
    /// while a lower value (e.g., 0.9) will be more lenient. (Default: 1.1)
    #[serde(rename = "repeat_penalty")]
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: f32,

    /// The temperature of the model. Increasing the temperature will make the model answer more creatively. (Default: 0.8)
    #[serde(rename = "temperature")]
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Sets the random number seed to use for generation. 
    /// Setting this to a specific number will make the model generate the same text for the same prompt. (Default: 0)
    #[serde(rename = "seed")]
    #[serde(default = "default_seed")]
    pub seed: i32,

    /// Sets the stop sequences to use. When this pattern is encountered the LLM will stop generating text and return. 
    /// Multiple stop patterns may be set by specifying multiple separate stop parameters in a modelfile.
    #[serde(rename = "stop")]
    #[serde(default = "default_stop")]
    pub stop: Vec<String>,

    /// Maximum number of tokens to predict when generating text. (Default: -1, infinite generation)
    #[serde(rename = "num_predict")]
    #[serde(default = "default_num_predict")]
    pub num_predict: i32,

    /// Reduces the probability of generating nonsense. A higher value (e.g. 100) 
    /// will give more diverse answers, while a lower value (e.g. 10) will be more conservative. (Default: 40)
    #[serde(rename = "top_k")]
    #[serde(default = "default_top_k")]
    pub top_k: i32,

    /// Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, 
    /// while a lower value (e.g., 0.5) will generate more focused and conservative text. (Default: 0.9)
    #[serde(rename = "top_p")]
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Alternative to the top_p, and aims to ensure a balance of quality and variety. 
    /// The parameter p represents the minimum probability for a token to be considered, 
    /// relative to the probability of the most likely token. For example, with p=0.05 
    /// and the most likely token having a probability of 0.9, logits with a value less than 0.045 are filtered out. (Default: 0.0)
    #[serde(rename = "min_p")]
    #[serde(default = "default_min_p")]
    pub min_p: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionRequest {
    /// A list of messages comprising the conversation so far. Depending on the [model](/docs/models) you use, different message types (modalities) are supported, like [text](/docs/guides/text-generation), [images](/docs/guides/vision), and [audio](/docs/guides/audio). 
    #[serde(rename = "messages")]
    pub messages: Vec<ChatCompletionRequestMessage>,

    #[serde(rename = "model")]
    pub model: String,

    /// An upper bound for the number of tokens that can be generated for a completion, including visible output tokens and [reasoning tokens](/docs/guides/reasoning). 
    #[serde(rename = "options")]
    #[serde(default = "default_model_options")]
    pub options: ModelOptions,

    #[serde(rename = "format")]
    pub format: Option<Box<RawValue>>,

    /// If set to true, the model response data will be streamed to the client as it is generated using [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format). See the [Streaming section below](/docs/api-reference/chat/streaming) for more information, along with the [streaming responses](/docs/guides/streaming-responses) guide for more information on how to handle the streaming events. 
    #[serde(rename = "stream")]
    #[serde(default = "default_stream")]
    pub stream: bool,

    /// If set to true, the model response data will be streamed to the client as it is generated using [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format). See the [Streaming section below](/docs/api-reference/chat/streaming) for more information, along with the [streaming responses](/docs/guides/streaming-responses) guide for more information on how to handle the streaming events. 
    #[serde(rename = "keep_alive")]
    #[serde(default = "default_keep_alive")]
    pub keep_alive: Duration,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerateRequest {
    #[serde(rename = "model")]
    pub model: String,

     #[serde(rename = "prompt")]
    pub prompt: String,

     #[serde(rename = "suffix")]
    pub suffix: String,

     #[serde(rename = "system")]
    pub system: String,

     #[serde(rename = "template")]
    pub template: String,

     #[serde(rename = "context")]
    pub context: Vec<i32>,

    #[serde(rename = "stream")]
    #[serde(default = "default_stream")]
    pub stream: bool,

    #[serde(rename = "raw")]
    #[serde(default = "default_raw")]
    pub raw: bool,

    #[serde(rename = "options")]
    #[serde(default = "default_model_options")]
    pub options: ModelOptions,

    #[serde(rename = "format")]
    pub format: Option<Box<RawValue>>,

    /// KeepAlive controls how long the model will stay loaded in memory following
	/// this request.
    #[serde(rename = "keep_alive")]
    #[serde(default = "default_keep_alive")]
    pub keep_alive: Duration,

    #[serde(rename = "think")]
    #[serde(default = "default_think")]
    pub think: bool,    

    /// An optional list of images to include in the message (for multimodal models such as llava)
    #[serde(rename = "images")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub images: Option<Vec<Vec<i32>>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    #[serde(rename = "model")]
    pub model: String,

    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "message")]
    pub message: ChatCompletionRequestMessage,

    #[serde(rename = "done_reason")]
    pub done_reason: String,    
    
    #[serde(rename = "done")]
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateResponse {
    #[serde(rename = "model")]
    pub model: String,

    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "response")]
    pub response: String,

    #[serde(rename = "thinking")]
    pub thinking: String,

    #[serde(rename = "done_reason")]
    pub done_reason: String,    
    
    #[serde(rename = "done")]
    pub done: bool,

    #[serde(rename = "context")]
    pub context: Vec<i32>,

}

/// PullRequest is the request passed to [Client.Pull].
#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    #[serde(rename = "model")]
	pub model: String,

    #[serde(rename = "stream")]
    #[serde(default = "default_stream")]
    pub stream: bool,
}

/// ProgressResponse is the response passed to progress functions like
/// [PullProgressFunc] and [PushProgressFunc].
#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressResponse {
    #[serde(rename = "status")]
	status:    String,
    #[serde(rename = "digest")]
	digest:    String ,
    #[serde(rename = "total")]
	total:     i64,
    #[serde(rename = "completed")]
	completed: i64 ,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EmbedInput {
    /// A single string to embed.
    String(String),
    /// A list of strings to embed.
    Strings(Vec<String>),
}

/// EmbedRequest is the request passed to [Client.Embed].
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedRequest {
	/// Model is the model name.
    #[serde(rename = "model")]
	pub model: String,

	// Input is the input to embed.
    #[serde(rename = "input")]
	pub input: EmbedInput,

	// KeepAlive controls how long the model will stay loaded in memory following
	// this request.
	#[serde(rename = "keep_alive")]
    #[serde(default = "default_keep_alive")]
    pub keep_alive: Duration,

    #[serde(rename = "truncate")]
     #[serde(default = "default_embed_truncation")]
	pub truncate: bool,

	// Options lists model-specific options.
    #[serde(rename = "options")]
    #[serde(default = "default_model_options")]
    pub options: ModelOptions,

}

/// EmbedResponse is the response from [Client.Embed].
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedResponse {
	#[serde(rename = "model")]
	pub model: String,
    #[serde(rename = "embeddings")]
	pub embeddings: Vec<Vec<f32>>,
    #[serde(rename = "total_duration")]
    pub total_duration: Duration,	
    #[serde(rename = "load_duration")]
    pub load_duration: Duration,	
    #[serde(rename = "prompt_eval_count")]
    pub prompt_eval_count: i32,
}


// DeleteRequest is the request passed to [Client.Delete].
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRequest {
	#[serde(rename = "model")]
	pub model: String,
}

// ShowRequest is the request passed to [Client.Show].
#[derive(Debug, Serialize, Deserialize)]
pub struct  ShowRequest  {
	#[serde(rename = "model")]
	pub model: String,

    #[serde(rename = "system")]
    pub system: String,

    #[serde(rename = "verbose")]
	pub verbose:  bool,

    #[serde(rename = "options")]
    #[serde(default = "default_model_options")]
    pub options: ModelOptions,
}

// ShowResponse is the response returned from [Client.Show].
#[derive(Debug, Serialize, Deserialize)]
pub struct  ShowResponse {
	pub license: String,
	pub modelfile: String,
	pub parameters: String,
	pub template: String,
	pub system : String,
	pub details : String,
	// #[serde(rename = "messages")]
    // pub messages: Vec<ChatCompletionRequestMessage>,
	// pub model_info     map[string]any     `json:"model_info,omitempty"`
	// pub projector_info map[string]any     `json:"projector_info,omitempty"`
	// pub tensors       []Tensor           `json:"tensors,omitempty"`
	// pub capabilities  []model.Capability `json:"capabilities,omitempty"`
    #[serde(rename = "modified_at")]
    pub modified_at: DateTime<Utc>,
}

// ListResponse is the response from [Client.List].
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
	pub models: Vec<ListModelResponse >,
}

// ListModelResponse is a single model description in [ListResponse].
#[derive(Debug, Serialize, Deserialize)]
pub struct ListModelResponse {
	name:    String,
	model:    String,
    #[serde(rename = "modified_at")]
    pub modified_at: DateTime<Utc>,
	pub size :      i64,
	pub digest :    String,
	pub details :   ModelDetails,
}

/// ModelDetails provides details about a model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDetails {
	parent_model :   String,
	format :   String,
	family :    String,
	families :   Vec<String>,
	parameter_size :    String,
	quantization_level:    String,
}
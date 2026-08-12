use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use chrono::Utc;
use futures::stream::{self, StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::server::{
    api_models::{
        ChatCompletionRequest, ChatCompletionRequestMessage, ChatCompletionResponse,
        OpenAiChatChunk, OpenAiChatRequest, OpenAiChatResponse, OpenAiChoice, OpenAiDelta,
        OpenAiMessage, OpenAiStreamChoice, OpenAiUsage, Role,
    },
    api_models::openai::OpenAiContent,
    api_models::translate::extract_content_and_images,
    rkllm_runtime::CompletionRequest,
    AppState,
};

// ---------------------------------------------------------------------------
// Helper: extract images from messages and build prompt
// ---------------------------------------------------------------------------

/// Extract base64-encoded images from messages
fn extract_images(messages: &[ChatCompletionRequestMessage]) -> Vec<String> {
    let mut images = Vec::new();
    for msg in messages {
        if let Some(img_vec) = &msg.images {
            images.extend(img_vec.iter().cloned());
        }
    }
    images
}

/// Build prompt string from messages (without images)
fn build_prompt_from_messages(messages: &[ChatCompletionRequestMessage]) -> String {
    messages
        .iter()
        .map(|m| match m.role {
            Role::System => format!("<|System|>: {}", m.content),
            Role::User => format!("<|User|>: {}", m.content),
            Role::Assistant => format!("<|Assistant|>: {}", m.content),
            _ => m.content.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Ollama  POST /api/chat
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/chat",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, description = "Chat completion response", body = ChatCompletionResponse)
    ),
    tag = "rkllm"
)]
pub async fn generate_chat_completion(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> axum::response::Result<Response> {
    let model = state
        .runtime
        .get_request_model(&CompletionRequest::Chat(request.clone()))
        .await
        .map_err(|e| axum::response::ErrorResponse::from(e))?;

    // Extract images from messages
    let images = extract_images(&request.messages);
    // Build prompt from messages
    let prompt = build_prompt_from_messages(&request.messages);
    let model_name = request.model.clone();
    let stream_mode = request.stream;

    // Use multimodal inference if images are present
    let rx = if images.is_empty() {
        model.run_inference(vec![prompt])
    } else {
        model.run_multimodal_inference(prompt, images)
    };

    if stream_mode {
        // Stream one JSON object per token.
        let token_stream = UnboundedReceiverStream::new(rx);
        let event_stream = token_stream.map(move |token| {
            let chunk = ChatCompletionResponse {
                model: model_name.clone(),
                created_at: Utc::now(),
                message: ChatCompletionRequestMessage {
                    role: Role::Assistant,
                    content: token,
                    thunking: None,
                    images: None,
                },
                done_reason: String::new(),
                done: false,
            };
            let data = serde_json::to_string(&chunk).unwrap_or_default();
            Ok::<Event, std::convert::Infallible>(Event::default().data(data))
        });

        // Append a final "done" event.
        let done_model = request.model.clone();
        let done_event = stream::once(async move {
            let final_chunk = ChatCompletionResponse {
                model: done_model,
                created_at: Utc::now(),
                message: ChatCompletionRequestMessage {
                    role: Role::Assistant,
                    content: String::new(),
                    thunking: None,
                    images: None,
                },
                done_reason: "stop".to_string(),
                done: true,
            };
            let data = serde_json::to_string(&final_chunk).unwrap_or_default();
            Ok::<Event, std::convert::Infallible>(Event::default().data(data))
        });

        let combined = event_stream.chain(done_event);
        Ok(Sse::new(combined).keep_alive(KeepAlive::default()).into_response())
    } else {
        // Buffer all tokens.
        let mut response_text = String::new();
        let mut rx = rx;
        while let Some(token) = rx.recv().await {
            response_text.push_str(&token);
        }
        let response = ChatCompletionResponse {
            model: request.model.clone(),
            created_at: Utc::now(),
            message: ChatCompletionRequestMessage {
                role: Role::Assistant,
                content: response_text,
                thunking: None,
                images: None,
            },
            done_reason: "stop".to_string(),
            done: true,
        };
        Ok(Json(response).into_response())
    }
}

// ---------------------------------------------------------------------------
// OpenAI  POST /v1/chat/completions
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    request_body = OpenAiChatRequest,
    responses(
        (status = 200, description = "OpenAI chat completion response", body = OpenAiChatResponse)
    ),
    tag = "rkllm"
)]
pub async fn openai_chat_completions(
    State(state): State<AppState>,
    Json(request): Json<OpenAiChatRequest>,
) -> axum::response::Result<Response> {
    // Translate OpenAI request → internal ChatCompletionRequest
    use crate::server::api_models::{ChatCompletionRequestMessage as Msg, ModelOptions};
    use crate::server::defaults::*;

    let messages: Vec<Msg> = request
        .messages
        .iter()
        .map(|m| {
            let (content, images) = extract_content_and_images(m.content.clone());
            Msg {
                role: match m.role.as_str() {
                    "system" => Role::System,
                    "assistant" => Role::Assistant,
                    _ => Role::User,
                },
                content,
                thunking: None,
                images,
            }
        })
        .collect();

    let num_predict = request.max_tokens.unwrap_or(default_num_predict());
    let internal = ChatCompletionRequest {
        model: request.model.clone(),
        messages,
        stream: request.stream,
        temperature: request.temperature,
        top_p: request.top_p,
        max_tokens: request.max_tokens,
        keep_alive: request.keep_alive,
        options: ModelOptions {
            temperature: request.temperature,
            top_p: request.top_p,
            num_predict,
            num_ctx: default_context_window(),
            repeat_last_n: default_repeat_last_n(),
            repeat_penalty: default_repeat_penalty(),
            seed: default_seed(),
            stop: default_stop(),
            top_k: default_top_k(),
            min_p: default_min_p(),
        },
    };

    let model = state
        .runtime
        .get_request_model(&CompletionRequest::Chat(internal.clone()))
        .await
        .map_err(|e| axum::response::ErrorResponse::from(e))?;

    let ollama_msgs = build_ollama_messages(&internal.messages);
    let rx = model.run_inference(ollama_msgs);
    let model_name = request.model.clone();
    let stream_mode = request.stream;
    let completion_id = format!("chatcmpl-{}", uuid_simple());

    if stream_mode {
        let id = completion_id.clone();
        let token_stream = UnboundedReceiverStream::new(rx);
        let event_stream = token_stream.map(move |token| {
            let chunk = OpenAiChatChunk {
                id: id.clone(),
                object: "chat.completion.chunk".to_string(),
                created: Utc::now().timestamp(),
                model: model_name.clone(),
                choices: vec![OpenAiStreamChoice {
                    index: 0,
                    delta: OpenAiDelta {
                        role: None,
                        content: Some(token),
                    },
                    finish_reason: None,
                }],
            };
            let data = serde_json::to_string(&chunk).unwrap_or_default();
            Ok::<Event, std::convert::Infallible>(Event::default().data(data))
        });

        let done_id = completion_id.clone();
        let done_model = request.model.clone();
        let done_event = stream::once(async move {
            let chunk = OpenAiChatChunk {
                id: done_id,
                object: "chat.completion.chunk".to_string(),
                created: Utc::now().timestamp(),
                model: done_model,
                choices: vec![OpenAiStreamChoice {
                    index: 0,
                    delta: OpenAiDelta {
                        role: None,
                        content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
            };
            let data = serde_json::to_string(&chunk).unwrap_or_default();
            Ok::<Event, std::convert::Infallible>(Event::default().data(data))
        });

        // OpenAI SSE terminates with "data: [DONE]"
        let sentinel = stream::once(async {
            Ok::<Event, std::convert::Infallible>(Event::default().data("[DONE]"))
        });

        let combined = event_stream.chain(done_event).chain(sentinel);
        Ok(Sse::new(combined).keep_alive(KeepAlive::default()).into_response())
    } else {
        let mut response_text = String::new();
        let mut rx = rx;
        while let Some(token) = rx.recv().await {
            response_text.push_str(&token);
        }
        let response = OpenAiChatResponse {
            id: completion_id,
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model: request.model.clone(),
            choices: vec![OpenAiChoice {
                index: 0,
                message: OpenAiMessage {
                    role: "assistant".to_string(),
                    content: OpenAiContent::Text(response_text),
                },
                finish_reason: "stop".to_string(),
            }],
            usage: OpenAiUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        };
        Ok(Json(response).into_response())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_ollama_messages(messages: &[ChatCompletionRequestMessage]) -> Vec<String> {
    messages
        .iter()
        .map(|m| match m.role {
            Role::System => format!("<|System|>: {}", m.content),
            Role::User => format!("<|User|>: {}", m.content),
            Role::Assistant => format!("<|Assistant|>: {}", m.content),
            _ => m.content.clone(),
        })
        .collect()
}

/// Tiny pseudo-UUID using the current timestamp nanos (no uuid crate needed).
fn uuid_simple() -> String {
    format!("{:x}", Utc::now().timestamp_nanos_opt().unwrap_or(0))
}
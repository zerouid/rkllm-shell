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
    api_models::{GenerateRequest, GenerateResponse},
    rkllm_runtime::CompletionRequest,
    AppState,
};

#[utoipa::path(
    post,
    path = "/api/generate",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Completion response", body = GenerateResponse)
    ),
    tag = "rkllm"
)]
pub async fn generate_completion(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> axum::response::Result<Response> {
    let model = state
        .runtime
        .get_request_model(&CompletionRequest::Generate(request.clone()))
        .await
        .map_err(|e| axum::response::ErrorResponse::from(e))?;

    // Build the prompt string, optionally prepending a system message.
    let mut messages = Vec::new();
    if !request.system.is_empty() {
        messages.push(format!("<|System|>: {}", request.system));
    }
    messages.push(request.prompt.clone());

    let rx = model.run_inference(messages);
    let model_name = request.model.clone();
    let stream_mode = request.stream;

    if stream_mode {
        let token_stream = UnboundedReceiverStream::new(rx);
        let event_stream = token_stream.map(move |token| {
            let chunk = GenerateResponse {
                model: model_name.clone(),
                created_at: Utc::now(),
                response: token,
                thinking: String::new(),
                done_reason: String::new(),
                done: false,
                context: vec![],
            };
            let data = serde_json::to_string(&chunk).unwrap_or_default();
            Ok::<Event, std::convert::Infallible>(Event::default().data(data))
        });

        let done_model = request.model.clone();
        let done_event = stream::once(async move {
            let final_chunk = GenerateResponse {
                model: done_model,
                created_at: Utc::now(),
                response: String::new(),
                thinking: String::new(),
                done_reason: "stop".to_string(),
                done: true,
                context: vec![],
            };
            let data = serde_json::to_string(&final_chunk).unwrap_or_default();
            Ok::<Event, std::convert::Infallible>(Event::default().data(data))
        });

        let combined = event_stream.chain(done_event);
        Ok(Sse::new(combined).keep_alive(KeepAlive::default()).into_response())
    } else {
        let mut response_text = String::new();
        let mut rx = rx;
        while let Some(token) = rx.recv().await {
            response_text.push_str(&token);
        }
        let response = GenerateResponse {
            model: request.model.clone(),
            created_at: Utc::now(),
            response: response_text,
            thinking: String::new(),
            done_reason: "stop".to_string(),
            done: true,
            context: vec![],
        };
        Ok(Json(response).into_response())
    }
}
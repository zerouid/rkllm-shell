use axum::{extract::{State}, Json};

use crate::server::{
    api_models::{
        ChatCompletionRequest, ChatCompletionRequestMessage, ChatCompletionResponse, Role}, 
    rkllm_runtime::{CompletionRequest, RkllmRuntime},
};


pub async fn generate_chat_completion(State(rt): State<RkllmRuntime>, Json(request): Json<ChatCompletionRequest>,) -> axum::response::Result<Json<ChatCompletionResponse>>{
    let r = CompletionRequest::Chat(request.clone());
    let model = rt.get_request_model(&r)
        .expect("Failed to get model");
    let messages =     request.messages.iter().map(|m| {
        match m.role {
            Role::System => format!("<|System|>: {}", m.content),
            Role::User => format!("<|User|>: {}", m.content),
            Role::Assistant => format!("<|Assistant|>: {}", m.content),
            _ => m.content.clone(),
        }
    }).collect();
    let rx = model.run_inference(messages)?;
    let mut response_message = String::new();
    while let Ok(streaming_response) = rx.recv() {
        response_message.push_str(&streaming_response);
    }

    let response = ChatCompletionResponse{
        created_at: chrono::Utc::now(),
        done: true,
        model: request.model.clone(),
        done_reason: "stop".to_string(),
        message: ChatCompletionRequestMessage{
            role: Role::Assistant,
            content: response_message,
            thunking: None,
            images: None,
        }
    };
    Ok(Json(response))
}

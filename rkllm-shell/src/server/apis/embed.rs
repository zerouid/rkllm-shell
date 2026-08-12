use axum::{extract::State, Json};
use std::time::Duration;

use crate::server::{
    api_models::{EmbedRequest, EmbedResponse},
    AppState,
};

#[utoipa::path(
    post,
    path = "/api/embed",
    request_body = EmbedRequest,
    responses(
        (status = 200, description = "Embedding response", body = EmbedResponse)
    ),
    tag = "rkllm"
)]
#[allow(unused_variables)]
pub async fn generate_embeddings(
    State(state): State<AppState>,
    Json(request): Json<EmbedRequest>,
) -> Json<EmbedResponse> {
    // RKLLM does not currently expose an embedding API.
    // Return an empty embedding so the endpoint is functional without panicking.
    Json(EmbedResponse {
        model: request.model,
        embeddings: vec![],
        total_duration: Duration::ZERO,
        load_duration: Duration::ZERO,
        prompt_eval_count: 0,
    })
}
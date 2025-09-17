use axum::{extract::State, Json};

use crate::server::{api_models::{EmbedRequest, EmbedResponse}, rkllm_runtime::RkllmRuntime};

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
pub async fn generate_embeddings(State(rt): State<RkllmRuntime>, Json(request): Json<EmbedRequest>) -> Json<EmbedResponse> {
        unimplemented!()
}

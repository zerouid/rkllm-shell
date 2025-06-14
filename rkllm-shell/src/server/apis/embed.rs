use axum::{extract::State, Json};

use crate::server::{api_models::{EmbedRequest, EmbedResponse}, rkllm_runtime::RkllmRuntime};

#[allow(unused_variables)]
pub async fn generate_embeddings(State(rt): State<RkllmRuntime>, Json(request): Json<EmbedRequest>) -> Json<EmbedResponse> {
        unimplemented!()
}
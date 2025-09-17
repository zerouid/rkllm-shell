use axum::extract::*;

use crate::server::{api_models::{GenerateRequest, GenerateResponse}, rkllm_runtime::RkllmRuntime};

#[utoipa::path(
    post,
    path = "/api/generate",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Completion response", body = GenerateResponse)
    ),
    tag = "rkllm"
)]
#[allow(unused_variables)]
pub async fn generate_completion(State(rt): State<RkllmRuntime>,Json(request): Json<GenerateRequest>) -> Json<GenerateResponse>{
    unimplemented!()
}

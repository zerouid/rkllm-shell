use axum::extract::*;

use crate::server::{api_models::{GenerateRequest, GenerateResponse}, rkllm_runtime::RkllmRuntime};

#[allow(unused_variables)]
pub async fn generate_completion(State(rt): State<RkllmRuntime>,Json(request): Json<GenerateRequest>) -> Json<GenerateResponse>{
    unimplemented!()
}

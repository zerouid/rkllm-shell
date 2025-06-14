#![allow(unused_variables)]
use axum::{extract::State, http::StatusCode, Json};

use crate::server::{api_models::{DeleteRequest, ListResponse, ProgressResponse, PullRequest, ShowRequest, ShowResponse}, rkllm_runtime::RkllmRuntime };


/// Models

/// Delete a fine-tuned model. You must have the Owner role in your organization to delete a model..
///
/// DeleteModel - DELETE /v1/models/{model}
pub async fn delete_model(State(rt): State<RkllmRuntime>, Json(model) : Json<DeleteRequest>) -> StatusCode {
    unimplemented!()
}

/// Lists the currently available models, and provides basic information about each one such as the owner and availability..
///
/// ListModels - GET /v1/models
pub async fn list_local_models(State(rt): State<RkllmRuntime>, ) -> Json<ListResponse> {
    unimplemented!()
}

/// Lists the currently available models, and provides basic information about each one such as the owner and availability..
///
/// ListModels - GET /v1/models
pub async fn list_running_models(State(rt): State<RkllmRuntime>, ) -> Json<ListResponse> {
    unimplemented!()
}


/// Lists the currently available models, and provides basic information about each one such as the owner and availability..
///
/// ListModels - GET /v1/models
pub async fn show_model_info(State(rt): State<RkllmRuntime>, Json(request): Json<ShowRequest>) -> Json<ShowResponse> {
    unimplemented!()
}


/// Retrieves a model instance, providing basic information about the model such as the owner and permissioning..
///
/// RetrieveModel - GET /v1/models/{model}
pub async fn pull_model(Json(model) : Json<PullRequest>) -> Json<ProgressResponse> {
    unimplemented!()
}

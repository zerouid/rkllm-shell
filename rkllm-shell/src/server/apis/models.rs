#![allow(unused_variables)]
use axum::{extract::State, http::StatusCode, Json};

use crate::server::{api_models::{DeleteRequest, ListResponse, ProgressResponse, PullRequest, ShowRequest, ShowResponse}, apis::error::ApiError, rkllm_runtime::RkllmRuntime };


/// Models

/// Delete a fine-tuned model. You must have the Owner role in your organization to delete a model..
///
/// DeleteModel - DELETE /v1/models/{model}
#[utoipa::path(
    delete,
    path = "/v1/models/{model}",
    request_body = DeleteRequest,
    responses(
        (status = 200, description = "Delete model")
    ),
    tag = "models"
)]
pub async fn delete_model(State(rt): State<RkllmRuntime>, Json(model) : Json<DeleteRequest>) -> StatusCode {
    unimplemented!()
}

/// Lists the currently available models, and provides basic information about each one such as the owner and availability..
///
/// ListModels - GET /v1/models
#[utoipa::path(
    get,
    path = "/v1/models",
    responses(
        (status = 200, description = "List of local models", body = ListResponse)
    ),
    tag = "models"
)]
pub async fn list_local_models(State(rt): State<RkllmRuntime>, ) -> Json<ListResponse> {
    unimplemented!()
}

/// Lists the currently available models, and provides basic information about each one such as the owner and availability..
///
/// ListModels - GET /v1/models
#[utoipa::path(
    get,
    path = "/api/ps",
    responses(
        (status = 200, description = "List of running models", body = ListResponse)
    ),
    tag = "models"
)]
pub async fn list_running_models(State(rt): State<RkllmRuntime>, ) -> Json<ListResponse> {
    unimplemented!()
}


/// Lists the currently available models, and provides basic information about each one such as the owner and availability..
///
/// ListModels - GET /v1/models
#[utoipa::path(
    post,
    path = "/v1/models",
    request_body = ShowRequest,
    responses(
        (status = 200, description = "Show model info", body = ShowResponse)
    ),
    tag = "models"
)]
pub async fn show_model_info(State(rt): State<RkllmRuntime>, Json(request): Json<ShowRequest>) -> Json<ShowResponse> {
    unimplemented!()
}


/// Retrieves a model instance, providing basic information about the model such as the owner and permissioning..
///
/// RetrieveModel - GET /v1/models/{model}
#[utoipa::path(
    post,
    path = "/v1/models/{model}",
    request_body = PullRequest,
    responses(
        (status = 200, description = "Pull model", body = ProgressResponse)
    ),
    tag = "models"
)]
pub async fn pull_model(Json(model): Json<PullRequest>) -> Result<Json<ProgressResponse>, ApiError> {
        // Use HF_MODEL_ID env var or default to "gpt2" for demonstration
    let repo_id = model.model.clone();
    let api = hf_hub::api::sync::Api::new()?;
    let repo = api.model(repo_id.clone());
    let files = repo.info()?.siblings;
    let rkllm_files: Vec<_> = files.iter()
        .filter(|f| f.rfilename.ends_with(".rkllm"))
        .collect();
    if rkllm_files.is_empty() {
        return Err(ApiError::ModelNotFound(format!("No .rkllm files found in repo {}", repo_id)));
    }
    for file in rkllm_files {
        let path = repo.get(&file.rfilename)?;
        println!("Downloaded {} to {}", file.rfilename, path.display());
    }
    Ok(Json(ProgressResponse {
        status: "completed".to_string(),
        digest: None,
        total: None,
        completed: None,
    }))
}

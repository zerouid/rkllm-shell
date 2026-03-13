#![allow(unused_variables)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use crate::server::{
    api_models::{
        DeleteRequest, ListModelResponse, ListResponse, ModelDetails, ProgressResponse,
        PullRequest, ShowRequest, ShowResponse,
    },
    apis::error::ApiError,
    AppState,
};

// ---------------------------------------------------------------------------
// Quantization helpers
// ---------------------------------------------------------------------------

/// Detect quantization level from a model filename, e.g.
/// `DeepSeek-R1-Distill-Qwen-1.5B_W4A16_RK3588.rkllm` → `W4A16`
fn detect_quantization(filename: &str) -> String {
    let upper = filename.to_uppercase();
    for q in &[
        "W4A16", "W8A8", "W4A8", "W8A16", "INT4", "INT8", "FP16", "FP32",
    ] {
        if upper.contains(q) {
            return q.to_string();
        }
    }
    "unknown".to_string()
}

// ---------------------------------------------------------------------------
// SHA-256 digest
// ---------------------------------------------------------------------------

fn sha256_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// DELETE /api/delete
// ---------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/v1/models/{model}",
    request_body = DeleteRequest,
    responses(
        (status = 200, description = "Delete model")
    ),
    tag = "models"
)]
pub async fn delete_model(
    State(state): State<AppState>,
    Json(request): Json<DeleteRequest>,
) -> Result<StatusCode, ApiError> {
    let models_dir = state
        .config
        .models_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./data"));
    let model_path = models_dir.join(&request.model);

    if !model_path.exists() {
        return Err(ApiError::ModelNotFound(format!(
            "Model '{}' not found",
            request.model
        )));
    }

    fs::remove_file(&model_path)
        .map_err(|e| ApiError::Internal(format!("Failed to delete model: {}", e)))?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// GET /v1/models/:model
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/v1/models/{model}",
    responses(
        (status = 200, description = "Retrieve model", body = ShowResponse)
    ),
    tag = "models"
)]
pub async fn retrieve_model(
    State(state): State<AppState>,
    Path(model_name): Path<String>,
) -> Result<Json<ShowResponse>, ApiError> {
    use crate::server::api_models::ModelOptions;
    use crate::server::defaults::*;

    let request = ShowRequest {
        model: model_name,
        system: String::new(),
        verbose: false,
        options: ModelOptions {
            num_ctx: default_context_window(),
            repeat_last_n: default_repeat_last_n(),
            repeat_penalty: default_repeat_penalty(),
            temperature: default_temperature(),
            seed: default_seed(),
            stop: default_stop(),
            num_predict: default_num_predict(),
            top_k: default_top_k(),
            top_p: default_top_p(),
            min_p: default_min_p(),
        },
    };
    show_model_info(State(state), Json(request)).await
}

// ---------------------------------------------------------------------------
// GET /v1/models  (also /api/tags)
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/v1/models",
    responses(
        (status = 200, description = "List of local models", body = ListResponse)
    ),
    tag = "models"
)]
pub async fn list_local_models(
    State(state): State<AppState>,
) -> Result<Json<ListResponse>, ApiError> {
    let models_dir = state
        .config
        .models_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./data"));

    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| ApiError::Internal(format!("Failed to create models directory: {}", e)))?;
    }

    let mut models = Vec::new();

    let entries = fs::read_dir(&models_dir)
        .map_err(|e| ApiError::Internal(format!("Failed to read models directory: {}", e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| ApiError::Internal(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("rkllm") {
            let metadata = fs::metadata(&path)
                .map_err(|e| ApiError::Internal(format!("Failed to get file metadata: {}", e)))?;

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                .unwrap_or_else(Utc::now);

            let digest = sha256_file(&path).unwrap_or_default();
            let quantization = detect_quantization(&file_name);

            models.push(ListModelResponse {
                name: file_name.clone(),
                model: file_name.clone(),
                modified_at,
                size: metadata.len() as i64,
                digest,
                details: ModelDetails {
                    parent_model: String::new(),
                    format: "rkllm".to_string(),
                    family: "rkllm".to_string(),
                    families: vec!["rkllm".to_string()],
                    parameter_size: format!(
                        "{:.2} GB",
                        metadata.len() as f64 / 1_073_741_824.0
                    ),
                    quantization_level: quantization,
                },
            });
        }
    }

    Ok(Json(ListResponse { models }))
}

// ---------------------------------------------------------------------------
// GET /api/ps
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/ps",
    responses(
        (status = 200, description = "List of running models", body = ListResponse)
    ),
    tag = "models"
)]
pub async fn list_running_models(
    State(state): State<AppState>,
) -> Result<Json<ListResponse>, ApiError> {
    let running = state.runtime.list_running_models();

    let models = running
        .into_iter()
        .map(|model_key| {
            let quantization = detect_quantization(&model_key);
            ListModelResponse {
                name: model_key.clone(),
                model: model_key,
                modified_at: Utc::now(),
                size: 0,
                digest: String::new(),
                details: ModelDetails {
                    parent_model: String::new(),
                    format: "rkllm".to_string(),
                    family: "rkllm".to_string(),
                    families: vec!["rkllm".to_string()],
                    parameter_size: "unknown".to_string(),
                    quantization_level: quantization,
                },
            }
        })
        .collect();

    Ok(Json(ListResponse { models }))
}

// ---------------------------------------------------------------------------
// POST /api/show
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/show",
    request_body = ShowRequest,
    responses(
        (status = 200, description = "Show model info", body = ShowResponse)
    ),
    tag = "models"
)]
pub async fn show_model_info(
    State(state): State<AppState>,
    Json(request): Json<ShowRequest>,
) -> Result<Json<ShowResponse>, ApiError> {
    let models_dir = state
        .config
        .models_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./data"));
    let model_path = models_dir.join(&request.model);

    if !model_path.exists() {
        return Err(ApiError::ModelNotFound(format!(
            "Model '{}' not found",
            request.model
        )));
    }

    let metadata = fs::metadata(&model_path)
        .map_err(|e| ApiError::Internal(format!("Failed to get file metadata: {}", e)))?;

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
        .unwrap_or_else(Utc::now);

    let size_gb = metadata.len() as f64 / 1_073_741_824.0;
    let quantization = detect_quantization(&request.model);

    let details = format!(
        "Model: {}\nSize: {:.2} GB\nFormat: RKLLM\nQuantization: {}",
        request.model, size_gb, quantization
    );

    Ok(Json(ShowResponse {
        license: "Unknown".to_string(),
        modelfile: format!("FROM {}", request.model),
        parameters: format!(
            "num_ctx: {}\ntemperature: {}\ntop_p: {}\ntop_k: {}\nrepeat_penalty: {}",
            request.options.num_ctx,
            request.options.temperature,
            request.options.top_p,
            request.options.top_k,
            request.options.repeat_penalty
        ),
        template: "<|System|>\n{{ .System }}\n<|User|>\n{{ .Prompt }}\n<|Assistant|>"
            .to_string(),
        system: request.system,
        details,
        modified_at,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/pull
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/pull",
    request_body = PullRequest,
    responses(
        (status = 200, description = "Pull model", body = ProgressResponse)
    ),
    tag = "models"
)]
pub async fn pull_model(
    State(state): State<AppState>,
    Json(model): Json<PullRequest>,
) -> Result<Json<ProgressResponse>, ApiError> {
    let models_dir = state
        .config
        .models_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./data"));

    // Ensure destination directory exists.
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| ApiError::Internal(format!("Failed to create models directory: {}", e)))?;
    }

    let repo_id = model.model.clone();
    let dest_dir = models_dir.clone();

    // Run the blocking HF download in a spawn_blocking thread.
    let result = tokio::task::spawn_blocking(move || -> Result<(), ApiError> {
        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(repo_id.clone());
        let files = repo.info()?.siblings;

        let rkllm_files: Vec<_> = files
            .iter()
            .filter(|f| f.rfilename.ends_with(".rkllm"))
            .collect();

        if rkllm_files.is_empty() {
            return Err(ApiError::ModelNotFound(format!(
                "No .rkllm files found in repo {}",
                repo_id
            )));
        }

        for file in &rkllm_files {
            // hf_hub downloads to a local cache; copy from cache to our models_dir.
            let cached_path = repo.get(&file.rfilename)?;
            let filename = std::path::Path::new(&file.rfilename)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(&file.rfilename));
            let dest = dest_dir.join(filename);
            if cached_path != dest {
                fs::copy(&cached_path, &dest).map_err(|e| {
                    ApiError::Internal(format!(
                        "Failed to copy {} to {}: {}",
                        cached_path.display(),
                        dest.display(),
                        e
                    ))
                })?;
            }
            println!("Saved {} to {}", file.rfilename, dest.display());
        }
        Ok(())
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Task panic: {}", e)))?;

    result?;

    Ok(Json(ProgressResponse {
        status: "completed".to_string(),
        digest: None,
        total: None,
        completed: None,
    }))
}
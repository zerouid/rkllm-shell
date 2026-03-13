#![allow(unused_variables)]
mod apis;
pub mod api_models;
mod rkllm_runtime;
mod defaults;

use std::path::PathBuf;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    routing::{delete, get, post},
    Router,
};
use apis::{
    chat::generate_chat_completion,
    embed::generate_embeddings,
    generate::generate_completion,
    models::{
        delete_model, list_local_models, list_running_models, pull_model, retrieve_model,
        show_model_info,
    },
};
use owo_colors::OwoColorize;
use rkllm_runtime::RkllmRuntime;
use tokio::{sync::oneshot, time::sleep};

use crate::config::Config;
use crate::error::Result;
use crate::server::api_models::*;
use utoipa::OpenApi;

// ---------------------------------------------------------------------------
// Shared application state — accessible by all axum handlers via State<…>
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub runtime: RkllmRuntime,
    pub config: Arc<Config>,
}

// ---------------------------------------------------------------------------
// OpenAPI spec
// ---------------------------------------------------------------------------

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        apis::chat::generate_chat_completion,
        apis::embed::generate_embeddings,
        apis::generate::generate_completion,
        apis::models::list_local_models,
        apis::models::show_model_info,
        apis::models::delete_model,
        apis::models::list_running_models,
        apis::models::pull_model,
        apis::models::retrieve_model,
        apis::chat::openai_chat_completions,
    ),
    components(
        schemas(
            ChatCompletionRequestMessage,
            ModelOptions,
            ChatCompletionRequest,
            GenerateRequest,
            ChatCompletionResponse,
            GenerateResponse,
            PullRequest,
            ProgressResponse,
            EmbedInput,
            EmbedRequest,
            EmbedResponse,
            DeleteRequest,
            ShowRequest,
            ShowResponse,
            ListResponse,
            ListModelResponse,
            ModelDetails,
            Role,
            ServiceTier,
            OpenAiChatRequest,
            OpenAiChatResponse,
            OpenAiMessage,
            OpenAiChoice,
            OpenAiUsage,
        )
    ),
    tags(
        (name = "rkllm", description = "rkllm API")
    )
)]
pub struct ApiDoc;

// ---------------------------------------------------------------------------
// Server entry-point
// ---------------------------------------------------------------------------

pub async fn run_server(
    base_url: &str,
    config: Arc<Config>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let models_path = config
        .models_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./data"));

    // Ensure the models directory exists.
    if !models_path.exists() {
        std::fs::create_dir_all(&models_path).ok();
    }

    let runtime = RkllmRuntime::new(models_path);
    let state = AppState { runtime, config };

    let openapi = ApiDoc::openapi();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/generate", post(generate_completion))
        .route("/api/chat", post(generate_chat_completion))
        .route("/api/embed", post(generate_embeddings))
        .route("/api/tags", get(list_local_models))
        .route("/api/show", post(show_model_info))
        .route("/api/delete", delete(delete_model))
        .route("/api/ps", get(list_running_models))
        .route("/api/pull", post(pull_model))
        // OpenAI-compatible endpoints
        .route("/v1/models", get(list_local_models))
        .route("/v1/models/{model}", get(retrieve_model))
        .route(
            "/v1/chat/completions",
            post(apis::chat::openai_chat_completions),
        )
        .with_state(state)
        .route("/healthz", get(|| async { "OK" }))
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/docs")
                .url("/openapi.json", openapi),
        );

    let addr = base_url.parse::<SocketAddr>().map_err(|e| {
        crate::error::Error::Server(format!(
            "Invalid address '{}': {}",
            base_url, e
        ))
    })?;
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        crate::error::Error::Server(format!(
            "Failed to bind to address {}: {}. The port may already be in use.",
            addr, e
        ))
    })?;

    use crate::terminal::message::write;
    write::info(format!("Listening on http://{}", addr).green()).ok();

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await
        .map_err(|e| crate::error::Error::Server(format!("Server error: {}", e)))?;

    write::info("Server has been shut down.".green()).ok();
    Ok(())
}

async fn wait_for_server(base_url: &str) -> crate::error::Result<()> {
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if let Ok(response) = client
            .get(&format!("http://{}/healthz", base_url))
            .send()
            .await
        {
            if response.status().is_success() || response.status().is_client_error() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(crate::error::Error::Server(
        "Server failed to start within timeout".to_owned(),
    ))
}

pub async fn start_background_server(
    config: &Config,
) -> Result<(tokio::task::JoinHandle<()>, oneshot::Sender<()>)> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let base_url = config.base_url.clone();
    let base_url_for_server = base_url.clone();
    let config_arc = Arc::new(config.clone());

    use crate::terminal::message::write;
    let server_handle = tokio::spawn(async move {
        if let Err(e) = run_server(&base_url_for_server, config_arc, shutdown_rx).await {
            write::error(format!("Server error: {}", e)).ok();
        }
    });

    wait_for_server(&base_url).await?;
    write::info("Background server started".green()).ok();

    Ok((server_handle, shutdown_tx))
}
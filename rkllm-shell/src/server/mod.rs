#![allow(unused_variables)]
mod apis;
mod api_models;
mod rkllm_runtime;
mod defaults;

use std::{net::SocketAddr, time::Duration};
use axum::{routing::{delete, get, post}, Router};
use apis::{
        chat::generate_chat_completion, 
        embed::generate_embeddings, 
        generate::generate_completion, 
        models::{
            list_local_models, show_model_info, delete_model, list_running_models, pull_model,
        }
    };
use owo_colors::OwoColorize;
use rkllm_runtime::RkllmRuntime;
use tokio::{sync::oneshot, time::sleep};

use crate::{server::api_models::*, terminal::message::write};
use utoipa::OpenApi; // trait for ApiDoc::openapi()
use crate::config::Config;
use crate::error::Result;

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
            ServiceTier
        )
    ),
    tags(
        (name = "rkllm", description = "rkllm API")
    )
)]
pub struct ApiDoc;

pub async fn run_server(base_url: &str, shutdown_rx: oneshot::Receiver<()>) {

    let llm_rt = RkllmRuntime::new();
    // let shutdown_signal = async {
    //     #[cfg(unix)]
    //     signal::unix::signal(signal::unix::SignalKind::terminate())
    //         .expect("failed to install signal handler")
    //         .recv()
    //         .await;

    //     signal::ctrl_c()
    //         .await
    //         .expect("failed to install Ctrl+C handler");
    // };

    // // Define your cleanup function
    // async fn cleanup() {
    //     println!("Performing cleanup actions...");
    //     // Add your cleanup logic here
    //     println!("Cleanup complete.");
    // }

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
        .with_state(llm_rt)
        .route("/api/pull", post(pull_model))
        .route("/healthz", axum::routing::get(|| async { "OK" }))
        .merge(utoipa_swagger_ui::SwaggerUi::new("/docs").url("/openapi.json", openapi));
    let addr = base_url.parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    write::info(format!("Listening on http://{}", addr).green()).ok();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await
        .expect("Failed to start server");
    write::info("Server has been shut down.".green()).ok();
}

async fn wait_for_server(base_url: &str) -> crate::error::Result<()> {
    let client = reqwest::Client::new();

    for _ in 0..50 {
        if let Ok(response) = client.get(&format!("http://{}/healthz", base_url)).send().await {
            if response.status().is_success() || response.status().is_client_error() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    Err(crate::error::Error::Server("Server failed to start within timeout".to_owned()))
}

pub async fn start_background_server(config: &Config) -> Result<(tokio::task::JoinHandle<()>, oneshot::Sender<()>)> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let base_url = config.base_url.clone();
    let base_url_for_server = base_url.clone();
    let server_handle = tokio::spawn(async move {
        run_server(&base_url_for_server, shutdown_rx).await;
    });

    wait_for_server(&base_url).await?;
    write::info("Background server started".green()).ok();

    Ok((server_handle, shutdown_tx))
}
#![allow(unused_variables)]
mod apis;
mod api_models;
mod rkllm_runtime;
mod defaults;

use std::{net::SocketAddr};
use axum::{routing::{delete, get, post}, Router};
use apis::{
        chat::generate_chat_completion, 
        embed::generate_embeddings, 
        generate::generate_completion, 
        models::{
            list_local_models, show_model_info, delete_model, list_running_models, pull_model,
        }
    };
use rkllm_runtime::RkllmRuntime;
use tokio::signal;

use crate::{commands::serve::Args, config::Config};

#[tokio::main]
pub async fn run_server(config: &Config, options: &Args) {

    let llm_rt = RkllmRuntime::new();
    let shutdown_signal = async {
        #[cfg(unix)]
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;

        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // Define your cleanup function
    async fn cleanup() {
        println!("Performing cleanup actions...");
        // Add your cleanup logic here
        println!("Cleanup complete.");
    }

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
        .route("/health", axum::routing::get(|| async { "OK" }));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = shutdown_signal => {
                    println!("Received shutdown signal.");
                    cleanup().await; // Execute cleanup before exiting
                },
            }
        })
        .await
        .expect("Failed to start server");
    println!("Listening on http://{}", addr);
}

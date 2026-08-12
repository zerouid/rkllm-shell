use clap::Parser;
use tokio::sync::oneshot;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// Start the rkllm server
#[derive(Default, Parser)]
pub struct Args {
}

async fn is_server_running(base_url: &str) -> bool {
    let client = reqwest::Client::new();
    if let Ok(response) = client
        .get(&format!("http://{}/healthz", base_url))
        .timeout(std::time::Duration::from_millis(500))
        .send()
        .await
    {
        response.status().is_success()
    } else {
        false
    }
}

pub async fn run(config: &Config, _options: &Args) -> Result<()> {
    let base_url = config.base_url.clone();
    
    // Check if server is already running
    if is_server_running(&base_url).await {
        write::info(format!(
            "Server is already running on http://{}",
            base_url
        ).green())?;
        write::info("Use 'rkllm-shell stop' to stop the server, or connect to the existing instance.".yellow())?;
        return Ok(());
    }
    
    write::info(format!(
        "Starting server with models path '{:?}'...",
        config.models_path
    ).yellow())?;

    // Keep the shutdown sender alive to prevent immediate shutdown.
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    
    // Wait for Ctrl+C or SIGTERM to trigger shutdown.
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = shutdown_tx.send(());
    });

    crate::server::run_server(&base_url, std::sync::Arc::new(config.clone()), shutdown_rx).await?;
    write::info("server shut down".green())?;
    Ok(())
}

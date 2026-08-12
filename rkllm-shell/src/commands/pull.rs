use clap::Parser;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// Pull a model from Hugging Face Hub
#[derive(Default, Parser)]
pub struct Args {
    /// Model repository id (e.g. `username/model-name`)
    #[clap(name = "model")]
    pub model: String,
}

pub async fn run(config: &Config, options: &Args) -> Result<()> {
    if options.model.is_empty() {
        write::error("Usage: pull <repo-id>".red())?;
        return Ok(());
    }

    write::info(format!("Pulling model '{}'...", options.model).green())?;

    let url = format!("http://{}/api/pull", config.base_url);
    // Server expects "name" field (Ollama API compatibility)
    let body = serde_json::json!({ "name": options.model });

    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::error::Error::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        write::error(format!("Pull failed ({}): {}", status, text).red())?;
        return Ok(());
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| crate::error::Error::Network(e.to_string()))?;

    let status = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    write::info(format!("Pull status: {}", status).green())?;
    Ok(())
}
use clap::Parser;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// List running (loaded) models
#[derive(Default, Parser)]
pub struct Args {}

pub async fn run(config: &Config, _options: &Args) -> Result<()> {
    let url = format!("http://{}/api/ps", config.base_url);
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| crate::error::Error::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        write::error(format!("Server returned {}: {}", status, body).red())?;
        return Ok(());
    }

    let list: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| crate::error::Error::Network(e.to_string()))?;

    let models = list
        .get("models")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    if models.is_empty() {
        write::info("No models currently loaded.".green())?;
        return Ok(());
    }

    println!("{:<50}  {}", "NAME", "QUANTIZATION");

    for m in &models {
        let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("-");
        let quant = m
            .pointer("/details/quantization_level")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        println!("{:<50}  {}", name, quant);
    }

    Ok(())
}
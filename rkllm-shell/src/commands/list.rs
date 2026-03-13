use clap::Parser;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// List local models
#[derive(Default, Parser)]
pub struct Args {}

pub async fn run(config: &Config, _options: &Args) -> Result<()> {
    let url = format!("http://{}/api/tags", config.base_url);
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
        write::info("No local models found.".green())?;
        return Ok(());
    }

    // Print header
    println!("{:<50} {:>12}  {}", "NAME", "SIZE", "QUANTIZATION");

    for m in &models {
        let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("-");
        let size = m.get("size").and_then(|v| v.as_i64()).unwrap_or(0);
        let quant = m
            .pointer("/details/quantization_level")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        println!(
            "{:<50} {:>12}  {}",
            name,
            format_bytes(size),
            quant
        );
    }

    Ok(())
}

fn format_bytes(bytes: i64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{} B", bytes)
    }
}
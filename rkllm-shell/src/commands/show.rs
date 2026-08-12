use clap::Parser;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// Show information for a model
#[derive(Default, Parser)]
pub struct Args {
    /// Model filename (e.g. `model.rkllm`)
    #[clap(name = "model")]
    pub model: String,
}

pub async fn run(config: &Config, options: &Args) -> Result<()> {
    if options.model.is_empty() {
        write::error("Usage: show <model>".red())?;
        return Ok(());
    }

    let url = format!("http://{}/api/show", config.base_url);
    let body = serde_json::json!({
        "model":   options.model,
        "system":  "",
        "verbose": false,
        "options": {}
    });

    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::error::Error::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        write::error(format!("Server returned {}: {}", status, text).red())?;
        return Ok(());
    }

    let info: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| crate::error::Error::Network(e.to_string()))?;

    // Pretty-print the key fields.
    let print_field = |label: &str, key: &str| {
        if let Some(val) = info.get(key).and_then(|v| v.as_str()) {
            if !val.is_empty() {
                println!("{}: {}", label, val);
            }
        }
    };

    print_field("Modelfile", "modelfile");
    print_field("Parameters", "parameters");
    print_field("Template", "template");
    print_field("Details", "details");
    print_field("License", "license");

    Ok(())
}
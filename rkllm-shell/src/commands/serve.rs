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

pub async fn run(config: &Config, _options: &Args) -> Result<()> {
    write::info(format!(
        "startting server with models path '{:?}'...",
        config.models_path
    ).yellow())?;
    let base_url = config.base_url.clone();
    let (_shutdown_tx, shutdown_rx) = oneshot::channel();
    crate::server::run_server(&base_url, shutdown_rx).await;
    write::info("server started successfully".green())?;
    Ok(())
}

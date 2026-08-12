use clap::Parser;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// Exit the interactive shell (server keeps running)
#[derive(Default, Parser)]
pub struct Args {}

pub async fn run(_config: &Config, _options: &Args) -> Result<()> {
    write::info("Exiting shell. Server will continue running in the background.".green())?;
    write::info("Use 'rkllm-shell stop' to stop the server.".yellow())?;
    Ok(())
}

use clap::Parser;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
    server::run_server,
};

/// Create something
#[derive(Default, Parser)]
pub struct Args {
}

pub fn run(config: &Config, options: &Args) -> Result<()> {
    write::info(format!(
        "startting server with models path '{:?}'...",
        config.models_path
    ).yellow())?;
    run_server(config, options);
    write::info("server started successfully".green())?;
    Ok(())
}

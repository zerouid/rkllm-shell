use clap::Parser;

use crate::{ 
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// Push a model to a registry
#[derive(Default, Parser)]
pub struct Args {
}

pub async fn run(_config: &Config, _options: &Args) -> Result<()> {
    write::info("executing push command...".green())?;
    Ok(())
}
use clap::Parser;

use crate::{ 
    config::Config,
    error::Result,
    terminal::{color::Colorize, message::write},
};

/// List running models
#[derive(Default, Parser)]
pub struct Args {
}

pub async fn run(_config: &Config, _options: &Args) -> Result<()> {
    write::info("executing ps command...".green())?;
    Ok(())
}
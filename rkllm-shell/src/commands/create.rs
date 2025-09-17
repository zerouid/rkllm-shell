use clap::Parser;

use crate::{ 
    config::Config,
    error::Result, 
    terminal::{color::Colorize, message::write},
};

/// Create a model from a Modelfile
#[derive(Default, Parser)]
pub struct Args {
}

pub async fn run(_config: &Config, _options: &Args) -> Result<()> {
    write::info("executing create command...".green())?;
    Ok(())
}
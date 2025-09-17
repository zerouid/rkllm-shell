use clap::Parser;

use crate::{ 
    config::Config,
    error::Result, 
    terminal::{color::Colorize, message::write},
};

///  List models
#[derive(Default, Parser)]
pub struct Args {
}

pub async fn run(_config: &Config, _options: &Args) -> Result<()> {
    write::info("executing list command...".green())?;
    Ok(())
}
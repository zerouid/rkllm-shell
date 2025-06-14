use std::path::PathBuf;

use clap::{ Parser, Subcommand};

use crate::{
    commands::{self, serve, info},
    config::Config,
    error::Result,
};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Config directory
    #[arg(short, long, value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// Increases verbosity; may be specified up to three times
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Serve(serve::Args),
    Info(info::Args),
}

pub fn route(config: &Config, options: Args) -> Result<()> {
    use Command::{Serve, Info};

    match options.command {
        Serve(options) => commands::serve::run(config, &options),
        Info(_) => commands::info::run(config),
    }
}

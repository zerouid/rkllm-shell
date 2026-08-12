use std::path::PathBuf;

use clap::{ Parser };

use crate::{
    commands::{Command, serve},
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
    pub command: Option<Command>,
}

impl Args {
    pub async fn run_command(self, config: &Config) -> Result<()> {
        match self.command {
            Some(Command::Serve(args)) => {
                // For serve command, run the server directly in foreground
                serve::run(config, &args).await
            }
            Some(command) => {
                let (_server_handle, shutdown_tx) = crate::server::start_background_server(config).await?;
                let tx = if matches!(command, Command::Stop(_)) {
                    Some(shutdown_tx)
                } else {
                    None
                };
                command.run(config, tx).await
            }
            None => {
                let (_server_handle, shutdown_tx) = crate::server::start_background_server(config).await?;
                crate::commands::run_repl(config, shutdown_tx).await
            }
        }
    }
}

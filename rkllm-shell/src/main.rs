mod args;
mod commands;
mod config;
mod error;
mod terminal;
mod logging;
mod server;

use clap::Parser;
use directories::ProjectDirs;

use crate::{args::Args, config::Config, error::Result, terminal::message::write};

// Those constants will be used for:
// https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.from
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_ORGANIZATION: &str = "";
pub const APP_QUALIFIER: &str = "";


struct AppInfo {
    name: String,
    homepage: &'static str,
    version: &'static str,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: APP_NAME.into(),
            homepage: option_env!("CARGO_PKG_HOMEPAGE").unwrap_or(""),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

async fn run() -> Result<()> {
    let mut args = Args::parse();
    let config_dir = args.config_dir.get_or_insert_with(|| {
        ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .expect("cannot retrieve home directory from operating system")
            .config_dir()
            .to_path_buf()
    });

    // Initialize logging
    let _ = logging::init(&args.verbosity);

    let config = Config::load(config_dir)?;
    args.run_command(&config).await
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        write::error(error).expect("cannot write error to stderr");
        std::process::exit(1);
    }
}
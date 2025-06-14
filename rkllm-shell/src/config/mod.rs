use std::{
    fs,
    path::{Path, PathBuf},
};

use config::{File, FileFormat};
use serde::Deserialize;

use crate::error::Result;

const CONFIG_FILE_NAME: &str = "config.toml";

const DEFAULT_CONFIG_FILE_CONTENT: &str = include_str!("default.yaml");

#[derive(Deserialize)]
pub struct Config {
    pub models_path: Option<PathBuf>,

    #[serde(skip)]
    pub dir: PathBuf,
}

impl Config {
    pub fn load(dir: &Path) -> Result<Config> {
        let file = dir.join(CONFIG_FILE_NAME);
        if !file.exists() {
            fs::create_dir_all(dir)?;
            fs::write(&file, DEFAULT_CONFIG_FILE_CONTENT)?;
        }
        let config = config::Config::builder()
            .add_source(File::from_str(
                DEFAULT_CONFIG_FILE_CONTENT,
                FileFormat::Yaml,
            ))
            .add_source(File::from(file).format(FileFormat::Yaml))
            .build()?;
        let mut config: Config = config.try_deserialize()?;
        config.dir = dir.into();
        Ok(config)
    }
}


/*

use std::fs::File;
use std::io::Read;

use serde::Deserialize;
use serde_yaml::from_str;

#[derive(Deserialize, Debug)]
pub struct Config;

impl Config {
    /// Load config using path specified in options
    pub fn load(opts: &crate::cli::Options) -> Result<Config, Box<dyn std::error::Error>> {
        // Read file to string
        let mut raw = String::new();
        let mut f = File::open(&opts.config_path)?;
        f.read_to_string(&mut raw)?;

        // Parse as yaml
        from_str(&raw).map_err(From::from)
    }
}


*/
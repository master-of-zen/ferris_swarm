use std::path::{Path, PathBuf};

use config::{Config, ConfigError, File};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct ClientSettings {
    pub node_addresses: Vec<String>,
    pub encoder_params: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeSettings {
    pub address:  String,
    pub temp_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ProcessingSettings {
    pub segment_duration: f64,
    pub temp_dir:         PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub client:     ClientSettings,
    pub node:       NodeSettings,
    pub processing: ProcessingSettings,
}

impl Settings {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let config = Config::builder().add_source(File::from(path)).build()?;

        config.try_deserialize()
    }

    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder().add_source(File::with_name("config")).build()?;

        debug!("Created config : {:?}", config);
        config.try_deserialize()
    }
}

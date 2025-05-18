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

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")] // Ensures "ffmpeg" or "mkvmerge" in config maps correctly
pub enum ConcatenatorChoice {
    Ffmpeg,
    Mkvmerge,
}

impl Default for ConcatenatorChoice {
    fn default() -> Self {
        ConcatenatorChoice::Ffmpeg // Default to ffmpeg if not specified in
                                   // config
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessingSettings {
    pub segment_duration: f64,
    pub temp_dir:         PathBuf,
    #[serde(default)] // Uses ConcatenatorChoice::default() if missing from config
    pub concatenator: ConcatenatorChoice,
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
        // Attempt to load from "config.toml" or "config.json" etc. in CWD
        // If not found, it will use defaults specified in structs (like for
        // concatenator)
        let config = Config::builder()
            .add_source(File::with_name("config").required(false)) // Make it not required to allow for no config file
            .build()?;

        debug!("Loaded settings configuration: {:?}", config);
        config.try_deserialize()
    }
}

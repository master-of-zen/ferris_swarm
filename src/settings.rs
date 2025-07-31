use std::path::{Path, PathBuf};

use config::{Config, ConfigError, File};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct ClientSettings {
    pub node_addresses: Vec<String>,
    pub encoder_params: Vec<String>,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            node_addresses: vec!["127.0.0.1:50051".to_string()],
            encoder_params: vec!["-c:v".to_string(), "libx264".to_string(), "-crf".to_string(), "23".to_string()],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NodeSettings {
    pub address:  String,
    pub temp_dir: PathBuf,
}

impl Default for NodeSettings {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:50051".to_string(),
            temp_dir: std::env::temp_dir().join("ferris_swarm_node"),
        }
    }
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

impl Default for ProcessingSettings {
    fn default() -> Self {
        Self {
            segment_duration: 10.0, // 10 seconds per segment
            temp_dir: std::env::temp_dir().join("ferris_swarm_processing"),
            concatenator: ConcatenatorChoice::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub client:     ClientSettings,
    #[serde(default)]
    pub node:       NodeSettings,
    #[serde(default)]
    pub processing: ProcessingSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            client: ClientSettings::default(),
            node: NodeSettings::default(),
            processing: ProcessingSettings::default(),
        }
    }
}

impl Settings {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let config = Config::builder().add_source(File::from(path)).build()?;
        config.try_deserialize()
    }

    pub fn new() -> Result<Self, ConfigError> {
        // Attempt to load from "config.toml" or "config.json" etc. in CWD
        // If not found, use defaults
        let config = Config::builder()
            .add_source(File::with_name("config").required(false)) // Make it not required to allow for no config file
            .build()?;

        debug!("Loaded settings configuration: {:?}", config);
        
        // If config is empty (no file found), return defaults
        let settings = match config.try_deserialize::<Settings>() {
            Ok(settings) => {
                debug!("Successfully loaded settings from config file");
                settings
            }
            Err(e) => {
                debug!("No config file found or failed to parse, using defaults: {}", e);
                Settings::default()
            }
        };
        
        Ok(settings)
    }
}

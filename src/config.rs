use std::{fs, path::PathBuf};

use sha2::{Digest, Sha256};
use tracing::{debug, instrument};

use crate::{error::VideoEncodeError, settings::Settings};

/// Configuration for the video encoding system
#[derive(Clone, Debug, Default)]
pub struct TempConfig {
    /// Base temporary directory
    pub temp_dir:      PathBuf,
    pub temp_segments: PathBuf,
    pub temp_encoded:  PathBuf,
}

impl TempConfig {
    #[instrument]
    pub fn new(temp_dir: Option<PathBuf>, input_file: &PathBuf, output_file: &str) -> Self {
        let hash = generate_hash(input_file, output_file);
        let temp_dir = temp_dir.unwrap_or_else(|| PathBuf::from(".").join(hash));

        let temp_segments = temp_dir.join("segments");
        let temp_encoded = temp_dir.join("encoded");

        let config = TempConfig {
            temp_dir,
            temp_segments,
            temp_encoded,
        };

        // Create temporary directory
        std::fs::create_dir_all(&config.temp_dir).expect("Failed to create temporary directory");
        std::fs::create_dir_all(&config.temp_segments)
            .expect("Failed to create temp/segments directory");

        std::fs::create_dir_all(&config.temp_encoded)
            .expect("Failed to create temp/encoded directory");

        debug!(
            "Created Config: temp_dir={:?}, temp_segments={:?}, temp_encoded={:?}",
            config.temp_dir, config.temp_segments, config.temp_encoded
        );

        config
    }

    /// Get the path for storing video segments
    pub fn segment_dir(&self) -> PathBuf {
        self.temp_dir.join("segments")
    }

    /// Get the path for storing encoded chunks
    pub fn encode_dir(&self) -> PathBuf {
        self.temp_dir.join("encoded")
    }

    pub fn delete(self) -> Result<(), VideoEncodeError> {
        // Delete the base temp_dir
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
        }

        Ok(())
    }
}

/// Creates a TempConfig instance
#[instrument]
pub fn create_temp_config(
    settings: &Settings,
    input_file: &PathBuf,
    output_file: &str,
) -> TempConfig {
    TempConfig::new(
        Some(settings.processing.temp_dir.clone()),
        input_file,
        output_file,
    )
}

fn generate_hash(input_file: &PathBuf, output_file: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input_file.to_string_lossy().as_bytes());
    hasher.update(output_file.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..4]) // Use first 4 bytes (8 characters in hex)
}

use std::{fs, path::PathBuf};

use sha2::{Digest, Sha256};
use tracing::{debug, instrument}; // Removed error, info

use ferris_swarm_core::error::VideoEncodeError;
use crate::settings::Settings;

/// Configuration for the video encoding system's temporary files for a specific
/// job.
#[derive(Clone, Debug, Default)]
pub struct JobTempConfig {
    // Renamed from TempConfig for clarity
    /// Base temporary directory for this job
    pub base_dir:        PathBuf, // Renamed from temp_dir
    pub segments_subdir: PathBuf, // Renamed from temp_segments
    pub encoded_subdir:  PathBuf, // Renamed from temp_encoded
}

impl JobTempConfig {
    #[instrument]
    pub fn new(
        processing_temp_base_dir: Option<PathBuf>, // Base dir from settings
        input_file_for_hash: &PathBuf,
        output_file_for_hash: &str,
    ) -> Self {
        let unique_job_hash = generate_hash(input_file_for_hash, output_file_for_hash);
        let base_dir = processing_temp_base_dir
            .unwrap_or_else(std::env::temp_dir) // Default to OS temp if not provided
            .join("ferris_swarm_jobs") // A subdirectory for all jobs
            .join(unique_job_hash);

        let segments_subdir = base_dir.join("segments");
        let encoded_subdir = base_dir.join("encoded_chunks");

        // Create directories
        // Panicking here might be acceptable if these are critical for operation
        fs::create_dir_all(&base_dir).expect("Failed to create job base temporary directory");
        fs::create_dir_all(&segments_subdir).expect("Failed to create job temp/segments directory");
        fs::create_dir_all(&encoded_subdir)
            .expect("Failed to create job temp/encoded_chunks directory");

        let config = JobTempConfig {
            base_dir,
            segments_subdir,
            encoded_subdir,
        };

        debug!(
            "Created JobTempConfig: base_dir={:?}, segments_subdir={:?}, encoded_subdir={:?}",
            config.base_dir, config.segments_subdir, config.encoded_subdir
        );

        config
    }

    /// Get the path for storing video segments
    pub fn segments_dir(&self) -> PathBuf {
        self.segments_subdir.clone()
    }

    /// Get the path for storing encoded chunks (by client, or by node before
    /// sending)
    pub fn encoded_chunks_dir(&self) -> PathBuf {
        self.encoded_subdir.clone()
    }

    /// Deletes the entire base temporary directory for this job.
    pub fn delete_job_temp_dirs(&self) -> Result<(), VideoEncodeError> {
        // Renamed method
        if self.base_dir.exists() {
            debug!("Deleting job temp directory: {:?}", self.base_dir);
            fs::remove_dir_all(&self.base_dir)?;
        }
        Ok(())
    }
}

/// Creates a JobTempConfig instance based on global settings and specific job
/// identifiers.
#[instrument]
pub fn create_job_temp_config(
    settings: &Settings,
    input_file: &PathBuf,
    output_file: &str,
) -> JobTempConfig {
    JobTempConfig::new(
        Some(settings.processing.temp_dir.clone()), // Pass the base temp dir from settings
        input_file,
        output_file,
    )
}

fn generate_hash(input_file: &PathBuf, output_file: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input_file.to_string_lossy().as_bytes());
    hasher.update(output_file.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // Use first 8 bytes (16 characters in hex) for
                              // more uniqueness
}

use std::path::{Path, PathBuf}; // Added Path

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};

use crate::{
    error::VideoEncodeError,
    ffmpeg, // Use the ffmpeg module
};

/// Represents a video chunk for processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub source_path:        PathBuf,
    pub encoded_path:       Option<PathBuf>,
    pub index:              usize,
    pub encoder_parameters: Vec<String>,
}

impl Chunk {
    #[instrument(skip(encoder_parameters))]
    pub fn new(source_path: PathBuf, index: usize, encoder_parameters: Vec<String>) -> Self {
        debug!(
            "Creating new Chunk: index={}, source={:?}",
            index, source_path
        );

        if !source_path.exists() {
            // This check is good, but consider if it should be an error returned rather
            // than panic
            error!("Source path does not exist: {:?}", source_path);
            // For a library, panicking is usually not ideal. Return Result instead.
            // However, keeping original behavior for now.
            panic!(
                "Source path does not exist for Chunk::new: {:?}",
                source_path
            );
        }

        Chunk {
            source_path,
            encoded_path: None,
            index,
            encoder_parameters,
        }
    }

    /// Encodes the chunk using ffmpeg.
    /// The result is a new Chunk instance with `encoded_path` set.
    #[instrument(skip(self))]
    pub fn encode(&self, output_path: PathBuf) -> Result<Chunk, VideoEncodeError> {
        debug!(
            "Encoding chunk {} [logic]: source={:?}, output={:?}, encoder_parameters={:?} ",
            self.index, self.source_path, output_path, self.encoder_parameters
        );

        ffmpeg::encoder::encode_with_ffmpeg(
            &self.source_path,
            &output_path,
            &self.encoder_parameters,
        )?;

        info!(
            "Successfully encoded chunk {} to {:?}",
            self.index, output_path
        );
        Ok(Chunk {
            source_path:        self.source_path.clone(),
            encoded_path:       Some(output_path),
            index:              self.index,
            encoder_parameters: self.encoder_parameters.clone(),
        })
    }
}

#[instrument(skip(segments, encoder_params))]
pub fn convert_files_to_chunks(
    segments: Vec<PathBuf>,
    encoder_params: Vec<String>,
) -> Result<Vec<Chunk>, VideoEncodeError> {
    debug!("Converting {} files to chunks", segments.len());

    let chunks: Vec<Chunk> = segments
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            // It's good to check path.exists() here too, or ensure `new` returns Result
            if !path.exists() {
                error!(
                    "Segment file does not exist during chunk conversion: {:?}",
                    path
                );
                // This should ideally propagate as an error
                panic!(
                    "Segment file does not exist for convert_files_to_chunks: {:?}",
                    path
                );
            }
            Chunk::new(path, index, encoder_params.clone())
        })
        .collect();

    info!("Converted {} files to chunks", chunks.len());
    Ok(chunks)
}
// `split_video` function removed (moved to orchestration)
// `verify_ffmpeg` function removed (moved to ffmpeg::utils)

pub mod concatenator;
pub mod encoder;
pub mod segmenter;
pub mod utils;

pub use concatenator::*;
pub use encoder::*;
pub use segmenter::*;
pub use utils::*;

use std::path::PathBuf;
use ferris_swarm_core::{Chunk, VideoEncodeError};

/// Extension trait for Chunk to add encoding functionality
pub trait ChunkEncoder {
    fn encode(&self, output_path: PathBuf) -> Result<Chunk, VideoEncodeError>;
}

impl ChunkEncoder for Chunk {
    fn encode(&self, output_path: PathBuf) -> Result<Chunk, VideoEncodeError> {
        encode_with_ffmpeg(
            &self.source_path,
            &output_path,
            &self.encoder_parameters,
        )?;

        Ok(self.with_encoded_path(output_path))
    }
}
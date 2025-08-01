use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::VideoEncodeError;

/// Represents a video chunk for processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub source_path:        PathBuf,
    pub encoded_path:       Option<PathBuf>,
    pub index:              usize,
    pub encoder_parameters: Vec<String>,
}

impl Chunk {
    pub fn new(source_path: PathBuf, index: usize, encoder_parameters: Vec<String>) -> Result<Self, VideoEncodeError> {
        if !source_path.exists() {
            return Err(VideoEncodeError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source path does not exist: {:?}", source_path)
            )));
        }

        Ok(Chunk {
            source_path,
            encoded_path: None,
            index,
            encoder_parameters,
        })
    }

    /// Sets the encoded path for this chunk
    pub fn set_encoded_path(&mut self, encoded_path: PathBuf) {
        self.encoded_path = Some(encoded_path);
    }

    /// Creates a new chunk with the encoded path set
    pub fn with_encoded_path(&self, encoded_path: PathBuf) -> Self {
        Chunk {
            source_path: self.source_path.clone(),
            encoded_path: Some(encoded_path),
            index: self.index,
            encoder_parameters: self.encoder_parameters.clone(),
        }
    }
}

pub fn convert_files_to_chunks(
    segments: Vec<PathBuf>,
    encoder_params: Vec<String>,
) -> Result<Vec<Chunk>, VideoEncodeError> {
    let mut chunks = Vec::new();
    
    for (index, path) in segments.into_iter().enumerate() {
        let chunk = Chunk::new(path, index, encoder_params.clone())?;
        chunks.push(chunk);
    }

    Ok(chunks)
}

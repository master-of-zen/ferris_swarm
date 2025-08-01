use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoEncodeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("FFmpeg not found")]
    FfmpegNotFound,

    #[error("mkvmerge not found")]
    MkvmergeNotFound,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Concatenation error: {0}")]
    Concatenation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network transport error: {0}")]
    Transport(String),

    #[error("Node connection error: {0}")]
    NodeConnection(String),

    #[error("Chunk processing error: {0}")]
    ChunkProcessing(String),
}

pub type VideoEncodeResult<T> = Result<T, VideoEncodeError>;

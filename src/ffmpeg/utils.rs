use tracing::{debug, error, info, instrument};

use crate::error::VideoEncodeError;

#[instrument]
pub fn verify_ffmpeg() -> Result<(), VideoEncodeError> {
    debug!("Verifying FFmpeg installation");
    match which::which("ffmpeg") {
        Ok(path) => {
            info!("FFmpeg found at: {:?}", path);
            Ok(())
        },
        Err(e) => {
            error!("FFmpeg not found: {}", e);
            Err(VideoEncodeError::FfmpegNotFound)
        },
    }
}

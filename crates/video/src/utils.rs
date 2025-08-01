use tracing::{debug, error, info, instrument};

use ferris_swarm_core::error::VideoEncodeError;

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

#[instrument]
pub fn verify_mkvmerge() -> Result<(), VideoEncodeError> {
    debug!("Verifying mkvmerge installation");
    match which::which("mkvmerge") {
        Ok(path) => {
            info!("mkvmerge found at: {:?}", path);
            Ok(())
        },
        Err(e) => {
            error!("mkvmerge not found: {}", e);
            Err(VideoEncodeError::MkvmergeNotFound)
        },
    }
}

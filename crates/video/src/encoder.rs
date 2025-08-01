use std::{path::Path, process::Command};

use ferris_swarm_core::error::VideoEncodeError;
use tracing::{debug, error, instrument};

#[instrument(skip(encoder_parameters))]
pub fn encode_with_ffmpeg(
    input_path: &Path,
    output_path: &Path,
    encoder_parameters: &[String],
) -> Result<(), VideoEncodeError> {
    debug!(
        "Encoding with ffmpeg: input={:?}, output={:?}, params={:?}",
        input_path, output_path, encoder_parameters
    );

    let command_output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-i")
        .arg(input_path)
        .args(encoder_parameters) // These should already include -y if needed
        .arg(output_path)
        .output()?;

    if !command_output.status.success() {
        let error_msg = format!(
            "Failed to encode with ffmpeg. Stderr: {}",
            String::from_utf8_lossy(&command_output.stderr)
        );
        error!("{}", error_msg);
        return Err(VideoEncodeError::Encoding(error_msg));
    }

    debug!("Successfully encoded {:?} to {:?}", input_path, output_path);
    Ok(())
}

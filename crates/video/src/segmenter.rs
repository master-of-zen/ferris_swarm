/// This module is responsible for segmenting input file
/// into multiple independent files which are ready for processing
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use ferris_swarm_core::error::VideoEncodeError;
use tracing::{debug, error, info, instrument};

use crate::utils::verify_ffmpeg;

/// Due to the nature of method -segment_time
/// Getting expected number of segments is not
/// guaranteed as splitting can only be done at keyframes
/// Which means that split time will be inconsistent
/// and exact on file keyframe structure.
#[instrument]
pub fn segment_video(
    input_path: &Path,
    segment_duration: f64,
    segment_dir: &Path,
) -> Result<Vec<PathBuf>, VideoEncodeError> {
    debug!(
        "Starting video segmentation: input={:?}, duration={}, segment_dir={:?}",
        input_path, segment_duration, segment_dir
    );

    verify_ffmpeg()?; // Call the moved function
    debug!("FFmpeg verification successful");

    std::fs::create_dir_all(segment_dir)?;
    debug!("Created segment directory: {:?}", segment_dir);

    let output_pattern = segment_dir.join("chunk_%04d.mp4");
    debug!("Output pattern: {:?}", output_pattern);

    let binding = segment_duration.to_string();
    let inp = input_path.to_string_lossy();
    let output_pattern_str = output_pattern.to_string_lossy(); // Renamed for clarity

    let ffmpeg_args = vec![
        "-hide_banner",
        "-i",
        &inp,
        "-y",
        "-an", // don't copy audio
        "-sn", // don't copy subtitles
        "-dn", // don't copy other data
        "-c",
        "copy",
        "-map",
        "0",
        "-segment_time",
        &binding,
        "-f",
        "segment",
        "-reset_timestamps",
        "1",
        &output_pattern_str,
    ];

    debug!("FFmpeg command: ffmpeg {:?}", ffmpeg_args);

    let status = Command::new("ffmpeg").args(&ffmpeg_args).status()?; // Removed -hide_banner from here as it's in args

    if !status.success() {
        error!("Failed to split video. FFmpeg exit status: {}", status);
        return Err(VideoEncodeError::Encoding(
            "Failed to split video".to_string(),
        ));
    }

    debug!("Video segmentation completed successfully");

    let segmented_files: Vec<PathBuf> = std::fs::read_dir(segment_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("mp4"))
        .map(|entry| entry.path())
        .collect();

    debug!(
        "Segmented files: count={}, files={:?}",
        segmented_files.len(),
        segmented_files
    );

    info!("Video split into {} segments", segmented_files.len());

    Ok(segmented_files)
}

/// Extracts audio and other non-video streams from the input file.
/// Returns paths to the extracted files.
#[instrument]
pub fn extract_non_video_streams(
    input_path: &Path,
    temp_dir: &Path,
) -> Result<PathBuf, VideoEncodeError> {
    debug!("Extracting non-video streams from: {:?}", input_path);

    std::fs::create_dir_all(temp_dir)?;

    let streams_path = temp_dir.join("non_video_streams.mkv"); // Changed extension for clarity
    let status = Command::new("ffmpeg")
        .arg("-hide_banner")
        .args(&[
            "-i",
            input_path.to_str().unwrap(),
            "-y",
            "-map",
            "0",
            "-vn", // no video
            "-c",  // copy all other streams
            "copy",
            streams_path.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        error!("Failed to extract non-video streams");
        return Err(VideoEncodeError::Encoding(
            "Failed to extract non-video streams".to_string(),
        ));
    }
    info!("Extracted non-video streams to {:?}", streams_path);
    Ok(streams_path)
}

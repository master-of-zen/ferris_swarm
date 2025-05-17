use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tracing::{debug, error, info, instrument};

use crate::error::VideoEncodeError;

/// Concatenates video segments and adds back non-video streams.
#[instrument(skip(segment_paths))]
pub fn concatenate_videos_and_copy_streams(
    segment_paths: Vec<PathBuf>,
    original_input: &Path,
    output_file: &Path,
    temp_dir: &PathBuf,
    expected_segments: usize,
) -> Result<(), VideoEncodeError> {
    // Verify that all segments exist and match the expected count
    if segment_paths.len() != expected_segments {
        return Err(VideoEncodeError::Concatenation(format!(
            "Mismatch in segment count. Expected: {}, Found: {}",
            expected_segments,
            segment_paths.len()
        )));
    }

    for path in segment_paths.iter() {
        if !path.exists() {
            return Err(VideoEncodeError::Concatenation(format!(
                "Segment file not found: {:?}",
                path
            )));
        }
    }

    // Create a temporary file list for FFmpeg
    // Unfortunately due to current implementation path of the files inside
    // is relative to the file
    let temp_file_list = PathBuf::from("file_list.txt");
    let file_list_content: String = segment_paths
        .iter()
        .map(|path| format!("file '{}'\n", path.to_str().unwrap()))
        .collect();
    fs::write(&temp_file_list, file_list_content)?;

    let temp_st = temp_file_list.to_string_lossy();
    let original_input = original_input.to_string_lossy();
    let output_file = output_file.to_string_lossy();

    // Prepare FFmpeg command
    let ffmpeg_args = vec![
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        &temp_st,
        "-i",
        &original_input,
        "-map",
        "0:v", // map video from concatenated segments
        "-map",
        "1", // map all streams from original input
        "-c",
        "copy",
        &output_file,
    ];

    debug!("FFmpeg command: ffmpeg {:?}", ffmpeg_args);

    // Execute FFmpeg command
    let status = Command::new("ffmpeg").arg("-hide_banner").args(&ffmpeg_args).status()?;

    if !status.success() {
        error!("Failed to concatenate videos and copy streams");
        return Err(VideoEncodeError::Concatenation(
            "Failed to concatenate videos and copy streams".to_string(),
        ));
    }

    info!(
        "Successfully concatenated {} video segments and copied all streams to the final video",
        segment_paths.len(),
    );

    // Clean up temporary file
    fs::remove_file(temp_file_list)?;

    Ok(())
}

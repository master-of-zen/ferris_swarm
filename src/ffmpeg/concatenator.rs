use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tracing::{debug, error, info, instrument};

use crate::error::VideoEncodeError;

/// Concatenates video segments and adds back non-video streams from an original
/// source.
#[instrument(skip(segment_paths, non_video_stream_file))]
pub fn concatenate_videos_and_copy_streams(
    segment_paths: Vec<PathBuf>,
    non_video_stream_file: &Path, // Changed from original_input to be more specific
    output_file: &Path,
    _temp_dir: &PathBuf, // temp_dir seems unused here now, file_list.txt is created in CWD
    expected_segments: usize,
) -> Result<(), VideoEncodeError> {
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
    if !non_video_stream_file.exists() {
        return Err(VideoEncodeError::Concatenation(format!(
            "Non-video stream file not found: {:?}",
            non_video_stream_file
        )));
    }

    // Create a temporary file list for FFmpeg relative to current working directory
    let temp_file_list_path = PathBuf::from("file_list.txt");
    let file_list_content: String = segment_paths
        .iter()
        .map(|path| {
            // Ensure paths in file_list.txt are absolute or resolvable by ffmpeg
            // For simplicity, let's assume ffmpeg is run where these paths are relative,
            // or make them absolute. Using absolute paths is safer.
            let abs_path = path.canonicalize().map_err(|e| VideoEncodeError::Io(e))?;
            Ok(format!("file '{}'\n", abs_path.to_string_lossy()))
        })
        .collect::<Result<String, VideoEncodeError>>()?; // Handle potential errors

    fs::write(&temp_file_list_path, file_list_content)?;

    let temp_file_list_str = temp_file_list_path.to_string_lossy();
    let non_video_stream_file_str = non_video_stream_file.to_string_lossy();
    let output_file_str = output_file.to_string_lossy();

    // FFmpeg command to concatenate videos and map streams
    // -map 0:v maps video from the first input (concatenated segments)
    // -map 1 maps all streams from the second input (original non-video streams)
    // -c copy copies streams without re-encoding
    // -shortest might be useful if audio/video durations mismatch slightly in
    // original
    let ffmpeg_args = vec![
        "-f",
        "concat",
        "-safe",
        "0", // Allow unsafe file paths if needed, but absolute paths are better
        "-i",
        &temp_file_list_str,
        "-i",
        &non_video_stream_file_str,
        "-map",
        "0:v?", // Map video from first input, if present
        "-map",
        "1?", // Map all streams from second input, if present
        "-c",
        "copy",
        "-y", // Overwrite output file if it exists
        &output_file_str,
    ];

    debug!("FFmpeg command: ffmpeg {:?}", ffmpeg_args);

    let output = Command::new("ffmpeg").arg("-hide_banner").args(&ffmpeg_args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(
            "Failed to concatenate videos and copy streams. Stderr: {}",
            stderr
        );
        fs::remove_file(&temp_file_list_path)?; // Clean up even on failure
        return Err(VideoEncodeError::Concatenation(format!(
            "Failed to concatenate. FFmpeg stderr: {}",
            stderr
        )));
    }

    info!(
        "Successfully concatenated {} video segments and copied streams to {}",
        segment_paths.len(),
        output_file_str
    );

    fs::remove_file(temp_file_list_path)?;

    Ok(())
}

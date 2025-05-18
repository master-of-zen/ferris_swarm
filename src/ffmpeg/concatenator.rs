use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tracing::{debug, error, info, instrument};

use crate::error::VideoEncodeError;

/// Concatenates video segments using FFmpeg and adds back non-video streams.
#[instrument(skip(segment_paths, non_video_stream_file))]
pub fn concatenate_videos_ffmpeg(
    segment_paths: Vec<PathBuf>,
    non_video_stream_file: &Path,
    output_file: &Path,
    temp_dir: &PathBuf, // Used for file_list.txt
    expected_segments: usize,
) -> Result<(), VideoEncodeError> {
    if segment_paths.is_empty() {
        return Err(VideoEncodeError::Concatenation(
            "No video segments provided for FFmpeg concatenation.".to_string(),
        ));
    }
    if segment_paths.len() != expected_segments {
        return Err(VideoEncodeError::Concatenation(format!(
            "FFmpeg: Mismatch in segment count. Expected: {}, Found: {}",
            expected_segments,
            segment_paths.len()
        )));
    }

    for path in segment_paths.iter() {
        if !path.exists() {
            return Err(VideoEncodeError::Concatenation(format!(
                "FFmpeg: Segment file not found: {:?}",
                path
            )));
        }
    }
    if !non_video_stream_file.exists() {
        return Err(VideoEncodeError::Concatenation(format!(
            "FFmpeg: Non-video stream file not found: {:?}",
            non_video_stream_file
        )));
    }

    fs::create_dir_all(temp_dir).map_err(VideoEncodeError::Io)?;
    let temp_file_list_path = temp_dir.join("ffmpeg_concat_list.txt");

    let file_list_content: String = segment_paths
        .iter()
        .map(|path| {
            let abs_path = path.canonicalize().map_err(|e| VideoEncodeError::Io(e))?;
            Ok(format!("file '{}'\n", abs_path.to_string_lossy()))
        })
        .collect::<Result<String, VideoEncodeError>>()?;

    fs::write(&temp_file_list_path, file_list_content).map_err(VideoEncodeError::Io)?;

    let temp_file_list_str = temp_file_list_path.to_string_lossy();
    let non_video_stream_file_str = non_video_stream_file.to_string_lossy();
    let output_file_str = output_file.to_string_lossy();

    let ffmpeg_args = vec![
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        &temp_file_list_str,
        "-i",
        &non_video_stream_file_str,
        "-map",
        "0:v?", // Map video from first input (concatenated segments), if present
        "-map",
        "1?", // Map all streams from second input (non-video streams), if present
        "-c",
        "copy",
        "-y", // Overwrite output file if it exists
        &output_file_str,
    ];

    debug!("FFmpeg command: ffmpeg {:?}", ffmpeg_args);
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .args(&ffmpeg_args)
        .output()
        .map_err(VideoEncodeError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Failed to concatenate with FFmpeg. Stderr: {}", stderr);
        // Attempt to clean up temp file even on failure
        let _ = fs::remove_file(&temp_file_list_path);
        return Err(VideoEncodeError::Concatenation(format!(
            "FFmpeg concatenation failed. FFmpeg stderr: {}",
            stderr
        )));
    }

    info!(
        "FFmpeg: Successfully concatenated {} video segments and copied streams to {}",
        segment_paths.len(),
        output_file_str
    );
    fs::remove_file(temp_file_list_path).map_err(VideoEncodeError::Io)?;
    Ok(())
}

/// Concatenates video segments using mkvmerge and adds back non-video streams.
#[instrument(skip(segment_paths, non_video_stream_file))]
pub fn concatenate_videos_mkvmerge(
    segment_paths: Vec<PathBuf>,
    non_video_stream_file: &Path,
    output_file: &Path,
    _temp_dir: &PathBuf, // Not used by mkvmerge for this concatenation strategy
    expected_segments: usize,
) -> Result<(), VideoEncodeError> {
    if segment_paths.is_empty() {
        return Err(VideoEncodeError::Concatenation(
            "No video segments provided for mkvmerge concatenation.".to_string(),
        ));
    }
    if segment_paths.len() != expected_segments {
        return Err(VideoEncodeError::Concatenation(format!(
            "Mkvmerge: Mismatch in segment count. Expected: {}, Found: {}",
            expected_segments,
            segment_paths.len()
        )));
    }

    for path in segment_paths.iter() {
        if !path.exists() {
            return Err(VideoEncodeError::Concatenation(format!(
                "Mkvmerge: Segment file not found: {:?}",
                path
            )));
        }
    }
    if !non_video_stream_file.exists() {
        return Err(VideoEncodeError::Concatenation(format!(
            "Mkvmerge: Non-video stream file not found: {:?}",
            non_video_stream_file
        )));
    }

    let mut mkvmerge_args: Vec<String> = Vec::new();
    mkvmerge_args.push("-o".to_string());
    mkvmerge_args.push(output_file.to_string_lossy().into_owned()); // Output path as is

    // Add segments for concatenation
    // mkvmerge uses `+` to append subsequent files to the first one listed.
    for (i, seg_path) in segment_paths.iter().enumerate() {
        let canonical_seg_path = seg_path.canonicalize().map_err(VideoEncodeError::Io)?;
        if i > 0 {
            mkvmerge_args.push("+".to_string());
        }
        mkvmerge_args.push(canonical_seg_path.to_string_lossy().into_owned());
    }

    // Add the non-video streams file as a separate input for muxing.
    // If `extract_non_video_streams` produced it with `ffmpeg -vn`, it won't have
    // video. mkvmerge will then mux its audio/subtitle streams with the video
    // from segments.
    let canonical_non_video_path =
        non_video_stream_file.canonicalize().map_err(VideoEncodeError::Io)?;
    mkvmerge_args.push(canonical_non_video_path.to_string_lossy().into_owned());

    debug!("mkvmerge command: mkvmerge {:?}", mkvmerge_args);
    let output = Command::new("mkvmerge")
        .args(&mkvmerge_args)
        .output()
        .map_err(VideoEncodeError::Io)?;

    if !output.status.success() {
        // mkvmerge often prints progress/errors to stdout, and critical errors to
        // stderr
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        error!(
            "Failed to concatenate with mkvmerge. Stdout: '{}', Stderr: '{}'",
            stdout_str, stderr_str
        );
        return Err(VideoEncodeError::Concatenation(format!(
            "mkvmerge concatenation failed. Stdout: '{}', Stderr: '{}'",
            stdout_str, stderr_str
        )));
    }

    info!(
        "mkvmerge: Successfully concatenated {} video segments and muxed streams to {:?}",
        segment_paths.len(),
        output_file
    );
    Ok(())
}

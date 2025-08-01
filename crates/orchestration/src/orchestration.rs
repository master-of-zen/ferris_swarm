use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument};

use ferris_swarm_core::error::VideoEncodeError;
use ferris_swarm_video as ffmpeg;

#[instrument]
pub fn split_video_into_segments(
    input_path: &Path,
    segment_duration: f64,
    segment_dir: &Path, // This is a subdirectory within the JobTempConfig
) -> Result<Vec<PathBuf>, VideoEncodeError> {
    debug!(
        "Orchestrating video split: input={:?}, duration={}, segment_output_dir={:?}",
        input_path, segment_duration, segment_dir
    );

    // The segment_dir path is already prepared by JobTempConfig
    let segmented_files =
        ffmpeg::segmenter::segment_video(input_path, segment_duration, segment_dir)?;

    info!(
        "Video segmentation complete: {} segments created in {:?}",
        segmented_files.len(),
        segment_dir
    );

    Ok(segmented_files)
}

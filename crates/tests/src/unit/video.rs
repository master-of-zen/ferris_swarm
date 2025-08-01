// Video processing unit tests
use ferris_swarm_video::{verify_ffmpeg, verify_mkvmerge};
use ferris_swarm_core::VideoEncodeError;
use crate::common::init_test_logging;

#[test]
fn test_ffmpeg_verification() {
    init_test_logging();
    
    let result = verify_ffmpeg();
    match result {
        Ok(_) => {
            println!("FFmpeg is available");
        }
        Err(VideoEncodeError::FfmpegNotFound) => {
            println!("FFmpeg not found (expected in some test environments)");
        }
        Err(e) => {
            panic!("Unexpected error checking FFmpeg: {}", e);
        }
    }
}

#[test]
fn test_mkvmerge_verification() {
    init_test_logging();
    
    let result = verify_mkvmerge();
    match result {
        Ok(_) => {
            println!("mkvmerge is available");
        }
        Err(VideoEncodeError::MkvmergeNotFound) => {
            println!("mkvmerge not found (expected in some test environments)");
        }
        Err(e) => {
            panic!("Unexpected error checking mkvmerge: {}", e);
        }
    }
}
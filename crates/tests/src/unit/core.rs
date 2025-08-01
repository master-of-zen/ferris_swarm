// Core types unit tests
use ferris_swarm_core::{Chunk, VideoEncodeError};
use crate::common::{init_test_logging, mock_data};

#[test]
fn test_chunk_creation_with_existing_file() {
    init_test_logging();
    
    // Create a temporary file for testing
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path().to_path_buf();
    
    let chunk = Chunk::new(
        temp_path,
        0,
        vec!["test_param".to_string()],
    );
    
    assert!(chunk.is_ok());
    let chunk = chunk.unwrap();
    assert_eq!(chunk.index, 0);
    assert_eq!(chunk.encoder_parameters, vec!["test_param".to_string()]);
}

#[test]
fn test_chunk_creation_with_nonexistent_file() {
    init_test_logging();
    
    // Test with non-existent file
    let result = Chunk::new(
        std::path::PathBuf::from("/nonexistent/file.mp4"),
        0,
        vec!["test_param".to_string()],
    );
    
    assert!(result.is_err());
}

#[test]
fn test_chunk_mock_data() {
    init_test_logging();
    
    let chunk = mock_data::create_test_chunk();
    assert_eq!(chunk.index, 0);
    assert_eq!(chunk.encoder_parameters, vec!["test_param".to_string()]);
}

#[test]
fn test_video_encode_error_display() {
    let error = VideoEncodeError::Encoding("test error".to_string());
    assert!(error.to_string().contains("test error"));
}
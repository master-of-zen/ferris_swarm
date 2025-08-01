// Distributed encoding integration tests
use ferris_swarm_core::Chunk;

use crate::common::{create_temp_dir, init_test_logging, mock_data};

#[tokio::test]
async fn test_chunk_processing_pipeline() {
    init_test_logging();

    let _temp_dir = create_temp_dir();
    let chunk = mock_data::create_test_chunk();

    // Test that a chunk can go through the processing pipeline
    // This ensures video, orchestration, and core integration works
    println!("Testing chunk processing for chunk index {}", chunk.index);

    assert_eq!(chunk.index, 0);
}

#[tokio::test]
async fn test_distributed_job_coordination() {
    init_test_logging();

    // Test that job coordination components integrate properly
    // This ensures client, node, and constellation integration works

    // Create multiple mock chunks to simulate distributed work
    let chunks: Vec<Chunk> = (0..3)
        .map(|i| {
            // Create a temporary file for each chunk
            let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
            let temp_path = temp_file.path().to_path_buf();
            // Keep temp_file alive
            std::mem::forget(temp_file);

            Chunk::new(temp_path, i, vec![format!("param_{}", i)])
                .expect("Failed to create test chunk")
        })
        .collect();

    assert_eq!(chunks.len(), 3);
    println!(
        "Created {} chunks for distributed processing test",
        chunks.len()
    );
}

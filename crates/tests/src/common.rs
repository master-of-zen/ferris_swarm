// Common test utilities and setup functions
use ferris_swarm_logging::init_logging;

/// Initialize test logging for all tests
pub fn init_test_logging() {
    let _ = std::panic::catch_unwind(|| {
        init_logging();
    });
}

/// Create a temporary directory for tests
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Mock data for testing
pub mod mock_data {
    use ferris_swarm_core::Chunk;
    
    pub fn create_test_chunk() -> Chunk {
        // Create a temporary file for testing
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path().to_path_buf();
        
        Chunk::new(
            temp_path,
            0,
            vec!["test_param".to_string()],
        ).expect("Failed to create test chunk")
    }
}

/// Network testing utilities
pub mod network {
    use std::net::TcpListener;
    
    /// Find an available port for testing
    pub fn find_available_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind to port")
            .local_addr()
            .expect("Failed to get local address")
            .port()
    }
}
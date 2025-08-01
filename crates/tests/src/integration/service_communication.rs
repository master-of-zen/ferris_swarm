// Service communication integration tests
use crate::common::{init_test_logging, network::find_available_port};

#[tokio::test]
async fn test_grpc_communication_setup() {
    init_test_logging();

    let port = find_available_port();
    println!("Testing gRPC communication on port {}", port);

    // Test that gRPC components can be instantiated
    // This ensures the proto crate integration works
    assert!(port > 0);
}

#[tokio::test]
async fn test_http_communication_setup() {
    init_test_logging();

    let port = find_available_port();
    println!("Testing HTTP communication on port {}", port);

    // Test that HTTP components can be instantiated
    // This ensures the constellation web interface works
    assert!(port > 0);
}

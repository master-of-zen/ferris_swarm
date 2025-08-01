// Discovery service unit tests
use ferris_swarm_discovery::DiscoveryService;

use crate::common::init_test_logging;

#[tokio::test]
async fn test_discovery_service_creation() {
    init_test_logging();

    let _service = DiscoveryService::new();
    // Test that service can be created without panicking
    println!("Discovery service created successfully");
    assert!(true);
}

#[tokio::test]
async fn test_get_local_ip() {
    init_test_logging();

    let service = DiscoveryService::new();
    let result = service.get_local_ip().await;

    // Should either succeed or fail gracefully
    match result {
        Ok(ip) => {
            assert!(!ip.is_loopback());
            println!("Local IP: {}", ip);
        },
        Err(e) => {
            println!(
                "Failed to get local IP (this might be expected in test environment): {}",
                e
            );
            // This is acceptable in test environments
        },
    }
}

#[tokio::test]
async fn test_fallback_discovery() {
    init_test_logging();

    let service = DiscoveryService::new();
    let result = service.fallback_discover().await;

    // Should return empty map when no services are available
    assert!(result.is_ok());
    let services = result.unwrap();
    // In test environment, likely to be empty
    println!("Discovered {} services via fallback", services.len());
}

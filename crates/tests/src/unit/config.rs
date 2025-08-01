// Configuration unit tests
use ferris_swarm_config::{Settings, TempConfig};
use crate::common::{init_test_logging, create_temp_dir};

#[test]
fn test_settings_default() {
    init_test_logging();
    
    let settings = Settings::default();
    // Test that default settings are reasonable
    assert!(settings.client.encoder_params.len() > 0);
    assert!(settings.node.address.len() > 0);
}

#[test]
fn test_temp_config_creation() {
    init_test_logging();
    
    let temp_dir = create_temp_dir();
    let input_file = temp_dir.path().join("input.mp4");
    let output_file = "output.mkv";
    
    let config = TempConfig::new(
        Some(temp_dir.path().to_path_buf()), 
        &input_file, 
        output_file
    );
    
    // TempConfig::new returns TempConfig directly, not Result
    assert!(config.temp_dir.exists());
}
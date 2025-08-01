use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use ferris_swarm_logging::init_logging;
use ferris_swarm_node::{
    auto_register::{detect_node_capabilities, get_local_ip, NodeAutoRegister},
    cli::Cli,
    config::load_settings_with_cli_overrides,
    service::NodeEncodingService,
};
use ferris_swarm_proto::protos::video_encoding::video_encoding_service_server::VideoEncodingServiceServer;
use ferris_swarm_video::utils::verify_ffmpeg;
use tonic::transport::Server;
use tracing::{debug, error, info, instrument, warn};

const MAX_MESSAGE_SIZE_BYTES: usize = 1 * 1024 * 1024 * 1024; // 1 GB

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    init_logging();
    info!("Ferris Swarm Node: Initializing...");

    let cli_args = Cli::parse();
    debug!("Parsed CLI arguments: {:?}", cli_args);

    let settings = load_settings_with_cli_overrides(&cli_args)?;
    debug!("Effective settings for node: {:?}", settings);

    verify_ffmpeg()?;

    // Handle auto-registration (enabled by default)
    let mut auto_register_handle = None;
    if cli_args.should_auto_register() {
        info!("Auto-registration enabled (default behavior)");

        let capabilities = detect_node_capabilities(
            cli_args.cpu_cores,
            cli_args.memory_gb,
            cli_args.max_chunks,
            cli_args.encoders.clone(),
        )?;

        // Use discovery with fallback to manual URL
        let auto_register_config = NodeAutoRegister::create_with_discovery_fallback(
            cli_args.constellation_url.clone(),
            cli_args.node_name.clone(),
            capabilities,
            Duration::from_secs(cli_args.heartbeat_interval),
        )
        .await?;

        // Determine node address for registration
        let node_address = if let Some(addr_str) = cli_args.address.as_ref() {
            addr_str.parse()?
        } else {
            let local_ip = get_local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
            let port = settings
                .node
                .address
                .split(':')
                .nth(1)
                .unwrap_or("8080")
                .parse::<u16>()
                .unwrap_or(8080);
            format!("{}:{}", local_ip, port).parse()?
        };

        let mut auto_register = NodeAutoRegister::new(auto_register_config.clone(), node_address);

        // Perform initial registration
        match auto_register.register().await {
            Ok(node_id) => {
                info!(
                    "Successfully registered as node: {} ({})",
                    node_id, auto_register_config.node_name
                );

                // Start heartbeat service (enabled by default)
                if cli_args.should_enable_heartbeat() {
                    info!("Starting heartbeat service (default behavior)...");
                    let heartbeat_service = auto_register.start_heartbeat_service();
                    auto_register_handle = Some(tokio::spawn(async move {
                        if let Err(e) = heartbeat_service.await {
                            error!("Heartbeat service failed: {}", e);
                        }
                    }));
                } else {
                    info!("Heartbeat service disabled via --no-heartbeat");
                }
            },
            Err(e) => {
                warn!(
                    "Auto-registration failed: {}. Continuing without registration.",
                    e
                );
            },
        }
    } else {
        info!("Auto-registration disabled via --no-auto-register");
    }
    // The NodeEncodingService now takes the temp_dir path directly.
    // This temp_dir comes from the node's specific configuration.
    let node_service = NodeEncodingService::new(settings.node.temp_dir.clone());

    let grpc_service = VideoEncodingServiceServer::new(node_service)
        .max_encoding_message_size(MAX_MESSAGE_SIZE_BYTES)
        .max_decoding_message_size(MAX_MESSAGE_SIZE_BYTES);

    let listen_address = settings.node.address.parse()?;
    info!(
        "Node server configured. Starting to listen on {}",
        listen_address
    );

    // Start the gRPC server
    let server_result = Server::builder().add_service(grpc_service).serve(listen_address);

    // Run server and heartbeat service concurrently
    if let Some(mut heartbeat_handle) = auto_register_handle {
        tokio::select! {
            result = server_result => {
                heartbeat_handle.abort();
                result?;
            },
            _ = &mut heartbeat_handle => {
                warn!("Heartbeat service ended unexpectedly");
            }
        }
    } else {
        server_result.await?;
    }

    info!("Ferris Swarm Node: Shutting down.");
    Ok(())
}

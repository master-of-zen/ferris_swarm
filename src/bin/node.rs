use anyhow::Result;
use clap::Parser;
use ferris_swarm::{
    ffmpeg::utils::verify_ffmpeg,
    logging::init_logging,
    node::{cli::Cli, config::load_settings_with_cli_overrides, service::NodeEncodingService},
    protos::video_encoding::video_encoding_service_server::VideoEncodingServiceServer,
    settings::Settings, // Only used for type hint
};
use tonic::transport::Server;
use tracing::{debug, info, instrument};

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

    Server::builder().add_service(grpc_service).serve(listen_address).await?;

    info!("Ferris Swarm Node: Shutting down.");
    Ok(())
}

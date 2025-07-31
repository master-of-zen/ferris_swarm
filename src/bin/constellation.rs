use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use ferris_swarm::constellation::{
    config::ConstellationConfig, routes::create_router, state::ConstellationState,
};
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "ferris_swarm_constellation")]
#[command(about = "Ferris Swarm Constellation - Central Management Service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start(StartArgs),
    Config(ConfigArgs),
}

#[derive(Args)]
struct StartArgs {
    #[arg(short, long, help = "Configuration file path")]
    config: Option<PathBuf>,
    
    #[arg(short, long, help = "Bind address (e.g., 0.0.0.0:3030)")]
    bind: Option<String>,
    
    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,
}

#[derive(Args)]
struct ConfigArgs {
    #[arg(short, long, help = "Generate default configuration file")]
    generate: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start(args) => start_constellation(args).await,
        Commands::Config(args) => handle_config(args).await,
    }
}

async fn start_constellation(args: StartArgs) -> anyhow::Result<()> {
    ferris_swarm::logging::init_logging(args.verbose)?;

    let config = load_config(args.config).await?;
    
    let bind_address = args.bind
        .map(|addr| addr.parse())
        .transpose()?
        .unwrap_or(config.server.bind_address);

    info!(
        "Starting Ferris Swarm Constellation on {}",
        bind_address
    );

    let state = ConstellationState::new(config);
    let cleanup_handle = state.clone().start_cleanup_task();

    let app = create_router(state);
    let listener = TcpListener::bind(bind_address).await?;
    
    info!("Constellation service is running on {}", bind_address);
    info!("Dashboard available at: http://{}", bind_address);
    info!("WebSocket endpoint: ws://{}/ws", bind_address);
    info!("API endpoint: http://{}/api", bind_address);

    let server_result = axum::serve(listener, app).await;
    
    cleanup_handle.abort();
    
    if let Err(e) = server_result {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}

async fn handle_config(args: ConfigArgs) -> anyhow::Result<()> {
    if let Some(path) = args.generate {
        let config = ConstellationConfig::default();
        let toml_content = toml::to_string_pretty(&config)?;
        
        tokio::fs::write(&path, toml_content).await?;
        println!("Generated default configuration at: {}", path.display());
    } else {
        println!("Use --generate <path> to create a default configuration file");
    }

    Ok(())
}

async fn load_config(config_path: Option<PathBuf>) -> anyhow::Result<ConstellationConfig> {
    if let Some(path) = config_path {
        info!("Loading configuration from: {}", path.display());
        let content = tokio::fs::read_to_string(&path).await?;
        let config: ConstellationConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        info!("Using default configuration");
        Ok(ConstellationConfig::default())
    }
}
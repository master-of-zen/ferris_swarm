use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use ferris_swarm_constellation::{
    auto_register::AutoRegister,
    config::ConstellationConfig, 
    routes::create_router, 
    state::ConstellationState,
};
use ferris_swarm_discovery::DiscoveryService;
use ferris_swarm_logging::init_logging;
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
    Nodes(NodesArgs),
}

#[derive(Args)]
struct StartArgs {
    #[arg(short, long, help = "Configuration file path")]
    config: Option<PathBuf>,
    
    #[arg(short, long, help = "Bind address (e.g., 0.0.0.0:3030)")]
    bind: Option<String>,
    
    #[arg(short, long, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(long, help = "Nodes configuration file for auto-registration")]
    nodes_config: Option<PathBuf>,

    #[arg(long, help = "Enable auto-registration from nodes config")]
    auto_register: bool,

    #[arg(long, help = "Disable mDNS service advertisement")]
    no_mdns: bool,
}

#[derive(Args)]
struct ConfigArgs {
    #[arg(short, long, help = "Generate default configuration file")]
    generate: Option<PathBuf>,
}

#[derive(Args)]
struct NodesArgs {
    #[arg(short, long, help = "Generate sample nodes configuration file")]
    generate: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start(args) => start_constellation(args).await,
        Commands::Config(args) => handle_config(args).await,
        Commands::Nodes(args) => handle_nodes_config(args).await,
    }
}

async fn start_constellation(args: StartArgs) -> anyhow::Result<()> {
    init_logging();

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

    // Start auto-registration service if enabled
    let auto_register_handle = if args.auto_register {
        if let Some(nodes_config_path) = args.nodes_config {
            info!("Starting auto-registration service with config: {:?}", nodes_config_path);
            let mut auto_register = AutoRegister::new(nodes_config_path, state.clone());
            
            Some(tokio::spawn(async move {
                if let Err(e) = auto_register.start_auto_registration().await {
                    error!("Auto-registration service failed: {}", e);
                }
            }))
        } else {
            info!("Auto-registration enabled but no nodes config provided. Use --nodes-config");
            None
        }
    } else {
        None
    };

    // Start mDNS service advertisement
    let _mdns_handle = if !args.no_mdns {
        let discovery_service = DiscoveryService::new();
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "ferris-constellation".to_string());
        let port = bind_address.port();

        info!("Starting mDNS service advertisement for {}:{}", hostname, port);
        
        match discovery_service.advertise_constellation(port, &hostname).await {
            Ok(_) => {
                info!("mDNS service advertisement started successfully");
                Some(())
            }
            Err(e) => {
                error!("Failed to start mDNS advertisement: {}", e);
                info!("Constellation will continue without network discovery");
                None
            }
        }
    } else {
        info!("mDNS service advertisement disabled");
        None
    };

    let app = create_router(state);
    let listener = TcpListener::bind(bind_address).await?;
    
    info!("Constellation service is running on {}", bind_address);
    info!("Dashboard available at: http://{}", bind_address);
    info!("WebSocket endpoint: ws://{}/ws", bind_address);
    info!("API endpoint: http://{}/api", bind_address);

    let server_result = axum::serve(listener, app).await;
    
    // Cleanup
    cleanup_handle.abort();
    if let Some(auto_handle) = auto_register_handle {
        auto_handle.abort();
    }
    
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

async fn handle_nodes_config(args: NodesArgs) -> anyhow::Result<()> {
    if let Some(path) = args.generate {
        AutoRegister::generate_sample_config(&path).await?;
        println!("Generated sample nodes configuration at: {}", path.display());
    } else {
        println!("Use --generate <path> to create a sample nodes configuration file");
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
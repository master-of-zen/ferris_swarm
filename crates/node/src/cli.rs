use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Ferris Swarm Node: Zero-config distributed video encoding node with automatic \
             constellation discovery.",
    long_about = "Ferris Swarm Node automatically discovers and registers with constellation \
                  services on the local network. Auto-registration and heartbeat are enabled by \
                  default for true plug-and-play operation."
)]
pub struct Cli {
    /// Path to the configuration file (e.g., config.toml)
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,

    /// Node's listening address (e.g., 0.0.0.0:50051).
    /// Overrides 'address' in [node] section of config file if provided.
    #[arg(long)] // Changed from -n to --address for clarity
    pub address: Option<String>,

    /// Temporary directory for this node's processing.
    /// Overrides 'temp_dir' in [node] section of config file if provided.
    #[arg(long)]
    pub temp_dir: Option<PathBuf>,

    /// Disable auto-registration with constellation
    #[arg(long, help = "Disable auto-registration with constellation")]
    pub no_auto_register: bool,

    /// Constellation URL for auto-registration
    #[arg(
        long,
        help = "Constellation URL (optional if using mDNS discovery)",
        env = "CONSTELLATION_URL"
    )]
    pub constellation_url: Option<String>,

    /// Node name for registration (defaults to hostname)
    #[arg(long, help = "Node name", env = "NODE_NAME")]
    pub node_name: Option<String>,

    /// Override detected CPU cores
    #[arg(long, help = "Override detected CPU cores", env = "NODE_CPU_CORES")]
    pub cpu_cores: Option<u32>,

    /// Override detected memory in GB
    #[arg(long, help = "Override detected memory GB", env = "NODE_MEMORY_GB")]
    pub memory_gb: Option<u32>,

    /// Maximum concurrent chunks this node can process
    #[arg(long, help = "Max concurrent chunks", env = "NODE_MAX_CHUNKS")]
    pub max_chunks: Option<u32>,

    /// Supported encoders (comma-separated)
    #[arg(long, help = "Supported encoders", env = "NODE_ENCODERS")]
    pub encoders: Option<String>,

    /// Disable heartbeat service to constellation
    #[arg(long, help = "Disable heartbeat to constellation")]
    pub no_heartbeat: bool,

    /// Heartbeat interval in seconds
    #[arg(long, help = "Heartbeat interval in seconds", default_value = "30")]
    pub heartbeat_interval: u64,
}

impl Cli {
    /// Check if auto-registration should be enabled (default: true, unless
    /// --no-auto-register)
    pub fn should_auto_register(&self) -> bool {
        !self.no_auto_register
    }

    /// Check if heartbeat should be enabled (default: true, unless
    /// --no-heartbeat)
    pub fn should_enable_heartbeat(&self) -> bool {
        !self.no_heartbeat
    }
}

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Ferris Swarm Node: Listens for and processes video encoding tasks.", long_about = None)]
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
}

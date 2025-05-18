use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Ferris Swarm Client: Distributes video encoding tasks.", long_about = None)]
pub struct Cli {
    /// Input video file path
    #[arg(short, long)]
    pub input_file: PathBuf,

    /// Output video file path
    #[arg(short, long)]
    pub output_file: String,

    /// Path to the configuration file (e.g., config.toml)
    #[arg(long)]
    pub config_file: Option<PathBuf>,

    /// List of node addresses (e.g., http://127.0.0.1:50051)
    /// Overrides node_addresses in config file if provided.
    #[arg(short, long, value_delimiter = ',')]
    pub nodes: Vec<String>,

    /// List of concurrent processing slots for each corresponding node.
    /// Must match the number of --nodes if provided. (e.g., 2,4,2)
    #[arg(long, value_delimiter = ',')]
    pub slots: Vec<usize>,

    /// Encoder parameters string (e.g., "-c:v libx264 -crf 23").
    /// Overrides encoder_params in config file if provided.
    /// Note: these are passed as a single string and split, or multiple args.
    /// For simplicity, current implementation expects them pre-split or one
    /// string. The original code splits a Vec<String> where each string
    /// might contain spaces.
    #[arg(long, num_args = 1..)]
    pub encoder_params: Option<Vec<String>>,

    /// Temporary directory for client-side processing for this job.
    /// Overrides temp_dir in [processing] section of config file if provided.
    #[arg(long)]
    pub temp_dir: Option<PathBuf>,

    /// Duration of each video segment in seconds.
    /// Overrides segment_duration in [processing] section of config file if
    /// provided.
    #[arg(long)]
    pub segment_duration: Option<f64>,
}

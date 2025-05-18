use anyhow::Result;
use tracing::{debug, instrument};

use super::cli::Cli;
use crate::settings::Settings; // Use the Cli from the same client module

#[instrument(skip(cli))]
pub fn load_settings_with_cli_overrides(cli: &Cli) -> Result<Settings> {
    debug!("Loading client settings with CLI overrides...");
    let mut settings = match &cli.config_file {
        Some(path) => {
            debug!("Loading settings from_file: {:?}", path);
            Settings::from_file(path)?
        },
        None => {
            debug!(
                "No config file specified, loading default settings (e.g., from 'config.toml' or \
                 defaults)."
            );
            Settings::new()? // Assumes Settings::new() tries to load
                             // "config.toml" or has defaults
        },
    };

    if !cli.nodes.is_empty() {
        debug!("Overriding node_addresses from CLI: {:?}", cli.nodes);
        settings.client.node_addresses = cli.nodes.clone();
    }

    if let Some(cli_encoder_params) = &cli.encoder_params {
        debug!(
            "Overriding encoder_params from CLI: {:?}",
            cli_encoder_params
        );
        // The original code had a specific way of splitting these params.
        // Assuming cli_encoder_params is already a Vec<String> as ffmpeg expects.
        let mut params: Vec<String> = Vec::new();
        cli_encoder_params.iter().for_each(|x| {
            params.extend(x.split_whitespace() // Split by whitespace
                 .map(str::to_string)
                 .collect::<Vec<String>>());
        });
        if !params.contains(&"-y".to_string()) {
            // Ensure -y for overwriting
            params.push("-y".to_string());
        }
        settings.client.encoder_params = params;
    }

    if let Some(temp_dir) = &cli.temp_dir {
        debug!("Overriding processing.temp_dir from CLI: {:?}", temp_dir);
        settings.processing.temp_dir = temp_dir.clone();
    }

    if let Some(segment_duration) = cli.segment_duration {
        debug!(
            "Overriding processing.segment_duration from CLI: {}",
            segment_duration
        );
        settings.processing.segment_duration = segment_duration;
    }

    // Validate that if CLI nodes are provided, CLI slots are also provided and
    // match length
    if !cli.nodes.is_empty() && cli.slots.is_empty() {
        return Err(anyhow::anyhow!(
            "If --nodes are provided via CLI, --slots must also be provided."
        ));
    }
    if !cli.nodes.is_empty() && !cli.slots.is_empty() && cli.nodes.len() != cli.slots.len() {
        return Err(anyhow::anyhow!(
            "Number of --nodes ({}) must match number of --slots ({}) via CLI.",
            cli.nodes.len(),
            cli.slots.len()
        ));
    }

    debug!("Final client settings: {:?}", settings);
    Ok(settings)
}

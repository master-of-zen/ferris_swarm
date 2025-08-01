use anyhow::Result;
use tracing::{debug, instrument, warn}; // Added warn

use super::cli::Cli;
use ferris_swarm_config::settings::{ConcatenatorChoice, Settings};

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
            Settings::new()?
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
        let mut params: Vec<String> = Vec::new();
        cli_encoder_params.iter().for_each(|x| {
            params.extend(x.split_whitespace().map(str::to_string).collect::<Vec<String>>());
        });
        if !params.contains(&"-y".to_string()) {
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

    if let Some(concat_choice_str) = &cli.concatenator {
        match concat_choice_str.as_str() {
            "ffmpeg" => {
                debug!("Overriding processing.concatenator from CLI: ffmpeg");
                settings.processing.concatenator = ConcatenatorChoice::Ffmpeg;
            },
            "mkvmerge" => {
                debug!("Overriding processing.concatenator from CLI: mkvmerge");
                settings.processing.concatenator = ConcatenatorChoice::Mkvmerge;
            },
            _ => {
                // This case should not be reached due to clap's value_parser
                warn!(
                    "Invalid concatenator choice from CLI: {}. Using default/config value.",
                    concat_choice_str
                );
            },
        }
    }

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

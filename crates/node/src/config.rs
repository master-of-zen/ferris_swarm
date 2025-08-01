use anyhow::Result;
use tracing::{debug, instrument};

use super::cli::Cli;
use ferris_swarm_config::settings::Settings;

#[instrument(skip(cli))]
pub fn load_settings_with_cli_overrides(cli: &Cli) -> Result<Settings> {
    debug!("Loading node settings with CLI overrides...");
    let mut settings = match &cli.config_file {
        Some(path) => {
            debug!("Loading settings from file: {:?}", path);
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

    if let Some(address) = &cli.address {
        debug!("Overriding node.address from CLI: {}", address);
        settings.node.address = address.clone();
    }
    if let Some(temp_dir) = &cli.temp_dir {
        debug!("Overriding node.temp_dir from CLI: {:?}", temp_dir);
        // Assuming settings.node has a temp_dir field or we use processing.temp_dir for
        // node's local work too. The original config.toml has [node] temp_dir,
        // so we should ensure Settings reflects this. For now, using
        // settings.node.temp_dir if it exists in Settings struct, or adapting
        // if it should be settings.processing.temp_dir for the node's own scratch
        // space. The provided Settings struct has node.address and
        // processing.temp_dir. Let's assume node.temp_dir is meant for the
        // settings. If `Settings::NodeSettings` needs a `temp_dir`, it should
        // be added there. Current `config.toml` has `[node] temp_dir =
        // "./server_50051"`. So, `NodeSettings` should have `pub temp_dir:
        // PathBuf`.
        //
        // Let's modify `src/settings.rs` for this:
        // In `NodeSettings`: add `pub temp_dir: PathBuf,`
        // Then here:
        settings.node.temp_dir = temp_dir.clone();
    }

    debug!("Final node settings: {:?}", settings);
    Ok(settings)
}

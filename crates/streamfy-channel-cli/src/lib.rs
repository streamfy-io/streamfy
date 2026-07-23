pub mod cli;

use tracing::debug;
use anyhow::{anyhow, Result};

use streamfy_channel::{StreamfyChannelConfig, StreamfyBinVersion};
use streamfy_cli_common::install::{
    install_println, fetch_latest_version, fetch_package_file, install_bin,
};
use streamfy_index::{PackageId, HttpAgent};

pub async fn install_channel_streamfy_bin(
    channel_name: String,
    channel_config: &StreamfyChannelConfig,
    version: StreamfyBinVersion,
) -> Result<()> {
    let agent = HttpAgent::default();
    let target = streamfy_index::package_target()?;
    let id: PackageId = "streamfy/streamfy".parse()?;
    debug!(%target, %id, "Streamfy CLI updating self:");

    // Get the current channel name and info
    let current_channel = channel_name;
    let _channel_info = if let Some(info) = channel_config.get_channel(&current_channel) {
        info
    } else {
        return Err(anyhow!("Channel info not found in config"));
    };

    // Find the latest version of this package
    install_println(format!(
        "🎣 Fetching '{current_channel}' channel binary for streamfy..."
    ));

    let install_version = match version {
        StreamfyBinVersion::Stable => fetch_latest_version(&agent, &id, &target, false).await?,
        StreamfyBinVersion::Latest => fetch_latest_version(&agent, &id, &target, true).await?,
        StreamfyBinVersion::Tag(version) => version,
        StreamfyBinVersion::Dev => return Err(anyhow!("Dev channel builds are not published")),
    };

    let id = id.into_versioned(install_version.into());

    // Download the package file from the package registry
    install_println(format!(
        "⏳ Downloading Streamfy CLI with latest version: {}...",
        id.version()
    ));
    let package_result = fetch_package_file(&agent, &id, &target).await;
    let package_file = match package_result {
        Ok(pf) => pf,
        Err(_e) => {
            install_println(format!(
                "❕ Streamfy is not published at version {} for {}, skipping self-update",
                id.version(),
                target
            ));
            return Ok(());
        }
    };
    install_println("🔑 Downloaded and verified package file");

    // Install the update over the current executable
    let streamfy_path = if let Some(c) = channel_config.config().channel().get(&current_channel) {
        c.clone().binary_location
    } else {
        return Err(anyhow!("Channel binary location not found"));
    };

    install_bin(&streamfy_path, package_file)?;
    install_println(format!(
        "✅ Successfully updated {}",
        streamfy_path.display(),
    ));

    Ok(())
}

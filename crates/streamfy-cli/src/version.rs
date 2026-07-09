use sha2::{Digest, Sha256};
use clap::Parser;
use anyhow::Result;

use streamfy::Streamfy;
use streamfy::config::ConfigFile;
use streamfy_cli_common::version_cmd::{StreamfyVersionPrinter, os_info};
use streamfy_extension_common::target::ClusterTarget;
use streamfy_channel::STREAMFY_RELEASE_CHANNEL;

use crate::metadata::subcommand_metadata;

#[derive(Debug, Parser)]
pub struct VersionOpt {
    #[clap(short, long)]
    /// Output in JSON format
    pub json: bool,
}

impl VersionOpt {
    pub async fn process(self, target: ClusterTarget) -> Result<()> {
        let mut version_printer =
            StreamfyVersionPrinter::new("Streamfy CLI", crate::VERSION.trim());

        if let Ok(channel_name) = std::env::var(STREAMFY_RELEASE_CHANNEL) {
            version_printer.append_extra("Release Channel", channel_name);
        };

        if let Some(sha) = self.format_frontend_sha() {
            version_printer.append_extra("Streamfy Channel Frontend SHA256", sha);
        }

        let platform = self.format_platform_version(target).await;
        version_printer.append_extra("Streamfy Platform", platform);

        version_printer.append_extra("Git Commit", env!("GIT_HASH"));

        if let Some(info) = os_info() {
            version_printer.append_extra("OS Details", info);
        }

        if self.json {
            println!("{}", version_printer.to_json_pretty()?);
            return Ok(());
        }

        println!("{version_printer}");

        if let Some(metadata) = self.format_subcommand_metadata()
            && !metadata.is_empty()
        {
            println!("=== Plugin Versions ===");

            for (name, version) in metadata {
                self.print_width(&name, &version, 30);
            }
        }

        Ok(())
    }

    fn print_width(&self, name: &str, version: &str, width: usize) {
        println!("{name:width$} : {version}");
    }

    // Read streamfy frontend (streamfy-channel)
    // (assuming it is named `streamfy` alongside a CLI named with its channel name (i.e. streamfy-stable))
    fn format_frontend_sha(&self) -> Option<String> {
        let streamfy_cli = std::env::current_exe().ok()?;
        let mut streamfy_frontend_path = streamfy_cli;
        streamfy_frontend_path.set_file_name("streamfy");

        let streamfy_cli_bin = std::fs::read(streamfy_frontend_path).ok()?;
        let mut hasher = Sha256::new();
        hasher.update(streamfy_cli_bin);
        let streamfy_cli_bin_sha256 = hasher.finalize();
        Some(format!("{:x}", &streamfy_cli_bin_sha256))
    }

    async fn format_platform_version(&self, target: ClusterTarget) -> String {
        // Attempt to connect to a Streamfy cluster to get platform version
        // Even if we fail to connect, we should not fail the other printouts
        let mut platform_version = String::from("Not available");
        if let Ok(streamfy_config) = target.load()
            && let Ok(streamfy) = Streamfy::connect_with_config(&streamfy_config).await
        {
            let version = streamfy.platform_version();
            platform_version = version.to_string();
        }

        let profile_name = ConfigFile::load(None)
            .ok()
            .and_then(|it| {
                it.config()
                    .current_profile_name()
                    .map(|name| name.to_string())
            })
            .map(|name| format!(" ({name})"))
            .unwrap_or_default();
        format!("{platform_version}{profile_name}")
    }

    fn format_subcommand_metadata(&self) -> Option<Vec<(String, String)>> {
        let metadata = subcommand_metadata().ok()?;
        let mut formats = Vec::new();
        for cmd in metadata {
            let filename = match cmd.path.file_name() {
                Some(f) => f.to_string_lossy().to_string(),
                None => continue,
            };
            let left = format!("{} ({})", cmd.meta.title, filename);
            formats.push((left, cmd.meta.version.to_string()));
        }

        Some(formats)
    }
}

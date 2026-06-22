use anyhow::Result;
use clap::Args;

use crate::STREAMFY_RELEASE_CHANNEL;

use super::{StreamfyVersionPrinter, os_info};

const VERSION: &str = include_str!("../../../../VERSION");

/// Display version information for local Streamfy CLIs using this build of
/// Streamfy details such as `VERSION`, `GIT_HASH`, and `STREAMFY_RELEASE_CHANNEL`.
#[derive(Debug, Args)]
pub struct BasicVersionCmd {
    #[cfg(feature = "serde")]
    #[clap(short, long)]
    /// Output in JSON format
    pub json: bool,
}

impl BasicVersionCmd {
    /// Display basic information about the current streamfy installation
    ///
    /// The following information is displayed:
    /// - Release channel, if available;
    /// - CLI version;
    /// - Platform arch;
    /// - CLI SHA256, if available;
    /// - Git hash, if available;
    /// - OS details, if available;
    pub fn process(self, cli_name: &str) -> Result<()> {
        let mut streamfy_version_printer = StreamfyVersionPrinter::new(cli_name, VERSION);

        if let Ok(channel) = std::env::var(STREAMFY_RELEASE_CHANNEL) {
            streamfy_version_printer.append_extra("Release Channel", channel);
        }

        if let Ok(git_hash) = std::env::var("GIT_HASH") {
            streamfy_version_printer.append_extra("Git Commit", git_hash);
        }

        if let Some(info) = os_info() {
            streamfy_version_printer.append_extra("OS Details", info);
        }

        #[cfg(feature = "serde")]
        {
            if self.json {
                println!("{}", streamfy_version_printer.to_json_pretty()?);
                return Ok(());
            }
        }

        println!("{streamfy_version_printer}");

        Ok(())
    }
}

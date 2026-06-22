use std::sync::Arc;

use clap::ValueEnum;
use clap::Parser;
use common::installation::InstallationType;
use streamfy::config::ConfigFile;
use semver::Version;
use tracing::debug;

mod group;
mod spu;
mod start;
mod resume;
mod delete;
mod util;
mod check;
mod error;
mod diagnostics;
mod status;
mod shutdown;
mod upgrade;

use start::StartOpt;
use resume::ResumeOpt;
use delete::DeleteOpt;
use check::CheckOpt;
use group::SpuGroupCmd;
use spu::SpuCmd;
use diagnostics::DiagnosticsOpt;
use status::StatusOpt;
use shutdown::ShutdownOpt;
use upgrade::UpgradeOpt;

pub use self::error::ClusterCliError;

use anyhow::Result;

use streamfy_extension_common as common;
use common::target::ClusterTarget;
use common::output::Terminal;
use streamfy_channel::{ImageTagStrategy, STREAMFY_IMAGE_TAG_STRATEGY};

pub(crate) const VERSION: &str = include_str!("../../../../VERSION");

/// Manage and view Streamfy clusters
#[derive(Debug, Parser)]
pub enum ClusterCmd {
    /// Install Streamfy cluster
    #[command(name = "start")]
    Start(Box<StartOpt>),

    /// Resume Streamfy cluster
    #[command(name = "resume")]
    Resume(ResumeOpt),

    /// Upgrades an already-started Streamfy cluster
    #[command(name = "upgrade")]
    Upgrade(Box<UpgradeOpt>),

    /// Uninstall a Streamfy cluster
    #[command(name = "delete")]
    Delete(DeleteOpt),

    /// Check that all requirements for cluster startup are met.
    ///
    /// This command is useful to check if user has all the required dependencies and permissions to run
    /// streamfy cluster.
    #[command(name = "check")]
    Check(CheckOpt),

    /// Manage and view Streaming Processing Units (SPUs)
    ///
    /// SPUs make up the part of a Streamfy cluster which is in charge
    /// of receiving messages from producers, storing those messages,
    /// and relaying them to consumers. This command lets you see
    /// the status of SPUs in your cluster.
    #[command(subcommand, name = "spu")]
    SPU(SpuCmd),

    /// Manage and view SPU Groups (SPGs)
    ///
    /// SPGs are groups of SPUs in a cluster which are managed together.
    #[command(subcommand, name = "spg")]
    SPUGroup(SpuGroupCmd),

    /// Collect anonymous diagnostic information to help with debugging
    #[command(name = "diagnostics")]
    Diagnostics(DiagnosticsOpt),

    /// Check the status of a Streamfy cluster
    #[command(name = "status")]
    Status(StatusOpt),

    /// Shutdown cluster processes without deleting data (alias: stop)
    #[command(name = "shutdown", alias = "stop")]
    Shutdown(ShutdownOpt),
}

impl ClusterCmd {
    /// process cluster commands
    pub async fn process<O: Terminal>(
        self,
        out: Arc<O>,
        platform_version: Version,
        target: ClusterTarget,
    ) -> Result<()> {
        match self {
            Self::Start(mut start) => {
                if let Ok(tag_strategy_value) = std::env::var(STREAMFY_IMAGE_TAG_STRATEGY) {
                    let tag_strategy = ImageTagStrategy::from_str(&tag_strategy_value, true)
                        .unwrap_or(ImageTagStrategy::Version);
                    match tag_strategy {
                        ImageTagStrategy::Version => {
                            debug!("Using image version: {}", VERSION);
                        }
                        ImageTagStrategy::VersionGit => {
                            let image_version = format!("{}-{}", VERSION, env!("GIT_HASH"));
                            debug!("Using image version: {:?}", &image_version);
                            start.k8_config.image_version = Some(image_version);
                        }
                        ImageTagStrategy::Git => {
                            debug!("Using developer image version: {}", env!("GIT_HASH"));
                            start.develop = true
                        }
                    }
                };

                start.process(platform_version, false).await?;
            }
            Self::Resume(opt) => {
                opt.process(platform_version).await?;
            }
            Self::Upgrade(mut upgrade) => {
                if let Ok(tag_strategy_value) = std::env::var(STREAMFY_IMAGE_TAG_STRATEGY) {
                    let tag_strategy = ImageTagStrategy::from_str(&tag_strategy_value, true)
                        .unwrap_or(ImageTagStrategy::Version);
                    match tag_strategy {
                        ImageTagStrategy::Version => {}
                        ImageTagStrategy::VersionGit => {
                            let image_version = format!("{}-{}", VERSION, env!("GIT_HASH"));
                            upgrade.start.k8_config.image_version = Some(image_version);
                        }
                        ImageTagStrategy::Git => upgrade.start.develop = true,
                    }
                };

                upgrade.process(platform_version).await?;
            }
            Self::Delete(uninstall) => {
                uninstall.process().await?;
            }
            Self::Check(check) => {
                check.process(platform_version).await?;
            }
            Self::SPU(spu) => {
                let streamfy = target.connect().await?;
                spu.process(out, &streamfy).await?;
            }
            Self::SPUGroup(group) => {
                let streamfy = target.connect().await?;
                group.process(out, &streamfy).await?;
            }
            Self::Diagnostics(opt) => {
                opt.process().await?;
            }
            Self::Status(status) => {
                status.process(target).await?;
            }
            Self::Shutdown(opt) => {
                opt.process().await?;
            }
        }

        Ok(())
    }
}

pub(crate) fn get_installation_type() -> Result<(InstallationType, ConfigFile)> {
    let config = ConfigFile::load_default_or_new()?;
    let fc = config.config().current_cluster()?;
    let itype = InstallationType::load(fc);
    Ok((itype, config))
}

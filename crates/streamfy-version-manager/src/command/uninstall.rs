//! Install Command
//!
//! Downloads and stores the sepecific Streamfy Version binaries in the local
//! SVM cache.

use anyhow::Result;
use clap::Parser;

use colored::Colorize;
use streamfy_artifacts_util::svm::Channel;

use crate::common::notify::Notify;

use crate::common::version_directory::VersionDirectory;

use crate::common::workdir::svm_versions_path;

/// The `install` command is responsible of installing the desired Package Set
#[derive(Debug, Parser)]
pub struct UninstallOpt {
    /// Version to install: stable, latest, or named-version x.y.z
    #[arg(index = 1, default_value_t = Channel::Stable)]
    version: Channel,
}

impl UninstallOpt {
    pub async fn process(&self, notify: Notify) -> Result<()> {
        let versions_path = svm_versions_path()?;

        if !versions_path.exists() {
            notify.warn("No versions installed");
            return Ok(());
        }

        let pkgset_path = versions_path.join(self.version.to_string());

        if !pkgset_path.exists() {
            notify.warn(format!(
                "Streamfy version {} is not installed",
                self.version.to_string().bold()
            ));

            return Ok(());
        }

        let version_directory = VersionDirectory::open(pkgset_path)?;
        version_directory.remove()?;

        Ok(())
    }
}

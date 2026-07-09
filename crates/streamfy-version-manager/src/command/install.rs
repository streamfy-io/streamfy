//! Install Command
//!
//! Downloads and stores the sepecific Streamfy Version binaries in the local
//! SVM cache.

use std::fs::create_dir_all;

use anyhow::Result;
use clap::Parser;

use streamfy_artifacts_util::svm::{Client, Channel};

use crate::common::TARGET;
use crate::common::notify::Notify;
use crate::common::version_installer::VersionInstaller;
use crate::common::workdir::svm_versions_path;

/// The `install` command is responsible of installing the desired Package Set
#[derive(Debug, Parser)]
pub struct InstallOpt {
    /// Binaries architecture triple to use
    #[arg(long, env = "SVM_BINARY_ARCH_TRIPLE", default_value = TARGET)]
    target: String,
    /// Version to install: stable, latest, or named-version x.y.z
    #[arg(index = 1, default_value_t = Channel::Stable)]
    version: Channel,
}

impl InstallOpt {
    pub async fn process(&self, notify: Notify) -> Result<()> {
        let versions_path = svm_versions_path()?;

        if !versions_path.exists() {
            tracing::info!(?versions_path, "Creating versions directory");
            create_dir_all(&versions_path)?;
        }

        let client = Client;
        let pkgset = client
            .fetch_default_package_set(&self.version, &self.target)
            .await?;

        VersionInstaller::new(self.version.to_owned(), pkgset, notify)
            .install()
            .await
    }
}

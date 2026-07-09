use std::env::var;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use semver::Version;
use octocrab::Octocrab;

use streamfy_artifacts_util::{REPO_NAME, REPO_OWNER};

use crate::{
    common::{notify::Notify, update_manager::UpdateManager},
    VERSION,
};

/// Environment variable to store the version of SVM to fetch
const SVM_UPDATE_VERSION: &str = "SVM_UPDATE_VERSION";

#[derive(Clone, Debug, Parser)]
pub struct SelfUpdateOpt;

// https://packages.streamfy.io/v1/packages/streamfy/svm/0.11.0/aarch64-apple-darwin/svm
impl SelfUpdateOpt {
    pub async fn process(&self, notify: Notify) -> Result<()> {
        let update_manager = UpdateManager::new(&notify);
        let next_version = self.resolve_version().await?;

        if next_version.to_string() != VERSION {
            notify.info(format!(
                "Updating SVM from {} to {}",
                VERSION.red(),
                next_version.to_string().green(),
            ));
            update_manager.update(&next_version).await?;
            return Ok(());
        }

        notify.info("Already up-to-date");
        Ok(())
    }

    /// Determines the version of SVM to fetch taking into account
    /// the environment variable `SVM_UPDATE_VERSION` and the `stable` channel
    async fn resolve_version(&self) -> Result<Version> {
        if let Ok(version) = var(SVM_UPDATE_VERSION) {
            return Ok(Version::parse(&version)?);
        }

        self.fetch_stable_tag().await
    }

    /// Fetches the `stable` channel tag from the Streamfy Version Manager
    async fn fetch_stable_tag(&self) -> Result<Version> {
        let octocrab = Octocrab::builder().build()?;

        // Use GitHub latest release for streamfy-io/streamfy (non-prerelease)
        let release = octocrab
            .repos(REPO_OWNER, REPO_NAME)
            .releases()
            .get_latest()
            .await
            .map_err(|e| anyhow::anyhow!("Unable to retrieve stable release for SVM: {e}"))?;

        let tag = release.tag_name;

        let tag = tag.trim_start_matches('v');
        let version = Version::parse(tag)?;

        Ok(version)
    }
}

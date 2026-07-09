use std::path::PathBuf;

use anyhow::{bail, Result};
use semver::Version;
use tempfile::TempDir;

use streamfy_artifacts_util::svm::{Client as SvmClient, Channel as SvmChannel, Download as _};

use crate::common::executable::{remove_svm_binary_if_exists, set_executable_mode};

use super::notify::Notify;
use super::workdir::svm_bin_path;
use super::TARGET;

/// Updates Manager for the Streamfy Version Manager
pub struct UpdateManager {
    notify: Notify,
}

impl UpdateManager {
    pub fn new(notify: &Notify) -> Self {
        Self {
            notify: notify.to_owned(),
        }
    }

    pub async fn update(&self, version: &Version) -> Result<()> {
        self.notify.info(format!("Downloading svm@{version}"));
        let (_tmp_dir, new_svm_bin) = self.download(version).await?;

        self.notify.info(format!("Installing svm@{version}"));
        self.install(&new_svm_bin).await?;
        self.notify
            .done(format!("Installed svm@{version} with success"));

        Ok(())
    }

    /// Downloads Streamfy Version Manager binary into a temporary directory
    async fn download(&self, version: &Version) -> Result<(TempDir, PathBuf)> {
        let tmp_dir = TempDir::new()?;
        let channel = SvmChannel::Tag(version.clone());
        let client = SvmClient;

        // Fetch the unfiltered package set for the requested version and
        // current target so that the `svm` binary artifact is included.
        let package_set = client.fetch_package_set(&channel, TARGET).await?;

        // Locate the SVM artifact within the package set
        let Some(svm_artifact) = package_set
            .artifacts
            .iter()
            .find(|artifact| artifact.name == "svm")
        else {
            bail!("SVM artifact not found in package set for version {version}");
        };

        // Require a SHA-256 digest for the SVM artifact so that integrity
        // verification is enforced during download. If the digest is missing,
        // we abort the self-update rather than proceeding unchecked.
        if svm_artifact.sha256_digest.is_none() {
            bail!(
                "Integrity verification unavailable for SVM artifact (missing sha256 digest) for version {version}. Please use a newer version of SVM to update."
            );
        }

        let out_path = svm_artifact.download(tmp_dir.path().to_path_buf()).await?;

        set_executable_mode(&out_path)?;

        Ok((tmp_dir, out_path))
    }

    async fn install(&self, new_svm_bin: &PathBuf) -> Result<()> {
        let old_svm_bin = svm_bin_path()?;

        if !new_svm_bin.exists() {
            tracing::warn!(?new_svm_bin, "New svm binary not found. Aborting update.");
            bail!("Failed to update SVM due to missing binary");
        }

        remove_svm_binary_if_exists()?;

        tracing::warn!(src=?new_svm_bin, dst=?old_svm_bin , "Copying new svm binary");
        std::fs::copy(new_svm_bin, &old_svm_bin)?;

        Ok(())
    }
}

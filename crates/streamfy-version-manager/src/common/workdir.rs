//! The `Workdir` is the directory used by Streamfy Version Manager (SVM) to
//! store its files and binaries.

use std::path::PathBuf;
use std::env::var;

use anyhow::Result;

use super::home_dir;

/// Home Directory for Streamfy
pub const STREAMFY_HOME_DIR: &str = ".streamfy";

/// Home Directory for the Streamfy Version Manager (SVM) CLI
pub const SVM_HOME_DIR: &str = ".svm";

/// SVM Binary Name
pub const SVM_BINARY_NAME: &str = "svm";

/// SVM Versions Directory Name
///
/// Here is where all the versions are stored
pub const SVM_VERSIONS_DIR: &str = "versions";

/// SVM Workdir Name Environment Variable
pub const SVM_WORKDIR_NAME_ENV_VAR: &str = "SVM_WORKDIR_NAME";

/// Retrieves the path to the `~/.svm` directory in the host system
pub fn svm_workdir_path() -> Result<PathBuf> {
    let svm_path = home_dir()?;

    if let Ok(workdir_name) = var(SVM_WORKDIR_NAME_ENV_VAR) {
        tracing::warn!("Using custom SVM workdir name: {}", workdir_name);

        Ok(svm_path.join(workdir_name))
    } else {
        Ok(svm_path.join(SVM_HOME_DIR))
    }
}

/// Retrieves the path to the `~/.svm/bin/svm` binary in the host system
pub fn svm_bin_path() -> Result<PathBuf> {
    Ok(svm_workdir_path()?.join("bin").join(SVM_BINARY_NAME))
}

/// Retrieves the path to the `~/.svm/versions` directory in the host system
pub fn svm_versions_path() -> Result<PathBuf> {
    Ok(svm_workdir_path()?.join(SVM_VERSIONS_DIR))
}

/// Retrieves the path to the `~/.streamfy` directory in the host system.
pub fn streamfy_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(STREAMFY_HOME_DIR))
}

/// Retrieves the path to the `~/.streamfy/bin` directory in the host system.
pub fn streamfy_binaries_path() -> Result<PathBuf> {
    Ok(streamfy_path()?.join("bin"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svm_workdir_path() {
        let svm_path = svm_workdir_path().expect("Failed to get svm path");
        let home = home_dir().expect("Failed to get home directory");

        assert_eq!(svm_path, home.join(SVM_HOME_DIR));
    }

    #[test]
    fn test_svm_bin_path() {
        let svm_bin_path = svm_bin_path().expect("Failed to get svm bin path");
        let svm_path = svm_workdir_path().expect("Failed to get svm path");

        assert_eq!(svm_bin_path, svm_path.join("bin").join(SVM_BINARY_NAME));
    }

    #[test]
    fn test_svm_versions_path() {
        let svm_version_path = svm_versions_path().expect("Failed to get svm pkgset path");
        let svm_path = svm_workdir_path().expect("Failed to get svm path");

        assert_eq!(svm_version_path, svm_path.join(SVM_VERSIONS_DIR));
    }

    #[test]
    fn test_streamfy_path() {
        let streamfy_path = streamfy_path().expect("Failed to get streamfy path");
        let home = home_dir().expect("Failed to get home directory");

        assert_eq!(streamfy_path, home.join(STREAMFY_HOME_DIR));
    }

    #[test]
    fn test_streamfy_binaries_path() {
        let streamfy_binaries_path =
            streamfy_binaries_path().expect("Failed to get streamfy binaries path");
        let streamfy_path = streamfy_path().expect("Failed to get streamfy path");

        assert_eq!(streamfy_binaries_path, streamfy_path.join("bin"));
    }
}

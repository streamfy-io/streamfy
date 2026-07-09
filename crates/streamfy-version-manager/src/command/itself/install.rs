use std::env::current_exe;
use std::fs::{copy, create_dir_all, write};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;

use crate::common::executable::remove_svm_binary_if_exists;
use crate::common::notify::Notify;
use crate::common::settings::Settings;
use crate::common::workdir::{svm_bin_path, svm_workdir_path, svm_versions_path};

const SVM_ENV_FILE_CONTENTS: &str = r#"
#!/bin/sh
case ":${PATH}:" in
    *:"$HOME/.svm/bin":*)
        ;;
    *)
        export PATH="$PATH:$HOME/.svm/bin:$HOME/.streamfy/bin"
        ;;
esac
"#;

#[derive(Clone, Debug, Parser)]
pub struct SelfInstallOpt;

impl SelfInstallOpt {
    pub async fn process(&self, notify: Notify) -> Result<()> {
        let svm_installation_path = self.install_svm()?;

        Settings::open()?;

        notify.done(format!(
            "SVM installed successfully at {}",
            svm_installation_path.display()
        ));
        notify.help(format!("Add SVM to PATH using {}", "source $HOME/.svm/env"));

        Ok(())
    }

    /// Creates the `~/.svm` directory and copies the current binary to this
    /// directory.
    ///
    /// # Usage of `create_dir` over `create_dir_all`
    ///
    /// Given that on updates the directories may be present, to avoid failing
    /// on `create_dir`, `create_dir_all` is used instead.
    ///
    /// Something similar happens on `mkdir` command, even though underlaying
    /// syscalls may differ.
    ///
    /// Consider the existent directory `~/.svm/versions`, executing `create_dir`
    /// will fail with error:
    ///
    /// ```ignore
    /// ~/.svm/versions: File exists
    /// ```
    ///
    /// Instead by doing `create_dir_all` the error will not happen.
    ///
    /// ```ignore
    /// mkdir -p ~/.svm/versions
    /// ```
    ///
    fn install_svm(&self) -> Result<PathBuf> {
        // Creates the directory `~/.svm` if doesn't exists
        let svm_dir = svm_workdir_path()?;

        // Creates the binaries directory
        let bin_dir = svm_dir.join("bin");
        create_dir_all(bin_dir)?;

        let svm_binary_path = svm_bin_path()?;
        let current_binary_path = current_exe()?;

        if svm_binary_path == current_binary_path {
            // We cant replace ourselves, user is running `svm self install`
            // from the binary itself and not from the installer script.
            return Err(anyhow::anyhow!("SVM is already installed"));
        }

        remove_svm_binary_if_exists()?;

        // Copies "this" binary to the SVM binary directory
        copy(current_binary_path.clone(), svm_binary_path.clone()).map_err(|e| {
            anyhow!(
                "Couldn't copy svm from {} to {} with error {}",
                current_binary_path.display(),
                svm_binary_path.display(),
                e
            )
        })?;
        tracing::debug!(
            ?svm_dir,
            "Copied the SVM binary to the SVM home directory with success"
        );

        // Creates the package set directory
        let svm_pkgset_dir = svm_versions_path()?;
        create_dir_all(svm_pkgset_dir)?;

        // Creates the `env` file
        let svm_env_file_path = svm_dir.join("env");
        write(svm_env_file_path, SVM_ENV_FILE_CONTENTS)?;

        Ok(svm_dir)
    }
}

use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use anyhow::Result;

use fluvio_command::CommandExt;
use streamfy_extension_common::StreamfyExtensionMetadata;

use crate::client::client_metadata;

#[derive(Debug, Parser)]
pub struct MetadataOpt {}
impl MetadataOpt {
    pub fn process(self) -> Result<()> {
        let metadata = Self::metadata();
        if let Ok(out) = serde_json::to_string(&metadata) {
            println!("{out}");
        }

        Ok(())
    }

    fn metadata() -> Vec<StreamfyExtensionMetadata> {
        let mut metadata = client_metadata();

        if let Ok(subcommand_meta) = subcommand_metadata() {
            let extension_meta = subcommand_meta.into_iter().map(|it| it.meta);
            metadata.extend(extension_meta);
        }

        metadata
    }
}

#[derive(Debug)]
pub struct SubcommandMetadata {
    pub path: PathBuf,
    pub meta: StreamfyExtensionMetadata,
}

/// Collects the metadata of Streamfy extensions installed on the system
pub fn subcommand_metadata() -> Result<Vec<SubcommandMetadata>> {
    let mut metadata = Vec::new();

    let extensions = streamfy_cli_common::install::get_extensions()?;
    for path in extensions {
        let result = Command::new(&path).arg("metadata").result();
        let output = match result {
            Ok(out) => out.stdout,
            _ => continue,
        };

        let json_result = serde_json::from_slice::<StreamfyExtensionMetadata>(&output);
        if let Ok(meta) = json_result {
            let subcommand = SubcommandMetadata { path, meta };
            metadata.push(subcommand);
        }
    }

    Ok(metadata)
}

use std::sync::Arc;
use std::fmt::Debug;

use async_trait::async_trait;

use tracing::debug;
use clap::Parser;
use anyhow::Result;

use streamfy::Streamfy;
use streamfy_extension_common::Terminal;
use streamfy::metadata::smartmodule::SmartModuleSpec;

use crate::client::cmd::ClientCmd;
use crate::error::CliError;

/// Delete an existing SmartModule with the given name
#[derive(Debug, Parser)]
pub struct DeleteSmartModuleOpt {
    /// Continue deleting in case of an error
    #[arg(short, long, required = false)]
    continue_on_error: bool,
    /// One or more name(s) of the smartmodule(s) to be deleted
    #[arg(value_name = "name", required = true)]
    names: Vec<String>,
}

#[async_trait]
impl ClientCmd for DeleteSmartModuleOpt {
    async fn process_client<O: Terminal + Debug + Send + Sync>(
        self,
        _out: Arc<O>,
        streamfy: &Streamfy,
    ) -> Result<()> {
        let admin = streamfy.admin().await;
        let mut err_happened = false;
        for name in self.names.iter() {
            debug!(name, "deleting smartmodule");
            if let Err(error) = admin.delete::<SmartModuleSpec>(name).await {
                err_happened = true;
                if self.continue_on_error {
                    println!("smart module \"{name}\" delete failed with: {error}");
                } else {
                    return Err(error);
                }
            } else {
                println!("smartmodule \"{name}\" deleted");
            }
        }
        if err_happened {
            Err(CliError::CollectedError(
                "Failed deleting smart module(s). Check previous errors.".to_string(),
            )
            .into())
        } else {
            Ok(())
        }
    }
}

//!
//! # Delete Managed SPU Groups
//!
//! CLI tree to generate Delete Managed SPU Groups
//!
use clap::Parser;
use anyhow::Result;

use streamfy::Streamfy;
use streamfy::metadata::spg::SpuGroupSpec;

// -----------------------------------
// CLI Options
// -----------------------------------

#[derive(Debug, Parser)]
pub struct DeleteManagedSpuGroupOpt {
    /// The name of the SPU Group to delete
    #[arg(value_name = "name")]
    name: String,
}

impl DeleteManagedSpuGroupOpt {
    pub async fn process(self, streamfy: &Streamfy) -> Result<()> {
        let admin = streamfy.admin().await;
        admin.delete::<SpuGroupSpec>(&self.name).await?;
        Ok(())
    }
}

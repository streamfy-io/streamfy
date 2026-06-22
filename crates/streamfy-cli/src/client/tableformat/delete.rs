//!
//! # Delete TableFormat spec
//!
//! CLI tree to generate Delete TableFormat spec
//!
use clap::Parser;
use anyhow::Result;

use streamfy::Streamfy;
use streamfy::metadata::tableformat::TableFormatSpec;

// -----------------------------------
// CLI Options
// -----------------------------------

#[derive(Debug, Parser)]
pub struct DeleteTableFormatOpt {
    /// The name of the table format to delete
    name: String,
}

impl DeleteTableFormatOpt {
    pub async fn process(self, streamfy: &Streamfy) -> Result<()> {
        let admin = streamfy.admin().await;
        admin.delete::<TableFormatSpec>(&self.name).await?;
        Ok(())
    }
}

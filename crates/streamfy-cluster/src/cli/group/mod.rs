use std::sync::Arc;
use clap::Parser;

mod create;
mod delete;
mod list;

use anyhow::Result;

use streamfy::Streamfy;
use crate::cli::common::output::Terminal;
use crate::cli::common::COMMAND_TEMPLATE;

use create::CreateManagedSpuGroupOpt;
use delete::DeleteManagedSpuGroupOpt;
use list::ListManagedSpuGroupsOpt;

#[derive(Debug, Parser)]
pub enum SpuGroupCmd {
    /// Create a new managed SPU Group
    #[command(
        name = "create",
        help_template = COMMAND_TEMPLATE,
    )]
    Create(CreateManagedSpuGroupOpt),

    /// Delete a managed SPU Group
    #[command(
        name = "delete",
        help_template = COMMAND_TEMPLATE,
    )]
    Delete(DeleteManagedSpuGroupOpt),

    /// List all SPU Groups
    #[command(
        name = "list",
        help_template = COMMAND_TEMPLATE,
    )]
    List(ListManagedSpuGroupsOpt),
}

impl SpuGroupCmd {
    pub async fn process<O: Terminal>(self, out: Arc<O>, streamfy: &Streamfy) -> Result<()> {
        match self {
            Self::Create(create) => {
                create.process(streamfy).await?;
            }
            Self::Delete(delete) => {
                delete.process(streamfy).await?;
            }
            Self::List(list) => {
                list.process(out, streamfy).await?;
            }
        }
        Ok(())
    }
}

mod list;
mod delete;

pub use cmd::ConsumerCmd;

mod cmd {

    use std::sync::Arc;
    use std::fmt::Debug;

    use async_trait::async_trait;
    use clap::Parser;
    use anyhow::Result;

    use streamfy::Streamfy;

    use crate::client::cmd::ClientCmd;
    use crate::common::output::Terminal;
    use crate::common::StreamfyExtensionMetadata;

    use super::delete::DeleteConsumerOpt;
    use super::list::ListConsumerOpt;

    #[derive(Debug, Parser)]
    #[command(name = "consumer", about = "Consumer operations")]
    pub enum ConsumerCmd {
        /// List all of the Consumer Offsets in this cluster
        #[command(
            name = "list",
            help_template = crate::common::COMMAND_TEMPLATE,
        )]
        List(ListConsumerOpt),
        /// Delete the Consumer Offset
        #[command(
            name = "delete",
            help_template = crate::common::COMMAND_TEMPLATE,
        )]
        Delete(DeleteConsumerOpt),
    }

    #[async_trait]
    impl ClientCmd for ConsumerCmd {
        async fn process_client<O: Terminal + Debug + Send + Sync>(
            self,
            out: Arc<O>,
            streamfy: &Streamfy,
        ) -> Result<()> {
            match self {
                Self::List(list) => {
                    list.process(out, streamfy).await?;
                }
                Self::Delete(delete) => {
                    delete.process(out, streamfy).await?;
                }
            }

            Ok(())
        }
    }

    impl ConsumerCmd {
        pub fn metadata() -> StreamfyExtensionMetadata {
            StreamfyExtensionMetadata {
                title: "consumer".into(),
                package: Some("streamfy/streamfy".parse().unwrap()),
                description: "Consumer Offsets Operations".into(),
                version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            }
        }
    }
}

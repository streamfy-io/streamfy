mod list;

pub use cmd::PartitionCmd;

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

    use super::list::ListPartitionOpt;

    #[derive(Debug, Parser)]
    #[command(name = "partition", about = "Partition operations")]
    pub enum PartitionCmd {
        /// List all of the Partitions in this cluster
        #[command(
            name = "list",
            help_template = crate::common::COMMAND_TEMPLATE,
        )]
        List(ListPartitionOpt),
    }

    #[async_trait]
    impl ClientCmd for PartitionCmd {
        async fn process_client<O: Terminal + Debug + Send + Sync>(
            self,
            out: Arc<O>,
            streamfy: &Streamfy,
        ) -> Result<()> {
            match self {
                Self::List(list) => {
                    list.process(out, streamfy).await?;
                }
            }

            Ok(())
        }
    }

    impl PartitionCmd {
        pub fn metadata() -> StreamfyExtensionMetadata {
            StreamfyExtensionMetadata {
                title: "partition".into(),
                package: Some("streamfy/streamfy".parse().unwrap()),
                description: "Partition Operations".into(),
                version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            }
        }
    }
}

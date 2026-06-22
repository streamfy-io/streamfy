mod topic;
mod consume;
mod produce;
mod partition;
mod tableformat;
mod smartmodule;
mod smartmodule_invocation;
mod consumer;
mod remote;
mod home;

pub use metadata::client_metadata;
pub use cmd::StreamfyCmd;
pub use tableformat::TableFormatConfig;
use cmd::ClientCmd;
mod metadata {

    use streamfy_extension_common::StreamfyExtensionMetadata;

    use super::topic::TopicCmd;
    use super::partition::PartitionCmd;
    use super::produce::ProduceOpt;
    use super::consume::ConsumeOpt;

    /// return list of metadata associated with client
    pub fn client_metadata() -> Vec<StreamfyExtensionMetadata> {
        vec![
            TopicCmd::metadata(),
            PartitionCmd::metadata(),
            ProduceOpt::metadata(),
            ConsumeOpt::metadata(),
        ]
    }
}

mod cmd {

    use std::sync::Arc;
    use std::fmt::Debug;

    use clap::Parser;
    use async_trait::async_trait;
    use anyhow::Result;

    use streamfy::Streamfy;

    use crate::common::target::ClusterTarget;
    use crate::common::Terminal;

    use super::consumer::ConsumerCmd;
    use super::remote::RemoteCmd;
    use super::home::HomeCmd;
    use super::smartmodule::SmartModuleCmd;
    use super::consume::ConsumeOpt;
    use super::produce::ProduceOpt;
    use super::topic::TopicCmd;
    use super::partition::PartitionCmd;
    use super::tableformat::TableFormatCmd;

    #[async_trait]
    pub trait ClientCmd: Sized {
        /// handle the command based on target
        async fn process<O: Terminal + Send + Sync + Debug>(
            self,
            out: Arc<O>,
            target: ClusterTarget,
        ) -> Result<()> {
            let mut streamfy_config = target.load()?;
            let client_id = match std::env::var("STREAMFY_CLIENT_ID") {
                Ok(id) => id,
                Err(_) => "STREAMFY_CLI".to_owned(),
            };
            streamfy_config.client_id = Some(client_id);
            let streamfy = Streamfy::connect_with_config(&streamfy_config).await?;
            self.process_client(out, &streamfy).await?;
            Ok(())
        }

        /// process client
        async fn process_client<O: Terminal + Debug + Send + Sync>(
            self,
            out: Arc<O>,
            streamfy: &Streamfy,
        ) -> Result<()>;
    }

    // For some reason this doc string is the one that gets used for the top-level help menu.
    // Please don't change it unless you want to update the top-level help menu "about".
    /// Streamfy command-line interface
    #[derive(Parser, Debug)]
    pub enum StreamfyCmd {
        /// Read messages from a topic/partition
        #[command(name = "consume")]
        Consume(Box<ConsumeOpt>),

        /// Write messages to a topic/partition
        #[command(name = "produce")]
        Produce(ProduceOpt),

        /// Manage and view Topics
        ///
        /// A Topic is essentially the name of a stream which carries messages that
        /// are related to each other. Similar to the role of tables in a relational
        /// database, the names and contents of Topics will typically reflect the
        /// structure of the application domain they are used for.
        #[command(subcommand, name = "topic")]
        Topic(TopicCmd),

        /// Manage and view Partitions
        ///
        /// Partitions are a way to divide the total traffic of a single Topic into
        /// separate streams which may be processed independently. Data sent to different
        /// partitions may be processed by separate SPUs on different computers. By
        /// dividing the load of a Topic evenly among partitions, you can increase the
        /// total throughput of the Topic.
        #[command(subcommand, name = "partition")]
        Partition(PartitionCmd),

        /// Create and manage SmartModules
        ///
        /// SmartModules are compiled WASM modules used to create SmartModules.
        #[command(
            subcommand,
            name = "smartmodule",
            visible_alias = "sm",
            // FIXME: We should remove this alias when we bump the platform version to 10.x
            alias = "smart-module"
        )]
        SmartModule(SmartModuleCmd),

        /// Create a TableFormat display specification
        ///
        /// Used with the consumer output type `full_table` to
        /// describe how to render JSON data in a tabular form
        #[command(subcommand, name = "table-format", visible_alias = "tf")]
        TableFormat(TableFormatCmd),

        /// Manage and view Consumers
        #[command(subcommand, name = "consumer")]
        Consumer(ConsumerCmd),

        /// Manage and view remote clusters mirrored
        #[command(subcommand, name = "remote")]
        Remote(Box<RemoteCmd>),

        /// Commands to interact with the home cluster
        #[command(subcommand, name = "home")]
        Home(Box<HomeCmd>),
    }

    impl StreamfyCmd {
        /// Connect to Streamfy and pass the Streamfy client to the subcommand handlers.
        pub async fn process<O: Terminal + Debug + Send + Sync>(
            self,
            out: Arc<O>,
            target: ClusterTarget,
        ) -> Result<()> {
            match self {
                Self::Consume(consume) => {
                    consume.process(out, target).await?;
                }
                Self::Produce(produce) => {
                    produce.process(out, target).await?;
                }
                Self::Topic(topic) => {
                    topic.process(out, target).await?;
                }
                Self::Partition(partition) => {
                    partition.process(out, target).await?;
                }
                Self::SmartModule(smartmodule) => {
                    smartmodule.process(out, target).await?;
                }
                Self::TableFormat(tableformat) => {
                    tableformat.process(out, target).await?;
                }
                Self::Consumer(consumer) => {
                    consumer.process(out, target).await?;
                }
                Self::Remote(remote) => {
                    remote.process(out, target).await?;
                }
                Self::Home(home) => {
                    home.process(out, target).await?;
                }
            }

            Ok(())
        }
    }
}

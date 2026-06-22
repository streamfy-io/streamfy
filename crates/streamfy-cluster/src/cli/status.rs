use clap::Parser;
use colored::Colorize;
use anyhow::{Result, anyhow};

use streamfy::{Streamfy, StreamfyAdmin, StreamfyClusterConfig};
use streamfy::config::ConfigFile;
use streamfy_controlplane_metadata::partition::PartitionSpec;
use streamfy_controlplane_metadata::{spu::SpuSpec, topic::TopicSpec};
use streamfy_extension_common::installation::InstallationType;
use streamfy_sc_schema::objects::Metadata;
use tracing::debug;

use crate::CheckStatus;
use crate::check::{ActiveKubernetesCluster, ClusterCheck};
use crate::render::ProgressRenderer;
use crate::{cli::ClusterCliError, cli::ClusterTarget};
use crate::progress::ProgressBarFactory;

#[derive(Debug, Parser)]
pub struct StatusOpt;

macro_rules! pad_format {
    ( $e:expr ) => {
        format!("{:>3} {}", "", $e)
    };
}

impl StatusOpt {
    pub async fn process(self, target: ClusterTarget) -> Result<()> {
        let pb_factory = ProgressBarFactory::new(false);

        let pb = match pb_factory.create() {
            Ok(pb) => pb,
            Err(_) => return Err(anyhow!("Failed to create progress bar")),
        };

        let streamfy_config = target.load()?;
        let config_file = ConfigFile::load_default_or_new()?;
        let installation_type = InstallationType::load(config_file.config().current_cluster()?);
        debug!(?installation_type);

        pb_factory.println(format!(
            "📝 Running cluster status checks with profile {}",
            Self::profile_name(&config_file).italic()
        ));

        if let InstallationType::K8 | InstallationType::LocalK8 = installation_type {
            let _ = Self::check_k8s_cluster(&pb).await;
        }
        Self::check_sc(&pb, &streamfy_config, &config_file).await?;
        Self::check_spus(&pb, &streamfy_config).await?;
        Self::check_topics(&pb, &streamfy_config).await?;

        pb.finish_and_clear();

        Ok(())
    }

    async fn check_k8s_cluster(pb: &ProgressRenderer) -> Result<()> {
        let k8s_cluster_check = Box::new(ActiveKubernetesCluster);

        pb.set_message(pad_format!(format!(
            "{} Checking {}",
            "📝".bold(),
            k8s_cluster_check.label()
        )));

        match k8s_cluster_check.perform_check(pb).await? {
            CheckStatus::Pass(status) => {
                pb.println(pad_format!(format!("{} {}", "✅".bold(), status)));
                Ok(())
            }
            CheckStatus::Unrecoverable(err) => {
                debug!("failed: {}", err);

                pb.println(pad_format!(format!(
                    "{} Check {} failed",
                    "❌",
                    k8s_cluster_check.label().italic(),
                )));

                Err(ClusterCliError::Other(err.to_string()).into())
            }
            _ => Err(ClusterCliError::Other("Should not be reachable".to_string()).into()),
        }
    }

    async fn check_sc(
        pb: &ProgressRenderer,
        streamfy_config: &StreamfyClusterConfig,
        config_file: &ConfigFile,
    ) -> Result<()> {
        pb.set_message(pad_format!(format!("{} Checking {}", "📝".bold(), "SC")));

        match Streamfy::connect_with_config(streamfy_config).await {
            Ok(_streamfy) => {
                pb.println(pad_format!(format!("{} SC is ok", "✅".bold())));
                Ok(())
            }
            Err(err) => {
                pb.println(pad_format!(format!(
                    "{} Unable to reach cluster on profile {}",
                    "❌",
                    Self::profile_name(config_file).italic(),
                )));

                Err(ClusterCliError::Other(err.to_string()).into())
            }
        }
    }

    async fn check_spus(pb: &ProgressRenderer, streamfy_config: &StreamfyClusterConfig) -> Result<()> {
        pb.set_message(pad_format!(format!("{} Checking {}", "📝".bold(), "SPUs")));

        match StreamfyAdmin::connect_with_config(streamfy_config).await {
            Ok(admin) => {
                let filters: Vec<String> = vec![];
                let spus = admin.list::<SpuSpec, _>(filters).await?;
                let spu_count = spus.len();
                let online_spu_count = spus.iter().filter(|spu| spu.status.is_online()).count();

                if online_spu_count == 0 {
                    pb.println(pad_format!(format!("{} No SPUs are online", "❌".red())));

                    Ok(())
                } else if online_spu_count < spu_count {
                    pb.println(pad_format!(format!(
                        "{} ({}/{}) SPUs are online",
                        "🟡".yellow(),
                        online_spu_count,
                        spu_count
                    )));

                    Ok(())
                } else {
                    pb.println(pad_format!(format!(
                        "{} ({}/{}) SPUs are online",
                        "✅".bold(),
                        spu_count,
                        spu_count
                    )));

                    Ok(())
                }
            }
            Err(e) => {
                pb.println(pad_format!(format!(
                    "{} Unable to connect to SPUs",
                    "❌".bold(),
                )));

                Err(e)
            }
        }
    }

    fn profile_name(config_file: &ConfigFile) -> String {
        config_file
            .config()
            .current_profile_name()
            .unwrap()
            .to_string()
    }

    async fn check_topics(
        pb: &ProgressRenderer,
        streamfy_config: &StreamfyClusterConfig,
    ) -> Result<()> {
        pb.set_message(pad_format!(format!(
            "{} Checking {}",
            "📝".bold(),
            "Topics"
        )));

        match StreamfyAdmin::connect_with_config(streamfy_config).await {
            Ok(admin) => {
                let partitions = admin.all::<PartitionSpec>().await?;
                let topics = admin.all::<TopicSpec>().await?;
                if topics.is_empty() {
                    pb.println(pad_format!(format!("{} No topics present", "🟡".yellow(),)));

                    return Ok(());
                }

                let size = Self::total_cluster_storage(&partitions).await?;
                pb.println(pad_format!(format!(
                    "{} {} topic{} using {}",
                    "✅".bold(),
                    topics.len(),
                    if topics.len() == 1 { "" } else { "s" },
                    bytesize::ByteSize::b(size as u64),
                )));

                Ok(())
            }
            Err(e) => {
                pb.println(pad_format!(format!(
                    "{} Unable to retrieve topics",
                    "❌".bold(),
                )));

                Err(e)
            }
        }
    }

    async fn total_cluster_storage(partitions: &Vec<Metadata<PartitionSpec>>) -> Result<i64> {
        let mut cluster_total = 0;
        for partition in partitions {
            let follower_count = partition.status.replicas.len() as i64;

            // add one for the leader
            let partition_total = (1 + follower_count) * Self::partition_size(partition)?;

            cluster_total += partition_total;
        }

        Ok(cluster_total)
    }

    fn partition_size(partition: &Metadata<PartitionSpec>) -> Result<i64> {
        match partition.status.size {
            size if size < 0 => Err(ClusterCliError::Other(format!(
                "A partition has an invalid size: {size}"
            ))
            .into()),
            size => Ok(size),
        }
    }
}

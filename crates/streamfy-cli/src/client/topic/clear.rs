//!
//! # Clear Topic
//!
//! Remove all stored records from a topic without deleting the topic,
//! partitions, replicas, or consumer offsets.
//!

use std::io::{self, Write};

use clap::Parser;
use anyhow::Result;

use streamfy::Streamfy;
use streamfy_sc_schema::topic::{ClearTopic, TopicSpec, UpdateTopicAction};

/// Clear all records from a topic while preserving configuration and consumer offsets
#[derive(Debug, Parser)]
pub struct ClearTopicOpt {
    /// Name of the topic to clear
    #[arg(value_name = "name")]
    name: String,

    /// Skip interactive confirmation prompt
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

impl ClearTopicOpt {
    pub async fn process(self, streamfy: &Streamfy) -> Result<()> {
        if !self.yes && !user_confirms(&self.name)? {
            println!("Aborted");
            return Ok(());
        }

        let admin = streamfy.admin().await;
        let action = UpdateTopicAction::Clear(ClearTopic::default());
        admin.update::<TopicSpec>(self.name.clone(), action).await?;

        println!("topic \"{}\" cleared", self.name);
        Ok(())
    }
}

fn user_confirms(name: &str) -> Result<bool> {
    print!(
        "This will permanently remove all records from topic '{name}'. \
Topic configuration, partitions, replicas, and consumer offsets will be preserved.\n\
Are you sure you want to proceed? (y/n): "
    );
    io::stdout().flush()?;

    let mut ans = String::new();
    io::stdin().read_line(&mut ans)?;
    let ans = ans.trim_end().to_lowercase();
    Ok(matches!(ans.as_str(), "y" | "yes"))
}

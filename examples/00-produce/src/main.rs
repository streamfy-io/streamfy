//! A minimal example showing how to produce messages on Streamfy
//!
//! Before running this example, make sure you have created a topic
//! named `simple` with the following command:
//!
//! ```text
//! $ streamfy topic create simple
//! ```
//!
//! Run this example using the following:
//!
//! ```text
//! $ cargo run --bin produce
//! Sent simple record: Hello, Streamfy!
//! ```
//!
//! After running this example, you can see the messages that have
//! been sent to the topic using the following command:
//!
//! ```text
//! $ streamfy consume simple -B -d
//! Hello, Streamfy!
//! ```

use streamfy::RecordKey;

#[tokio::main]
async fn main() {
    if let Err(e) = produce().await {
        println!("Produce error: {e:?}");
    }
}

async fn produce() -> anyhow::Result<()> {
    let producer = streamfy::producer("simple").await?;

    let value = "Hello, Streamfy!";
    producer.send(RecordKey::NULL, value).await?;
    producer.flush().await?;
    println!("{value}");

    Ok(())
}

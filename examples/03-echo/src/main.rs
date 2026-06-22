//! A simple echo application with Streamfy, to demonstrate producing and consuming messages
//!
//! Streamfy is a streaming platform with a simple API for quick development.
//! This example demonstrates how to perform two key streaming operations:
//! producing and consuming messages.
//!
//! # Installation
//!
//! Before running this example, you need to make sure you have Streamfy installed.
//! If you haven't yet, visit our [installation page], then come back here.
//!
//! Once you have the Streamfy CLI and you have a Streamfy cluster available
//! (either through minikube or Streamfy Cloud), you're ready to get started.
//!
//! [installation page]: https://nightly.streamfy.io/docs/kubernetes/install/
//!
//! # Getting Started
//!
//! When writing a streaming application, you first "produce" messages by
//! sending them to a Streamfy cluster, then you "consume" those messages
//! somewhere else by reading them back from Streamfy. In this example, we'll
//! be producing and consuming messages from the same program, but this is
//! a simple example. In a real-world application, the producer and consumer
//! may be different programs on different machines.
//!
//! Messages must be sent to a specific Topic, which is a sort of category
//! for your events. For our echo example, we'll create a topic called "echo".
//!
//! To create your topic, run this on the command line:
//!
//! ```text
//! $ streamfy topic create echo
//! ```
//!
//! To confirm that your topic was created, we can ask Streamfy to list it back:
//!
//! ```text
//! $ streamfy topic list
//! NAME   TYPE      PARTITIONS  REPLICAS  IGNORE-RACK  STATUS                   REASON
//! echo   computed      1          1                   resolution::provisioned
//! ```
//!
//! Now, we can run the example
//!
//! ```text
//! $ cargo run --bin echo
//!    Compiling echo v0.1.0 (.../streamfy/examples/echo)
//!     Finished dev [unoptimized + debuginfo] target(s) in 6.22s
//!      Running `target/debug/echo`
//! Sending record 0
//! Got record: Hello Streamfy 0!
//! Sending record 1
//! Got record: Hello Streamfy 1!
//! Sending record 2
//! Got record: Hello Streamfy 2!
//! Sending record 3
//! Got record: Hello Streamfy 3!
//! Sending record 4
//! Got record: Hello Streamfy 4!
//! Sending record 5
//! Got record: Hello Streamfy 5!
//! Sending record 6
//! Got record: Hello Streamfy 6!
//! Sending record 7
//! Got record: Hello Streamfy 7!
//! Sending record 8
//! Got record: Hello Streamfy 8!
//! Sending record 9
//! Got record: Hello Streamfy 9!
//! Got record: Done!
//! ```
//!
//! If you want to double check that all of the messages made it into
//! the topic, you can manually consume them using the Streamfy CLI
//!
//! ```text
//! $ streamfy consume echo -B -d
//! Hello Streamfy 0!
//! Hello Streamfy 1!
//! Hello Streamfy 2!
//! Hello Streamfy 3!
//! Hello Streamfy 4!
//! Hello Streamfy 5!
//! Hello Streamfy 6!
//! Hello Streamfy 7!
//! Hello Streamfy 8!
//! Hello Streamfy 9!
//! Done!
//! ```

use std::time::Duration;
use streamfy::consumer::ConsumerConfigExtBuilder;
use streamfy::{Streamfy, Offset, RecordKey};
use futures::future::join;
use tokio::spawn;
use tokio::time::timeout;

const TOPIC: &str = "echo";
const TIMEOUT_MS: u64 = 5_000;

#[tokio::main]
async fn main() {
    let produce_handle = spawn(produce());
    let consume_handle = spawn(consume());

    let timed_result = timeout(
        Duration::from_millis(TIMEOUT_MS),
        join(produce_handle, consume_handle),
    )
    .await;

    let (produce_result, consume_result) = match timed_result {
        Ok(results) => results,
        Err(_) => {
            println!("Echo timed out after {TIMEOUT_MS}ms");
            std::process::exit(1);
        }
    };

    match (produce_result, consume_result) {
        (Err(produce_err), Err(consume_err)) => {
            println!("Echo produce error: {produce_err:?}");
            println!("Echo consume error: {consume_err:?}");
            std::process::exit(1);
        }
        (Err(produce_err), _) => {
            println!("Echo produce error: {produce_err:?}");
            std::process::exit(1);
        }
        (_, Err(consume_err)) => {
            println!("Echo consume error: {consume_err:?}");
            std::process::exit(1);
        }
        _ => (),
    }
}

/// Produces 10 "Hello, Streamfy" events, followed by a "Done!" event
async fn produce() -> anyhow::Result<()> {
    let producer = streamfy::producer(TOPIC).await?;

    for i in 0..10u32 {
        println!("Sending record {i}");
        producer
            .send(format!("Key {i}"), format!("Value {i}"))
            .await?;
    }
    producer.send(RecordKey::NULL, "Done!").await?;
    producer.flush().await?;

    Ok(())
}

/// Consumes events until a "Done!" event is read
async fn consume() -> anyhow::Result<()> {
    use futures::StreamExt;

    let streamfy = Streamfy::connect().await?;
    let mut stream = streamfy
        .consumer_with_config(
            ConsumerConfigExtBuilder::default()
                .topic(TOPIC)
                .partition(0)
                .offset_start(Offset::beginning())
                .build()?,
        )
        .await?;

    while let Some(Ok(record)) = stream.next().await {
        let key = record.get_key().map(|key| key.as_utf8_lossy_string());
        let value = record.get_value().as_utf8_lossy_string();
        println!("Got record: key={key:?}, value={value}");
        if value == "Done!" {
            return Ok(());
        }
    }

    Ok(())
}

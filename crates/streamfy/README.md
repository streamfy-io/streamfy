<div align="center">
  <h1>Streamfy</h1>
  <a href="https://streamfy.io" target="_blank">
    <strong>The programmable data streaming platform</strong>
  </a>
</div>

<div align="center">

  [![CI Status](https://github.com/streamfy/streamfy/workflows/CI/badge.svg)](https://github.com/streamfy/streamfy/actions/workflows/ci.yml)
  [![CD Status](https://github.com/streamfy/streamfy/workflows/CD_Dev/badge.svg)](https://github.com/streamfy/streamfy/actions/workflows/cd_dev.yaml)
  [![streamfy Crates.io version](https://img.shields.io/crates/v/streamfy?style=flat)](https://crates.io/crates/streamfy)
  [![Streamfy client API documentation](https://docs.rs/streamfy/badge.svg)](https://docs.rs/streamfy)
  [![Streamfy dependency status](https://deps.rs/repo/github/streamfy/streamfy/status.svg)](https://deps.rs/repo/github/streamfy/streamfy)
  [![Streamfy Discord](https://img.shields.io/discord/695712741381636168.svg?logo=discord&style=flat)](https://discordapp.com/invite/bBG2dTz)

</div>

## What's Streamfy?

Streamfy is a programmable data streaming platform written in Rust. With Streamfy
you can create performant real time applications that scale.

Read more about Streamfy in the [official website][Streamfy.io].

## Getting Started

Let's write a very simple solution with Streamfy, in the following demostration
we will create a topic using the Streamfy CLI and then we wisll produce some
records on this topic. Finally these records will be consumed from the topic
and printed to the stdout.

1. Install [Streamfy CLI][Install Streamfy CLI] if you havent already

2. Create a new topic using the CLI

```bash
streamfy topic create "echo-test"
```

3. Create a new cargo project and install `streamfy`, `futures` and `tokio`

```bash
cargo add streamfy
cargo add futures
cargo add tokio
```

4. Copy and paste the following snippet into your  `src/main.rs`

```ignore
use std::time::Duration;

use streamfy::{Offset, RecordKey};
use futures::StreamExt;

const TOPIC: &str = "echo-test";
const MAX_RECORDS: u8 = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let producer = streamfy::producer(TOPIC).await?;
    let consumer = streamfy::consumer(TOPIC, 0).await?;
    let mut consumed_records: u8 = 0;

    for i in 0..10 {
        producer.send(RecordKey::NULL, format!("Hello from Streamfy {}!", i)).await?;
        println!("[PRODUCER] sent record {}", i);
        tokio::sleep(Duration::from_secs(1)).await;
    }

    // Streamfy batches records by default, call flush() when done producing
    // to ensure all records are sent
    producer.flush().await?;

    let mut stream = consumer.stream(Offset::beginning()).await?;

    while let Some(Ok(record)) = stream.next().await {
        let value_str = record.get_value().as_utf8_lossy_string();

        println!("[CONSUMER] Got record: {}", value_str);
        consumed_records += 1;

        if consumed_records >= MAX_RECORDS {
            break;
        }
    }

    Ok(())
}
```

5. Run `cargo run` and expect the following output

```txt
[PRODUCER] sent record 0
[PRODUCER] sent record 1
[PRODUCER] sent record 2
[PRODUCER] sent record 3
[PRODUCER] sent record 4
[PRODUCER] sent record 5
[PRODUCER] sent record 6
[PRODUCER] sent record 7
[PRODUCER] sent record 8
[PRODUCER] sent record 9
[CONSUMER] Got record: Hello, Streamfy 0!
[CONSUMER] Got record: Hello, Streamfy 1!
[CONSUMER] Got record: Hello, Streamfy 2!
[CONSUMER] Got record: Hello, Streamfy 3!
[CONSUMER] Got record: Hello, Streamfy 4!
[CONSUMER] Got record: Hello, Streamfy 5!
[CONSUMER] Got record: Hello, Streamfy 6!
[CONSUMER] Got record: Hello, Streamfy 7!
[CONSUMER] Got record: Hello, Streamfy 8!
[CONSUMER] Got record: Hello, Streamfy 9!
```

6. Clean Up

```bash
streamfy topic delete echo-test
topic "echo-test" deleted
```

## Learn More

- [Read on tutorials][Tutorials] to get the most from Streamfy and Streamfy Cloud
  to scale your streaming solution.

- You can use Streamfy to send or receive records from different sources using [Connectors][Connectors].

- If you want to filter or transform records on the fly read more about [SmartModules][SmartModules].

[Streamfy.io]: https://www.streamfy.io
[Install Streamfy CLI]: https://www.streamfy.io/docs/streamfy/cli/overview
[Connectors]: https://www.streamfy.io/docs/connectors/overview
[SmartModules]: https://www.streamfy.io/docs/smartmodules/overview
[Tutorials]: https://www.streamfy.io/docs/cloud/tutorials/

use std::sync::Arc;
use streamfy::Streamfy;
use futures::StreamExt;

// This is a temporary example for testing the new Admin Watch API
#[tokio::main]
async fn main() {
    let streamfy = Streamfy::connect().await.unwrap();
    let admin = Arc::new(streamfy.admin().await);

    let admin1 = admin.clone();
    tokio::spawn(async move {
        let mut topic_stream = admin1.watch_topics();
        while let Some(thing) = topic_stream.next().await {
            println!("Got Topic update: {thing:#?}");
        }
    });

    let admin2 = admin.clone();
    tokio::spawn(async move {
        let mut partition_stream = admin2.watch_partitions();
        while let Some(thing) = partition_stream.next().await {
            println!("Got Partition update: {thing:#?}");
        }
    });

    let mut spu_stream = admin.watch_spus();
    while let Some(thing) = spu_stream.next().await {
        println!("Got SPU update: {thing:#?}");
    }
}

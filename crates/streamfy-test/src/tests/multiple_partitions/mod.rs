pub mod producer;
pub mod consumer;

use clap::Parser;

use streamfy_future::task::spawn;
use streamfy_test_derive::streamfy_test;
use streamfy_test_case_derive::MyTestCase;
use streamfy_future::task::run_block_on;

#[derive(Debug, Clone, Parser, Default, Eq, PartialEq, MyTestCase)]
#[command(name = "Streamfy MultiplePartition Test")]
pub struct MultiplePartitionTestOption {}

#[streamfy_test(topic = "test-multiple-partition")]
pub fn multiple_partition(mut test_driver: TestDriver, mut test_case: TestCase) -> TestResult {
    println!("Testing multiple partition consumer");

    let option: MyTestCase = test_case.into();

    run_block_on(async {
        spawn(producer::producer(test_driver.clone(), option.clone()));

        consumer::consumer_stream(&test_driver, option).await;
    });
}

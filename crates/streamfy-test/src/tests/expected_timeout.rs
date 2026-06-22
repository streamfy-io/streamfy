use std::any::Any;
use std::time::Duration;

use clap::Parser;
use streamfy_future::timer::sleep;
use streamfy_test_derive::streamfy_test;
use streamfy_test_util::test_meta::{TestOption, TestCase};
use streamfy_test_util::async_process;

#[derive(Debug, Clone)]
pub struct ExpectedTimeoutTestCase {}

impl From<TestCase> for ExpectedTimeoutTestCase {
    fn from(_test_case: TestCase) -> Self {
        ExpectedTimeoutTestCase {}
    }
}

#[derive(Debug, Parser, Clone)]
#[command(name = "Streamfy Expected timeout Test")]
pub struct ExpectedTimeoutTestOption {}
impl TestOption for ExpectedTimeoutTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[streamfy_test(name = "expected_timeout", topic = "unused")]
pub fn run(mut test_driver: StreamfyTestDriver, mut test_case: TestCase) {
    println!("\nStarting example test that timeouts");

    let infinite_loop = async_process!(
        async {
            loop {
                sleep(Duration::from_secs(1)).await
            }
            // Do nothing and exit
        },
        "infinite loop"
    );
    infinite_loop.join().unwrap();
}

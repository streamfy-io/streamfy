use std::any::Any;

use clap::Parser;
use streamfy_test_derive::streamfy_test;
use streamfy_test_util::test_meta::{TestOption, TestCase};
use streamfy_test_util::async_process;

#[derive(Debug, Clone)]
pub struct ExpectedPassTestCase {}

impl From<TestCase> for ExpectedPassTestCase {
    fn from(_test_case: TestCase) -> Self {
        ExpectedPassTestCase {}
    }
}

#[derive(Debug, Parser, Clone)]
#[command(name = "Streamfy Expected Fail Test")]
pub struct ExpectedPassTestOption {}
impl TestOption for ExpectedPassTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[streamfy_test(name = "expected_pass", topic = "unused")]
pub fn run(mut test_driver: StreamfyTestDriver, mut test_case: TestCase) {
    println!("\nStarting example test that passes");

    let fast_success = async_process!(
        async {
            // Do nothing and exit
        },
        "fast-success"
    );
    fast_success.join().unwrap();
}

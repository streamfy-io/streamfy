use std::any::Any;
use std::time::Duration;

use clap::Parser;
use streamfy_future::timer::sleep;
use streamfy_test_derive::streamfy_test;
use streamfy_test_util::test_meta::{TestOption, TestCase};
use streamfy_test_util::async_process;

#[derive(Debug, Clone)]
pub struct ExpectedFailJoinSuccessFirstTestCase {}

impl From<TestCase> for ExpectedFailJoinSuccessFirstTestCase {
    fn from(_test_case: TestCase) -> Self {
        ExpectedFailJoinSuccessFirstTestCase {}
    }
}

#[derive(Debug, Parser, Clone)]
#[command(name = "Streamfy Expected FailJoinSuccessFirst Test")]
pub struct ExpectedFailJoinSuccessFirstTestOption {}
impl TestOption for ExpectedFailJoinSuccessFirstTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[streamfy_test(name = "expected_fail_join_success_first", topic = "unused")]
pub fn run(mut test_driver: StreamfyTestDriver, mut test_case: TestCase) {
    println!("\nStarting example test that fails");

    let success = async_process!(
        async {
            sleep(Duration::from_millis(100)).await;
        },
        "success"
    );

    let fail = async_process!(
        async {
            sleep(Duration::from_millis(200)).await;
            panic!("This test should fail");
        },
        "fail"
    );
    success.join().unwrap();
    fail.join().unwrap();
}

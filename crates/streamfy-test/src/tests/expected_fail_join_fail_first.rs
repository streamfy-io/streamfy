use std::any::Any;
use std::time::Duration;

use clap::Parser;
use streamfy_future::timer::sleep;
use streamfy_test_derive::streamfy_test;
use streamfy_test_util::test_meta::{TestOption, TestCase};
use streamfy_test_util::async_process;

#[derive(Debug, Clone)]
pub struct ExpectedFailJoinFailFirstTestCase {}

impl From<TestCase> for ExpectedFailJoinFailFirstTestCase {
    fn from(_test_case: TestCase) -> Self {
        ExpectedFailJoinFailFirstTestCase {}
    }
}

#[derive(Debug, Parser, Clone)]
#[command(name = "Streamfy Expected FailJoinFailFirst Test")]
pub struct ExpectedFailJoinFailFirstTestOption {}
impl TestOption for ExpectedFailJoinFailFirstTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[streamfy_test(name = r#"expected_fail_join_fail_first"#, topic = "unused")]
pub fn run(mut test_driver: StreamfyTestDriver, mut test_case: TestCase) {
    println!("\nStarting example test that fails");

    let success = async_process!(
        async {
            sleep(Duration::from_millis(2000)).await;
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
    fail.join().unwrap();
    success.join().unwrap();
}

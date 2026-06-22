use streamfy_test_derive::streamfy_test;
#[warn(unused_imports)]
use streamfy_test_util::test_meta::TestCase;
use clap::Parser;
use std::any::Any;
use streamfy_test_util::test_meta::TestOption;

#[derive(Debug, Clone, Parser, Default, PartialEq)]
#[clap(name = "Streamfy Test Example One")]
pub struct TestOneTestOption {}

#[derive(Debug, Clone, Parser, Default, PartialEq)]
#[clap(name = "Streamfy Test Example Two")]
pub struct TestTwoTestOption {}

impl TestOption for TestOneTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TestOption for TestTwoTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[streamfy_test(cluster_type = "k8")]
pub fn test_one(mut test_driver: TestDriver, test_case: TestCase) {}

#[streamfy_test(cluster_type = "local")]
pub fn test_two(mut test_driver: TestDriver, test_case: TestCase) {}

fn main() {}

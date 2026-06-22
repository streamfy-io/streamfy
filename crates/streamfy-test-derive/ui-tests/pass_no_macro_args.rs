use streamfy_test_derive::streamfy_test;
#[warn(unused_imports)]
use streamfy_test_util::test_meta::TestCase;
use clap::Parser;
use std::any::Any;
use streamfy_test_util::test_meta::TestOption;

#[derive(Debug, Clone, Parser, Default, PartialEq)]
#[clap(name = "Streamfy Test Example")]
pub struct RunTestOption {}

impl TestOption for RunTestOption {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[streamfy_test]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {}

fn main() {}

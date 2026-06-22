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

#[streamfy_test(timeout = 60)]
pub fn valid_timeout_sec(mut test_driver: TestDriver, test_case: TestCase) {}

#[streamfy_test(timeout = 0)]
pub fn disable_timeout1(mut test_driver: TestDriver, test_case: TestCase) {}

#[streamfy_test(timeout = false)]
pub fn disable_timeout2(mut test_driver: TestDriver, test_case: TestCase) {}

fn main() {}

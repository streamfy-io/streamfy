use streamfy_test_derive::streamfy_test;
#[allow(unused_imports)]
use streamfy_test_util::test_meta::TestCase;

#[streamfy_test(async = 1)]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {
}

#[streamfy_test(async = a)]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {
}

#[streamfy_test(async = "true")]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {
}

#[streamfy_test(async = "false")]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {
}

fn main() {
}

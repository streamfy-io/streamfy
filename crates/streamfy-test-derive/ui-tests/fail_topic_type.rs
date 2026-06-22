use streamfy_test_derive::streamfy_test;
#[allow(unused_imports)]
use streamfy_test_util::test_meta::TestCase;

#[streamfy_test(topic = 1)]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {
}

fn main() {
}

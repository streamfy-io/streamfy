use streamfy_test_derive::streamfy_test;
#[allow(unused_imports)]
use streamfy_test_util::test_meta::TestCase;

#[streamfy_test(min_spu = a, topic = 2)]
pub fn run(mut test_driver: TestDriver, test_case: TestCase) {
}

fn main() {
}

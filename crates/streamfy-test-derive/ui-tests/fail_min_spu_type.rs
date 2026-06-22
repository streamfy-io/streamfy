use streamfy_test_derive::streamfy_test;
#[allow(unused_imports)]
use streamfy_test_util::test_meta::TestCase;

#[streamfy_test(min_spu = a)]
pub fn test1(mut test_driver: TestDriver, test_case: TestCase) {
}

#[streamfy_test(min_spu = "1")]
pub fn test2(mut test_driver: TestDriver, test_case: TestCase) {
}

fn main() {
}

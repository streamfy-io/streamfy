use std::env;

use clap::Parser;

use streamfy_test_derive::streamfy_test;
use streamfy_test_util::async_process;
use streamfy_test_case_derive::MyTestCase;

#[derive(Debug, Clone, Parser, Default, Eq, PartialEq, MyTestCase)]
#[command(name = "Streamfy Test Self Check")]
pub struct SelfCheckTestOption {
    /// Intentionally panic to test panic handling
    #[arg(long)]
    pub force_panic: bool,
}

#[streamfy_test()]
pub fn self_check(mut test_driver: StreamfyTestDriver, mut test_case: TestCase) {
    let self_test_case: MyTestCase = test_case.into();

    // If the CI env var is exists, we're in CI
    if env::var("CI").is_ok() {
        println!("Running in CI")
    }

    println!("Starting Streamfy Test Self-Check");

    let another_process = async_process!(
        async {
            // Sleep for a moment to help (visually) validate global test timer
            std::thread::sleep(std::time::Duration::from_secs(3));

            if self_test_case.option.force_panic {
                panic!("Intentionally panicking inside another process");
            }
        },
        "sleep"
    );

    another_process.join().unwrap();
}

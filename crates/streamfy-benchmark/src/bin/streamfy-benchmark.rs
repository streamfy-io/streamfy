use anyhow::Result;
use clap::Parser;
use streamfy_benchmark::cli::{BenchmarkOpt, run_benchmarks};
use streamfy_future::task::run_block_on;

fn main() -> Result<()> {
    streamfy_future::subscriber::init_logger();
    let args = BenchmarkOpt::parse();

    run_block_on(run_benchmarks(args))
}

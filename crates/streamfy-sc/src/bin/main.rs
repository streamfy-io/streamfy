#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;

use streamfy_sc::cli::ScOpt;
use streamfy_sc::start::main_loop;

fn main() {
    streamfy_future::subscriber::init_tracer(None);

    let opt = ScOpt::parse();
    main_loop(opt);
}

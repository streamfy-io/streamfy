#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;

fn main() {
    streamfy_future::subscriber::init_tracer(None);

    let opt = streamfy_spu::SpuOpt::parse();
    streamfy_spu::main_loop(opt);
}

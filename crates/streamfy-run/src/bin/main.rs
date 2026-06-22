use clap::Parser;
use streamfy_run::RunCmd;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd: RunCmd = RunCmd::parse();

    streamfy_future::subscriber::init_tracer(None);

    cmd.process()?;
    Ok(())
}

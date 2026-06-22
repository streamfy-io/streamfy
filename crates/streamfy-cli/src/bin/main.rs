#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;
use anyhow::Result;

use streamfy_cli::{Root, HelpOpt};
use streamfy_future::task::run_block_on;

fn main() -> Result<()> {
    streamfy_future::subscriber::init_tracer(None);

    print_help_hack()?;
    let root: Root = Root::parse();

    // If the CLI comes back with an error, attempt to handle it
    if let Err(e) = run_block_on(root.process()) {
        eprintln!("{e}");
        std::process::exit(1);
    }

    Ok(())
}

fn print_help_hack() -> Result<()> {
    let mut args = std::env::args();
    if args.len() < 2 {
        HelpOpt {}.process()?;
        std::process::exit(0);
    } else if let Some(first_arg) = args.nth(1) {
        // We pick help up here as a courtesy
        if ["-h", "--help", "help"].contains(&first_arg.as_str()) {
            HelpOpt {}.process()?;
            std::process::exit(0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use streamfy_cli::Root;

    #[test]
    fn test_correct_command_parsing_help() {
        assert!(parse("streamfy").is_err());
        assert!(parse("streamfy -h").is_err());
        assert!(parse("streamfy --help").is_err());
    }

    #[test]
    fn test_correct_command_parsing_consume() {
        assert!(parse("streamfy consume -B hello").is_ok());
        assert!(parse("streamfy consume -T 10 hello").is_ok());
        assert!(parse("streamfy consume -H 10 hello").is_ok());
        assert!(parse("streamfy consume --end 10 hello").is_ok());
        assert!(parse("streamfy consume --start  0 hello").is_ok());
        assert!(parse("streamfy consume hello --start 0 --end 5").is_ok());

        assert!(parse("streamfy consume").is_err());
        assert!(parse("streamfy consume -H 10 -T 10 hello").is_err());
        assert!(parse("streamfy consume -B -H hello").is_err());
        assert!(parse("streamfy consume --end hello").is_err());
    }

    #[test]
    fn test_supply_negative_end_offset() {
        assert!(parse("streamfy consume --start 0 --end 5  hello").is_ok());
        assert!(parse("streamfy consume --end 5 hello").is_ok());

        assert!(parse("streamfy consume --end -5 hello").is_err());
        assert!(parse("streamfy consume --start --end -5 -n 0 hello").is_err());
    }

    fn parse(command: &str) -> Result<Root, clap::error::Error> {
        Root::try_parse_from(command.split_whitespace())
    }
}

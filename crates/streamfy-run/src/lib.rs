use clap::Parser;

mod error;

pub use error::RunnerError;
use error::Result;
use streamfy_spu::SpuOpt;
use streamfy_sc::cli::ScOpt;
use streamfy_extension_common::StreamfyExtensionMetadata;

const VERSION: &str = include_str!("../../../VERSION");

#[derive(Debug, Parser)]
#[command(version = crate::VERSION)]
pub enum RunCmd {
    /// Run a new Streaming Processing Unit (SPU)
    #[command(name = "spu")]
    SPU(SpuOpt),
    /// Run a new Streaming Controller (SC)
    #[command(name = "sc")]
    SC(ScOpt),
    /// Return plugin metadata as JSON
    #[command(name = "metadata")]
    Metadata(MetadataOpt),

    /// Print version information
    #[command(name = "version")]
    Version(VersionOpt),
}

impl RunCmd {
    pub fn process(self) -> Result<()> {
        match self {
            Self::SPU(opt) => {
                streamfy_spu::main_loop(opt);
            }
            Self::SC(opt) => {
                streamfy_sc::start::main_loop(opt);
            }
            Self::Metadata(meta) => {
                meta.process()?;
            }
            Self::Version(opt) => {
                opt.process()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct MetadataOpt {}
impl MetadataOpt {
    pub fn process(self) -> Result<()> {
        if let Ok(metadata) = serde_json::to_string(&Self::metadata()) {
            println!("{metadata}");
        }
        Ok(())
    }

    pub fn metadata() -> StreamfyExtensionMetadata {
        StreamfyExtensionMetadata {
            title: "Streamfy Runner".into(),
            package: Some("streamfy/streamfy-run".parse().unwrap()),
            description: "Run Streamfy cluster components (SC and SPU)".into(),
            version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        }
    }
}

#[derive(Debug, Parser)]
pub struct VersionOpt {}

impl VersionOpt {
    pub fn process(self) -> Result<()> {
        println!("Git Commit: {}", env!("GIT_HASH"));
        println!("Platform Version: {VERSION}");

        Ok(())
    }
}

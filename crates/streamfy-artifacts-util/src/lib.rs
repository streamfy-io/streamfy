mod package_meta_ext;
mod utils;

pub mod htclient;

pub mod fvm;

pub use http;
pub use package_meta_ext::*;
pub use utils::*;
pub use utils::sha256_digest;

pub use streamfy_hub_protocol::*;
pub use streamfy_hub_protocol::constants::*;

pub const REPO_OWNER: &str = "streamfy-io";
pub const REPO_NAME: &str = "streamfy";

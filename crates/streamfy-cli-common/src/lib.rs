pub mod http;
pub mod install;
pub mod error;

#[cfg(feature = "file-records")]
pub mod user_input;

#[cfg(feature = "version-cmd")]
pub mod version_cmd;

#[cfg(feature = "smartmodule-test")]
pub mod smartmodule;

// Environment vars for Channels
pub const STREAMFY_RELEASE_CHANNEL: &str = "STREAMFY_RELEASE_CHANNEL";
pub const STREAMFY_EXTENSIONS_DIR: &str = "STREAMFY_EXTENSIONS_DIR";
pub const STREAMFY_IMAGE_TAG_STRATEGY: &str = "STREAMFY_IMAGE_TAG_STRATEGY";
pub const STREAMFY_ALWAYS_CHECK_UPDATES: &str = "STREAMFY_ALWAYS_CHECK_UPDATES";

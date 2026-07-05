pub mod setup;
pub mod test_runner;
pub mod tls;

pub mod test_meta;
use std::sync::LazyLock;

static VERSION: LazyLock<String> = LazyLock::new(|| {
    let version = include_str!("../../VERSION");
    match option_env!("STREAMFY_VERSION_SUFFIX") {
        Some(suffix) => format!("{version}-{suffix}"),
        None => version.to_string(),
    }
});

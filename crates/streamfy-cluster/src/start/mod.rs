pub mod k8;
pub mod local;
mod common;

mod constants {

    use std::env;

    use std::sync::LazyLock;

    /// maximum time waiting for SC and SPU to provision
    pub static MAX_PROVISION_TIME_SEC: LazyLock<u16> = LazyLock::new(|| {
        let var_value = env::var("FLV_CLUSTER_PROVISION_TIMEOUT").unwrap_or_default();
        var_value.parse().unwrap_or(300)
    });
}

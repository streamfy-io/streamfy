mod public_server;
mod spg;
mod smartmodule;
mod spu;
mod topic;
mod partition;
mod api_version;
mod create;
mod delete;
mod update;
mod list;
mod watch;
mod tableformat;
mod derivedstream;
mod mirror;
mod mirroring;

pub use server::start_public_server;

mod server {

    use std::fmt::Debug;

    use streamfy_stream_model::core::MetadataItem;
    use tracing::debug;

    use streamfy_service::StreamfyApiServer;
    use streamfy_auth::Authorization;

    use crate::services::auth::AuthGlobalContext;
    use super::public_server::PublicService;

    /// create public server
    pub fn start_public_server<A, C>(ctx: AuthGlobalContext<A, C>)
    where
        A: Authorization + Sync + Send + Debug + 'static,
        C: MetadataItem + 'static,
        C::UId: Send + Sync,
        AuthGlobalContext<A, C>: Clone + Debug,
        <A as Authorization>::Context: Send + Sync,
    {
        let addr = ctx.global_ctx.config().public_endpoint.clone();
        debug!("starting public api service");
        let server = StreamfyApiServer::new(addr, ctx, PublicService::new());
        server.run();
    }
}

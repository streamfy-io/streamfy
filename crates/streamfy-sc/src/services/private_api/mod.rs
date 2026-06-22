mod private_server;

use streamfy_stream_model::core::MetadataItem;
use tracing::info;
use tracing::instrument;

use private_server::ScInternalService;
use streamfy_service::StreamfyApiServer;

use crate::core::SharedContext;

// start server
#[instrument(
    name = "sc_private_server"
    skip(ctx),
    fields(address = &*ctx.config().private_endpoint)
)]
pub fn start_internal_server<C>(ctx: SharedContext<C>)
where
    C: MetadataItem + 'static,
{
    info!("starting internal services");

    let addr = ctx.config().private_endpoint.clone();
    let server = StreamfyApiServer::new(addr, ctx, ScInternalService::new());
    server.run();
}

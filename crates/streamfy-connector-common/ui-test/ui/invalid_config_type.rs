use streamfy_connector_common::connector;

#[connector(source)]
async fn start_fn(config: &[i32], producer: ()) {}

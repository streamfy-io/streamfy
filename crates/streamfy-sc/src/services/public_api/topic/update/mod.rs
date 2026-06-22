mod add_partition;
mod add_mirror;

use std::io::{Error, ErrorKind};

use anyhow::Result;
use tracing::{info, instrument, trace};

use streamfy_controlplane_metadata::extended::SpecExt;
use streamfy_protocol::link::ErrorCode;
use streamfy_auth::{AuthContext, InstanceAction};
use streamfy_stream_model::core::MetadataItem;
use streamfy_sc_schema::{
    topic::{TopicSpec, UpdateTopicAction},
    Status,
};

use crate::services::auth::AuthServiceContext;

#[instrument(skip(topic_name, action, auth_ctx))]
pub async fn handle_topic_update_request<AC: AuthContext, C: MetadataItem>(
    topic_name: String,
    action: UpdateTopicAction,
    auth_ctx: &AuthServiceContext<AC, C>,
) -> Result<Status, Error> {
    info!(%topic_name, "Updating topic");

    if let Ok(authorized) = auth_ctx
        .auth
        .allow_instance_action(TopicSpec::OBJECT_TYPE, InstanceAction::Update, &topic_name)
        .await
    {
        if !authorized {
            trace!("authorization failed");
            return Ok(Status::new(
                topic_name.clone(),
                ErrorCode::PermissionDenied,
                Some(String::from("permission denied")),
            ));
        }
    } else {
        return Err(Error::new(ErrorKind::Interrupted, "authorization io error"));
    }

    let status = match action {
        UpdateTopicAction::AddPartition(req) => {
            add_partition::handle_add_partition(topic_name, req, auth_ctx).await?
        }
        UpdateTopicAction::AddMirror(req) => {
            add_mirror::handle_add_mirror(topic_name, req, auth_ctx).await?
        }
    };

    Ok(status)
}

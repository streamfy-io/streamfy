//!
//! # Clear Topic Request
//!
//! Remove all stored records from a topic without deleting topic metadata,
//! partitions, replicas, or consumer offsets.
//!
use std::io::Error;

use tracing::{info, instrument};

use streamfy_auth::AuthContext;
use streamfy_protocol::link::ErrorCode;
use streamfy_sc_schema::Status;
use streamfy_stream_model::core::MetadataItem;

use crate::services::auth::AuthServiceContext;

/// Handler for clear topic request
#[instrument(skip(auth_ctx))]
pub async fn handle_clear_topic<AC: AuthContext, C: MetadataItem>(
    topic_name: String,
    auth_ctx: &AuthServiceContext<AC, C>,
) -> Result<Status, Error> {
    let topic = auth_ctx
        .global_ctx
        .topics()
        .store()
        .value(&topic_name)
        .await;

    let Some(_topic) = topic else {
        return Ok(Status::new(
            topic_name,
            ErrorCode::TopicNotFound,
            Some("not found".to_owned()),
        ));
    };

    let partitions = auth_ctx
        .global_ctx
        .partitions()
        .store()
        .clone_values()
        .await;
    let mut cleared = 0u32;

    for partition in partitions {
        let key = partition.key_owned();
        if key.topic != topic_name {
            continue;
        }

        let mut spec = partition.spec().clone();
        // Saturating add avoids overflow on repeated clears in long-lived clusters.
        spec.clear_epoch = spec.clear_epoch.saturating_add(1);

        info!(%key, clear_epoch = spec.clear_epoch, "clearing partition storage");

        if let Err(err) = auth_ctx
            .global_ctx
            .partitions()
            .create_spec(key.clone(), spec)
            .await
        {
            return Ok(Status::new(
                topic_name,
                ErrorCode::TopicError,
                Some(format!("failed to clear partition {key}: {err}")),
            ));
        }
        cleared += 1;
    }

    // Topic with no partitions yet: still succeed (nothing to clear).
    info!(%topic_name, cleared, "topic clear requested");

    Ok(Status::new_ok(topic_name))
}

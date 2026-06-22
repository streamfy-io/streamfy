use streamfy_storage::config::ConfigOption;
use streamfy_storage::FileReplica;
use streamfy_storage::StorageError;
use streamfy_controlplane_metadata::partition::ReplicaKey;
use streamfy_types::SpuId;

use crate::config::Log;


/* 
pub async fn clear_replica_storage(local_spu: SpuId, replica: &ReplicaKey, config: &Log) {
    let storage_config = config.as_storage_config();
    let config = default_config(local_spu, &storage_config);
    FileReplica::clear(replica, &config).await
}
*/

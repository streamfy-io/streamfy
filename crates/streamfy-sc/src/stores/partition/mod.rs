use streamfy_controlplane::PartitionMetadata;
pub use streamfy_controlplane_metadata::partition::*;
pub use streamfy_controlplane_metadata::store::k8::K8MetaItem;

pub type PartitionAdminMd = PartitionMetadata<K8MetaItem>;
pub type PartitionAdminStore = PartitionLocalStore<K8MetaItem>;

mod policy;
mod store;

pub use store::*;
pub use policy::*;

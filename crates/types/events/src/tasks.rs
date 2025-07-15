use kabu_types_entities::{PoolClass, PoolId};

#[derive(Clone, Debug)]
pub enum LoomTask {
    FetchAndAddPools(Vec<(PoolId, PoolClass)>),
}

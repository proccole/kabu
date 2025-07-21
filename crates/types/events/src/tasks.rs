use kabu_types_market::{PoolClass, PoolId};

#[derive(Clone, Debug)]
pub enum LoomTask {
    FetchAndAddPools(Vec<(PoolId, PoolClass)>),
}

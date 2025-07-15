use crate::PoolId;
use alloy_primitives::Address;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Debug)]
pub struct SwapDirection {
    token_from: Address,
    token_to: Address,
}

impl SwapDirection {
    #[inline]
    pub fn new(token_from: Address, token_to: Address) -> Self {
        Self { token_from, token_to }
    }

    #[inline]
    pub fn from(&self) -> &Address {
        &self.token_from
    }
    #[inline]
    pub fn to(&self) -> &Address {
        &self.token_to
    }

    #[inline]
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[inline]
    pub fn get_hash_with_pool(&self, pool_id: &PoolId) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        pool_id.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for SwapDirection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.token_from.hash(state);
        self.token_to.hash(state);
    }
}

impl PartialEq for SwapDirection {
    fn eq(&self, other: &Self) -> bool {
        self.token_from.eq(&other.token_from) && self.token_to.eq(&other.token_to)
    }
}

impl Eq for SwapDirection {}

impl From<(Address, Address)> for SwapDirection {
    fn from(value: (Address, Address)) -> Self {
        Self { token_from: value.0, token_to: value.1 }
    }
}

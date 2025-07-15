use crate::Message;
use alloy_primitives::TxHash;
use kabu_types_entities::{EstimationError, SwapError};

#[derive(Clone, Debug)]
pub enum HealthEvent {
    PoolSwapError(SwapError),
    SwapLineEstimationError(EstimationError),
    MonitorTx(TxHash),
}

pub type MessageHealthEvent = Message<HealthEvent>;

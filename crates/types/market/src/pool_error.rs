use alloy_primitives::Address;
use alloy_sol_types::Error as AlloyError;
use kabu_evm_db::KabuDBError;

#[derive(Clone, Debug, thiserror::Error)]
pub enum PoolError {
    // Pool-specific errors
    #[error("UniswapV2: {0}")]
    UniswapV2(#[from] UniswapV2Error),

    #[error("UniswapV3: {0}")]
    UniswapV3(#[from] UniswapV3Error),

    #[error("Curve: {0}")]
    Curve(#[from] CurveError),

    #[error("Maverick: {0}")]
    Maverick(#[from] MaverickError),

    // Common errors
    #[error("functionality not implemented")]
    NotImplemented,

    #[error("invalid input: {reason}")]
    InvalidInput { reason: &'static str },

    // Database/RPC errors
    #[error(transparent)]
    DatabaseError(#[from] KabuDBError),

    #[error("error: {0}")]
    GeneralError(String),

    // ABI errors
    #[error("ABI decoding failed for {method}: {source}")]
    AbiDecodingError { method: &'static str, source: AlloyError },

    // EVM revert errors
    #[error("EVM reverted: {reason} (gas: {gas_used})")]
    EvmReverted { reason: String, gas_used: u64 },
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum UniswapV2Error {
    #[error("amount in with fee calculation overflowed")]
    AmountInWithFeeOverflow,

    #[error("numerator calculation overflowed")]
    NumeratorOverflow,

    #[error("numerator fee calculation overflowed")]
    NumeratorOverflowFee,

    #[error("denominator calculation overflowed")]
    DenominatorOverflow,

    #[error("denominator fee calculation overflowed")]
    DenominatorOverflowFee,

    #[error("denominator calculation underflowed")]
    DenominatorUnderflow,

    #[error("input amount calculation overflowed")]
    InAmountOverflow,

    #[error("cannot calculate with zero reserves")]
    CannotCalculateZeroReserve,

    #[error("calculated output amount is zero")]
    OutAmountIsZero,

    #[error("calculated input amount is zero")]
    InAmountIsZero,

    #[error("output amount exceeds pool reserves")]
    ReserveExceeded,

    #[error("requested output exceeds available reserves")]
    ReserveOutExceeded,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum UniswapV3Error {
    #[error("invalid tick value")]
    InvalidTick,

    #[error("no liquidity in tick range")]
    NoLiquidityInRange,

    #[error("price out of valid bounds")]
    PriceOutOfBounds,

    #[error("invalid sqrt price")]
    InvalidSqrtPrice,

    #[error("bad price step in calculation")]
    BadPriceStep,

    #[error("calculated output amount is zero")]
    OutAmountIsZero,

    #[error("calculated input amount is zero")]
    InAmountIsZero,

    #[error("overflow in calculation")]
    OverflowInCalculation,

    #[error("underflow in calculation")]
    UnderflowInCalculation,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum CurveError {
    #[error("token {token} not in pool")]
    TokenNotInPool { token: Address },

    #[error("calculated output amount is zero")]
    OutAmountIsZero,

    #[error("calculated input amount is zero")]
    InAmountIsZero,

    #[error("pool in invalid state")]
    InvalidPoolState,

    #[error("cannot get balance from pool")]
    CannotGetBalance,

    #[error("underlying coin not found")]
    UnderlyingCoinNotFound,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum MaverickError {
    #[error("input amount exceeds maximum allowed")]
    MaxInAmount,

    #[error("calculated output amount is zero")]
    OutAmountIsZero,

    #[error("requested output exceeds available reserves")]
    ReserveOutExceeded,

    #[error("calculated input amount is zero")]
    InAmountIsZero,
}

impl From<eyre::Report> for PoolError {
    fn from(error: eyre::Report) -> Self {
        PoolError::GeneralError(error.to_string())
    }
}

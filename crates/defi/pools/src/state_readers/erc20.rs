use alloy::primitives::{Address, U256};
use alloy::sol_types::{SolCall, SolInterface};
use alloy_evm::EvmEnv;
use eyre::Result;
use kabu_defi_abi::IERC20;
use kabu_evm_db::KabuDBError;
use kabu_evm_utils::evm_call;
use kabu_types_entities::PoolError;
use revm::DatabaseRef;

pub struct ERC20StateReader {}

#[allow(dead_code)]
impl ERC20StateReader {
    pub fn balance_of<DB: DatabaseRef<Error = KabuDBError> + ?Sized>(
        db: &DB,
        evm_env: &EvmEnv,
        erc20_token: Address,
        account: Address,
    ) -> Result<U256, PoolError> {
        let input = IERC20::IERC20Calls::balanceOf(IERC20::balanceOfCall { account }).abi_encode();
        let (call_data_result, _, _) = evm_call(db, evm_env.clone(), erc20_token, input)?;

        let call_return = IERC20::balanceOfCall::abi_decode_returns(&call_data_result)
            .map_err(|e| PoolError::AbiDecodingError { method: "balanceOf", source: e })?;
        Ok(call_return)
    }

    pub fn allowance<DB: DatabaseRef<Error = KabuDBError> + ?Sized>(
        db: &DB,
        evm_env: &EvmEnv,
        erc20_token: Address,
        owner: Address,
        spender: Address,
    ) -> Result<U256, PoolError> {
        let input = IERC20::IERC20Calls::allowance(IERC20::allowanceCall { owner, spender }).abi_encode();
        let (call_data_result, _, _) = evm_call(db, evm_env.clone(), erc20_token, input)?;

        let call_return = IERC20::allowanceCall::abi_decode_returns(&call_data_result)
            .map_err(|e| PoolError::AbiDecodingError { method: "allowance", source: e })?;
        Ok(call_return)
    }
}

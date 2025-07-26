use alloy_primitives::{Address, Bytes, TxHash};
use alloy_rpc_types::TransactionTrait;
use alloy_rpc_types_eth::Header;
use std::fmt::Debug;

pub trait KabuTx<LDT: KabuDataTypes> {
    fn get_gas_price(&self) -> u128;
    fn get_gas_limit(&self) -> u64;

    fn get_tx_hash(&self) -> TxHash;

    fn get_nonce(&self) -> u64;
    fn get_from(&self) -> Address;

    fn encode(&self) -> Vec<u8>;

    fn to_transaction_request(&self) -> LDT::TransactionRequest;
}

pub trait KabuBlock<LDT: KabuDataTypes> {
    fn get_transactions(&self) -> Vec<LDT::Transaction>;

    fn get_header(&self) -> Header;
}

pub trait KabuTransactionRequest<LDT: KabuDataTypes> {
    fn get_to(&self) -> Option<Address>;
    fn build_call(to: Address, data: Bytes) -> LDT::TransactionRequest;
}

pub trait KabuDataTypes: Debug + Clone + Send + Sync {
    type Transaction: Debug + Clone + Send + Sync + KabuTx<Self> + TransactionTrait;
    type TransactionRequest: Debug + Clone + Send + Sync + KabuTransactionRequest<Self>;
    type TransactionReceipt: Debug + Clone + Send + Sync;
    type Block: Default + Debug + Clone + Send + Sync + KabuBlock<Self>;
}

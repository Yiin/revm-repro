mod actions;
mod config;
// mod logger;
pub mod abis;
mod models;
mod utils;
mod worker;

use config::get_config;
use ethers::types::transaction::eip2718::TypedTransaction;
use inquire::error::InquireResult;
use models::chain::{get_chain_id, simulate_send};
use worker::Worker;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> InquireResult<()> {
    let config = get_config();

    let worker = Worker::new().await;

    let (gas_limit, max_fee_per_gas, max_priority_fee_per_gas) = (
        config.gas.gas_limit,
        config.gas.max_fee_per_gas,
        config.gas.max_priority_fee_per_gas,
    );

    let transaction: TypedTransaction = worker
        .get_buy_transaction()
        .from(worker.wallet.address)
        .gas(gas_limit)
        .max_fee_per_gas(max_fee_per_gas)
        .max_priority_fee_per_gas(max_priority_fee_per_gas)
        .chain_id(get_chain_id().await)
        .into();

    simulate_send(transaction).await;

    Ok(())
}

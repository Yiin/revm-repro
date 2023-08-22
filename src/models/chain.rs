use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use lazy_static::lazy_static;
use revm::db::{CacheDB, EmptyDB, EthersDB};
use revm::inspectors::CustomPrintTracer;
use revm::primitives::{TransactTo, B160, U256};
use revm::{Database, EVM};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::{get_config, get_private_keys};
use crate::utils::key_to_address::private_key_to_address;

use super::router::Router;

static CURRENT_PROVIDER_INDEX: AtomicUsize = AtomicUsize::new(0);

pub async fn create_chain() -> Arc<Provider<Ws>> {
    let config = get_config();
    let current_provider_index = CURRENT_PROVIDER_INDEX.load(Ordering::SeqCst);

    CURRENT_PROVIDER_INDEX.store(
        (current_provider_index + 1) % config.network.providers.len(),
        Ordering::SeqCst,
    );

    let ws = Provider::<Ws>::connect(&config.network.providers[current_provider_index])
        .await
        .unwrap();

    let client = Arc::new(ws);

    client
}

lazy_static! {
    pub static ref EVM_CLIENT: Mutex<Option<Arc<Mutex<EVM<CacheDB<EmptyDB>>>>>> = Mutex::new(None);
}

pub async fn get_evm_client() -> Arc<Mutex<EVM<CacheDB<EmptyDB>>>> {
    let mut evm_client = EVM_CLIENT.lock().await;

    match &*evm_client {
        Some(evm) => evm.clone(),
        None => {
            let config = get_config();
            let client = get_chain().await;
            let mut ethersdb = EthersDB::new(Arc::clone(&client), None).unwrap();
            let mut cache_db = CacheDB::new(EmptyDB::default());

            let mut accounts: Vec<H160> = vec![];

            // Add sniper accounts
            for private_key in get_private_keys() {
                let private_key = private_key.clone();
                let address = private_key_to_address(&private_key);

                accounts.push(address);
            }

            // Add dev account
            accounts.push(config.buy.dev_wallet_address);

            // Insert accounts into cache
            for account_address in accounts {
                let account_address = B160::from(account_address);
                let acc_info = ethersdb.basic(account_address).unwrap().unwrap();

                cache_db.insert_account_info(account_address, acc_info);

                for slot in 0..=10 {
                    let storage_slot = U256::from(slot);
                    let storage = ethersdb.storage(account_address, storage_slot).unwrap();
                    cache_db
                        .insert_account_storage(account_address, storage_slot, storage)
                        .unwrap();
                }
            }

            let mut contracts = vec![];

            // Add contracts
            contracts.push(Router::new().await.factory_address);
            contracts.push(config.dex_router_address);
            contracts.push(config.bot_contract_address);
            contracts.push(config.buy.purchase_token_address);
            contracts.push(config.buy.liquidity_token_address);

            if config.network.chain_token_address != config.buy.liquidity_token_address {
                contracts.push(config.network.chain_token_address);
            }

            // Insert contracts into cache
            for contract_address in contracts {
                let contract_address = B160::from(contract_address);
                let mut contract_info = ethersdb.basic(contract_address).unwrap().unwrap();
                cache_db.insert_contract(&mut contract_info);
            }

            let mut evm: EVM<CacheDB<EmptyDB>> = EVM::new();
            evm.database(cache_db);

            let evm = Arc::new(Mutex::new(evm));

            *evm_client = Some(evm.clone());

            evm
        }
    }
}

pub async fn simulate_send(tx: TypedTransaction) -> bool {
    let tx = tx.as_eip1559_ref().unwrap();
    let from = tx.from.unwrap();
    let to = tx.to.clone().unwrap();
    let value = tx.value.unwrap_or_default();
    let bytes = Bytes::default();
    let data = tx.data.clone().unwrap_or(bytes);
    let gas_limit = tx.gas.unwrap_or_default();
    let gas_price = tx.max_fee_per_gas.unwrap_or_default();
    let gas_priority_fee = tx.max_priority_fee_per_gas.unwrap_or_default();

    let evm = get_evm_client().await;
    let mut evm_guard = evm.lock().await;

    evm_guard.env.tx.caller = B160::from(from.0);
    evm_guard.env.tx.transact_to = TransactTo::Call(B160::from(&to.as_address().unwrap().0));
    evm_guard.env.tx.data = revm::precompile::Bytes::from(data.to_vec());
    evm_guard.env.tx.value = revm::primitives::U256::from(value.as_u64());
    evm_guard.env.tx.gas_limit = gas_limit.as_u64();
    evm_guard.env.tx.gas_price = revm::primitives::U256::from(gas_price.as_u64());
    evm_guard.env.tx.gas_priority_fee =
        Some(revm::primitives::U256::from(gas_priority_fee.as_u64()));
    evm_guard.env.tx.nonce = Some(17);
    evm_guard.env.tx.chain_id = Some(tx.chain_id.unwrap_or(get_chain_id().await.into()).as_u64());

    println!("Simulating tx: {:?}", evm_guard.env.tx);

    let execution_result = evm_guard
        .inspect_commit(CustomPrintTracer::default())
        .unwrap();
    println!("Execution result: {:?}", execution_result);

    return true;

    // let ref_tx = evm_guard.transact_commit().unwrap();
    // let result = ref_tx.clone();

    // match result {
    //     ExecutionResult::Success { logs, .. } => {
    //         // Handle success
    //         println!("Sim ref_tx: {:?}", ref_tx);
    //         println!("Sim success: {:?}", logs);
    //         true
    //     }
    //     ExecutionResult::Revert { output, .. } => {
    //         // Handle revert
    //         println!("Sim ref_tx: {:?}", ref_tx);
    //         println!("Sim revert: {:?}", output);
    //         false
    //     }
    //     ExecutionResult::Halt { reason, .. } => {
    //         // Handle failure
    //         println!("Sim ref_tx: {:?}", ref_tx);
    //         println!("Sim halt: {:?}", reason);
    //         false
    //     }
    // }
}

thread_local! {
    pub static CHAIN: RefCell<Option<Arc<Provider<Ws>>>> = RefCell::new(None);
    pub static CHAIN_ID: RefCell<Option<u64>> = RefCell::new(None);
}

lazy_static! {
    pub static ref ONE_CHAIN: Mutex<Option<Arc<Provider<Ws>>>> = Mutex::new(None);
    pub static ref ONE_CHAIN_ID: Mutex<Option<u64>> = Mutex::new(None);
}

pub async fn get_chain() -> Arc<Provider<Ws>> {
    let config = get_config();

    if config.use_single_chain_for_all_workers {
        let mut one_chain = ONE_CHAIN.lock().await;

        match &*one_chain {
            Some(one_chain) => one_chain.clone(),
            None => {
                let new_one_chain = create_chain().await;
                *one_chain = Some(new_one_chain.clone());
                new_one_chain.clone()
            }
        }
    } else {
        let chain = CHAIN.with(|chain_slot| chain_slot.borrow().clone());

        match chain {
            Some(chain) => chain.clone(),
            None => {
                let new_chain = create_chain().await;
                CHAIN.with(|chain_slot| *chain_slot.borrow_mut() = Some(new_chain.clone()));
                new_chain.clone()
            }
        }
    }
}

pub async fn get_chain_id() -> u64 {
    let chain_id = ONE_CHAIN_ID.lock().await.clone();

    match chain_id {
        Some(one_chain_id) => one_chain_id.clone(),
        None => {
            let new_one_chain_id = get_chain().await.get_chainid().await.unwrap().as_u64();

            *ONE_CHAIN_ID.lock().await = Some(new_one_chain_id);
            new_one_chain_id
        }
    }
}

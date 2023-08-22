use super::chain::get_chain;
use crate::{
    abis::{SWAP_ROUTER, UNISWAP_V2_ROUTER02},
    config::get_config,
};
use ethers::prelude::*;

#[derive(Clone)]
pub struct Router {
    pub contract: UNISWAP_V2_ROUTER02<Provider<Ws>>,
    pub swap_router: SWAP_ROUTER<Provider<Ws>>,
    pub address: Address,
    pub factory_address: Address,
}

impl Router {
    pub async fn new() -> Self {
        let config = get_config();

        let contract = UNISWAP_V2_ROUTER02::new(config.dex_router_address, get_chain().await);
        let swap_router = SWAP_ROUTER::new(config.dex_router_address, get_chain().await);

        let factory_address = match contract.factory().await {
            Ok(address) => address,
            Err(e) => panic!("Failed to get factory address: {}", e),
        };

        Router {
            contract,
            swap_router,
            address: config.dex_router_address,
            factory_address,
        }
    }
}

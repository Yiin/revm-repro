use crate::abis::IERC20;

use super::chain::get_chain;
use ethers::prelude::*;

pub struct IERC20TokenOptions {
    pub contract_address: H160,
}

pub struct IERC20Token {
    pub contract: IERC20<Provider<Ws>>,
    pub decimals: u8,
    pub address: Address,
}

impl IERC20Token {
    pub async fn new(options: IERC20TokenOptions) -> Self {
        let contract = IERC20::new(options.contract_address, get_chain().await);
        let decimals = contract.decimals().call().await.unwrap();
        let address = options.contract_address;

        IERC20Token {
            contract,
            decimals,
            address,
        }
    }
}

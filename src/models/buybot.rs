use ethers::prelude::*;

use crate::{abis::BUY_BOT, config::get_config};

use super::chain::get_chain;

pub struct BuyBot {
    pub contract: BUY_BOT<Provider<Ws>>,
    pub address: Address,
}

impl BuyBot {
    pub async fn new() -> Self {
        let config = get_config();
        let contract = BUY_BOT::new(config.bot_contract_address, get_chain().await);
        let address = config.bot_contract_address;

        BuyBot { contract, address }
    }
}

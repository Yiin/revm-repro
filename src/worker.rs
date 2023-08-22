use crate::models::chain::get_chain_id;
use crate::{
    config::get_config,
    models::{
        buybot::BuyBot,
        ierc20_token::{IERC20Token, IERC20TokenOptions},
        private_keys::{PrivateKeys, PrivateKeysOptions},
        router::Router,
        snipers::{Snipers, SnipersOptions},
        wallet::BlazingWallet,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Worker {
    pub private_keys: Arc<PrivateKeys>,
    pub wallet: Arc<BlazingWallet>,
    pub snipers: Arc<Snipers>,
    pub buy_bot: Arc<BuyBot>,
    pub router: Arc<Router>,
    pub chain_token: Arc<IERC20Token>,
    pub liquidity_token: Arc<IERC20Token>,
    pub purchase_token: Arc<IERC20Token>,
}

impl Worker {
    pub async fn new() -> Arc<Self> {
        let config = get_config();

        let private_keys = Arc::new(PrivateKeys::new(PrivateKeysOptions { rotate: 0 }));

        let router = Arc::new(Router::new().await);

        get_chain_id().await; // cache chain id

        Arc::new(Self {
            private_keys: private_keys.clone(),
            wallet: Arc::new(BlazingWallet::new(private_keys.sender_key.clone()).await),

            snipers: Arc::new(
                Snipers::new(SnipersOptions {
                    recipient_keys: private_keys.recipient_keys.clone(),
                    approve_sniper_keys: private_keys.approve_sniper_keys.clone(),
                })
                .await,
            ),
            buy_bot: Arc::new(BuyBot::new().await),
            router: router.clone(),
            chain_token: Arc::new(
                IERC20Token::new(IERC20TokenOptions {
                    contract_address: config.network.chain_token_address.clone(),
                })
                .await,
            ),
            liquidity_token: Arc::new(
                IERC20Token::new(IERC20TokenOptions {
                    contract_address: config.buy.liquidity_token_address.clone(),
                })
                .await,
            ),
            purchase_token: Arc::new(
                IERC20Token::new(IERC20TokenOptions {
                    contract_address: config.buy.purchase_token_address.clone(),
                })
                .await,
            ),
        })
    }
}

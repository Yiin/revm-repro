use std::sync::Arc;

use crate::utils::key_to_address::private_key_to_address;
use ethers::prelude::{k256::SecretKey, *};

use super::chain::{get_chain, get_chain_id};

#[derive(Clone)]
pub struct BlazingWallet {
    pub address: Address,
    pub private_key: SecretKey,
    pub local: LocalWallet,
    pub signer: SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>,
}

impl BlazingWallet {
    pub async fn new(private_key: SecretKey) -> Self {
        let private_key = private_key.clone();
        let address = private_key_to_address(&private_key);
        let local: LocalWallet = LocalWallet::from(private_key.clone())
            .with_chain_id(get_chain_id().await)
            .into();
        let signer = SignerMiddleware::new(get_chain().await, local.clone());

        Self {
            address,
            private_key,
            local,
            signer,
        }
    }
}

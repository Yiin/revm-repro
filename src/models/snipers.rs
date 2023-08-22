use ethers::prelude::k256::SecretKey;
use futures::{stream, StreamExt};

use super::wallet::BlazingWallet;

pub struct SnipersOptions {
    pub recipient_keys: Vec<SecretKey>,
    pub approve_sniper_keys: Vec<SecretKey>,
}

pub struct Snipers {
    pub recipients: Vec<BlazingWallet>,
    pub approve_snipers: Vec<BlazingWallet>,
}

impl Snipers {
    pub async fn new(options: SnipersOptions) -> Self {
        let recipients = stream::iter(options.recipient_keys)
            .then(|key| BlazingWallet::new(key.clone()))
            .collect::<Vec<_>>()
            .await;

        let approve_snipers = stream::iter(options.approve_sniper_keys)
            .then(|key| BlazingWallet::new(key.clone()))
            .collect::<Vec<_>>()
            .await;

        Self {
            recipients,
            approve_snipers,
        }
    }
}

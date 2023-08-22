use ethers::prelude::k256::SecretKey;

use crate::config::{get_config, get_private_keys};

pub struct PrivateKeysOptions {
    pub rotate: usize,
}

#[derive(Clone)]
pub struct PrivateKeys {
    pub sender_key: SecretKey,
    pub recipient_keys: Vec<SecretKey>,
    pub approve_sniper_keys: Vec<SecretKey>,
}

impl PrivateKeys {
    pub fn new(options: PrivateKeysOptions) -> Self {
        let config = get_config();
        let private_keys = get_private_keys();

        let rotated_keys = {
            let mut keys = private_keys.clone();
            keys.rotate_left(options.rotate);
            keys
        };

        let sender_key = rotated_keys[0].clone();
        let recipient_keys = rotated_keys
            .iter()
            .skip(if config.buy.include_caller { 0 } else { 1 })
            .take(config.buy.snipers)
            .cloned()
            .collect();

        let approve_sniper_keys = rotated_keys
            .iter()
            .take(config.get_approve_snipers())
            .cloned()
            .collect();

        PrivateKeys {
            sender_key,
            recipient_keys,
            approve_sniper_keys,
        }
    }
}

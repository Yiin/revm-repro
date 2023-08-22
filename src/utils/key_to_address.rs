use ethers::prelude::{k256::SecretKey, *};

pub fn private_key_to_address(secret_key: &SecretKey) -> H160 {
    let wallet = Wallet::from(secret_key.clone());

    wallet.address()
}

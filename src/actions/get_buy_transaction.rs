use crate::{config::get_config, utils::decimals::Decimals, worker::Worker};
use ethers::prelude::*;

impl Worker {
    pub fn get_buy_transaction(&self) -> Eip1559TransactionRequest {
        let config = get_config();

        let txid = U256::from(0);

        let buy_amounts = vec![
            // todo decimals should be read from the token
            U256::from(
                config
                    .buy
                    .token_amount
                    .to_decimals(self.purchase_token.decimals),
            ),
            U256::from(
                config
                    .buy
                    .chain_token_spend_limit
                    .to_decimals(self.chain_token.decimals),
            ),
        ];

        Eip1559TransactionRequest::new()
            .data(
                self.buy_bot
                    .contract
                    .buy_de_gainzz(
                        // RouterAddress
                        self.router.address,
                        // PurchaseTokenAddress
                        self.purchase_token.address,
                        // LiquidityTokenAddress
                        self.liquidity_token.address,
                        // BuyMethod
                        config.buy.method,
                        // BuyAmounts
                        buy_amounts,
                        // Snipers
                        self.snipers
                            .recipients
                            .iter()
                            .map(|sniper| Address::from(sniper.address))
                            .collect::<Vec<_>>(),
                        // UseBuyBotChecks
                        config.buy.use_buybot_checks,
                        // CheckSellability
                        config.buy.check_sellability,
                        // WTokenAmountForBuybotTaxChecks
                        U256::from(config.w_token_amount_for_buybot_tax_checks.to_decimals(18)),
                        // Taxes
                        vec![U256::from(0), U256::MAX],
                        // TXID
                        txid,
                    )
                    .calldata()
                    .unwrap(),
            )
            .to(self.buy_bot.address)
    }
}

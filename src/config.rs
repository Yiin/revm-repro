use crossterm::style::Stylize;
use ethers::prelude::{k256::SecretKey, *};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{
    de::{DeserializeOwned, Error},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use std::process;
use std::str::FromStr;
use std::{cmp::max, fmt::Debug};

use crate::utils::decimals::Decimals;

fn deserialize_h160<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    H160::from_str(s.trim_start_matches("0x")).map_err(D::Error::custom)
}

fn deserialize_vec_secret_key<'de, D>(deserializer: D) -> Result<Vec<SecretKey>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec_string: Vec<String> = Vec::deserialize(deserializer)?;
    let mut vec_secret_key: Vec<SecretKey> = vec![];
    for s in vec_string {
        let s_no_prefix = s.trim_start_matches("0x");
        let bytes = hex::decode(s_no_prefix).map_err(D::Error::custom)?;
        vec_secret_key.push(SecretKey::from_slice(&bytes).map_err(D::Error::custom)?);
    }
    Ok(vec_secret_key)
}

fn deserialize_gas<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    Ok(value.to_decimals(9u8))
}

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let value = U256::from(u128::deserialize(deserializer)?);
    Ok(value)
}

fn deserialize_vec_signatures<'de, D>(deserializer: D) -> Result<Vec<[u8; 4]>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec_string: Vec<String> = Vec::deserialize(deserializer)?;
    let mut vec_hex_bytes: Vec<[u8; 4]> = vec![];
    for s in vec_string {
        let hex = hex::decode(s.trim_start_matches("0x"))
            .map_err(|_| D::Error::custom("Failed to decode hex"))?;
        let bytes: [u8; 4] = hex
            .try_into()
            .map_err(|_| D::Error::custom("Failed to convert to [u8; 4]"))?;
        vec_hex_bytes.push(bytes);
    }
    Ok(vec_hex_bytes)
}

fn deserialize_headermap<'de, D>(
    deserializer: D,
) -> Result<Option<HeaderMap<HeaderValue>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: std::collections::HashMap<String, String> =
        std::collections::HashMap::deserialize(deserializer)?;
    let mut header_map = HeaderMap::new();
    for (key, value) in map {
        let header_name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|_| D::Error::custom("invalid header name"))?;
        let header_value =
            HeaderValue::from_str(&value).map_err(|_| D::Error::custom("invalid header value"))?;
        header_map.insert(header_name, header_value);
    }
    Ok(Some(header_map))
}

fn serialize_headermap<S>(
    header_map: &Option<HeaderMap<HeaderValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let header_map = header_map.clone().unwrap_or_default();
    let mut map = serializer.serialize_map(Some(header_map.len()))?;
    for (k, v) in &header_map {
        map.serialize_entry(&k.as_str(), &v.to_str().map_err(serde::ser::Error::custom)?)?;
    }
    map.end()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keys {
    #[serde(deserialize_with = "deserialize_vec_secret_key")]
    pub private_keys: Vec<SecretKey>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub use_single_chain_for_all_workers: bool,
    pub websocket_polling_ms: u64,
    pub loop_delay_ms: u64,
    pub eip1559: bool,
    pub workers: usize,
    pub network: NetworkConfig,
    pub mev: MevConfig,

    #[serde(deserialize_with = "deserialize_h160")]
    pub bot_contract_address: H160,

    #[serde(deserialize_with = "deserialize_h160")]
    pub dex_router_address: H160,
    pub buy: BuyConfig,
    pub gas: GasConfig,
    pub approve_gas: GasConfig,
    pub stop_after_first_fail: bool,
    pub sell: SellConfig,
    pub dev_action: DevActionConfig,
    pub check: CheckConfig,
    pub w_token_amount_for_buybot_tax_checks: f64,
    pub blocks_delay_before_first_buy: u8,
    pub wait_before_first_buy_m_s: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    pub providers: Vec<String>,

    #[serde(deserialize_with = "deserialize_h160")]
    pub chain_token_address: H160,
    pub currency: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyConfig {
    #[serde(deserialize_with = "deserialize_h160")]
    pub purchase_token_address: H160,

    #[serde(deserialize_with = "deserialize_h160")]
    pub liquidity_token_address: H160,

    #[serde(deserialize_with = "deserialize_h160")]
    pub dev_wallet_address: H160,
    pub rounds: u8,
    pub snipers: usize,
    pub pre_approve: bool,
    pub pre_sign: bool,
    pub method: u8,
    pub token_amount: f64,
    pub percent_of_total_supply: f64,
    pub chain_token_spend_limit: f64,
    pub use_txid: bool,
    pub id_salt: String,
    pub approve_to: String,
    pub include_caller: bool,
    pub use_buybot_checks: bool,
    pub check_sellability: bool,
    pub max_buy_tax: u64,
    pub max_sell_tax: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasConfig {
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_limit: U256,

    #[serde(deserialize_with = "deserialize_gas")]
    pub max_fee_per_gas: U256,

    #[serde(deserialize_with = "deserialize_gas")]
    pub max_priority_fee_per_gas: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellConfig {
    pub sell_percentage: u64,
    pub gas_multiplier: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevActionConfig {
    pub action: String,

    #[serde(deserialize_with = "deserialize_vec_signatures")]
    pub dev_action_ids: Vec<[u8; 4]>,

    #[serde(deserialize_with = "deserialize_vec_signatures")]
    pub dev_action_ignored_ids: Vec<[u8; 4]>,

    pub minimum_liquidity: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckConfig {
    pub high_gas_tx: HighGasTxConfig,
    pub anti_rug_pull: AntiRugPullConfig,
    pub anti_blacklist: AntiBlacklistConfig,
    pub anti_toxic: AntiToxicConfig,
    pub balance_check_multiplier: BalanceCheckMultiplierConfig,
    pub sell_on_percentage_gain: SellOnPercentageGainConfig,
    pub skem_gwei: SkemGweiConfig,
    pub purchase_token_enabled: bool,
    pub pregen: PregenConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MevConfig {
    pub enabled: bool,
    pub simulate: bool,

    pub bribe_amount: f64, // eth

    pub endpoints: Vec<Endpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub endpoint: String,
    #[serde(
        default,
        deserialize_with = "deserialize_headermap",
        serialize_with = "serialize_headermap"
    )]
    pub headers: Option<HeaderMap<HeaderValue>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighGasTxConfig {
    pub enabled: bool,
    pub send_with_first_worker_only: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiRugPullConfig {
    pub enabled: bool,
    pub min_purchase_token_pull_percentage: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiBlacklistConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiToxicConfig {
    pub enabled: bool,

    #[serde(deserialize_with = "deserialize_vec_signatures")]
    pub toxic_ids: Vec<[u8; 4]>,

    #[serde(deserialize_with = "deserialize_vec_signatures")]
    pub non_toxic_ids: Vec<[u8; 4]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceCheckMultiplierConfig {
    pub enabled: bool,
    pub balance_multiplier: u8,
    pub priority_and_fee_modifier_in_gwei: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellOnPercentageGainConfig {
    pub enabled: bool,
    pub gain_percentage: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkemGweiConfig {
    pub enabled: bool,
    pub limit: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PregenConfig {
    pub max_priority_fee_per_gas: GasStepsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasStepsConfig {
    #[serde(deserialize_with = "deserialize_gas")]
    pub from: U256,

    #[serde(deserialize_with = "deserialize_gas")]
    pub to: U256,

    #[serde(deserialize_with = "deserialize_gas")]
    pub step: U256,

    #[serde(skip)]
    current: U256,
}

impl Iterator for GasStepsConfig {
    type Item = U256;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.to {
            None
        } else {
            let result = self.current;
            self.current += self.step;
            Some(result)
        }
    }
}

static mut CONFIG: Option<Config> = None;
static mut PRIVATE_KEYS: Option<Vec<SecretKey>> = None;
pub static mut CONFIG_JSON: Value = Value::Null;

impl Config {
    pub fn get_approve_snipers(&self) -> usize {
        max(self.workers, self.buy.snipers)
    }
}

pub fn get_private_keys() -> &'static Vec<SecretKey> {
    unsafe {
        if let Some(private_keys) = &PRIVATE_KEYS {
            private_keys
        } else {
            let keys: Keys = read_object("./pkeys.json").unwrap();

            if keys.private_keys.is_empty() {
                eprintln!(
                    "{}{}",
                    "Config file has not been read or private keys not configured in".red(),
                    " pkeys.json".yellow()
                );
                process::exit(1);
            }

            PRIVATE_KEYS = Some(keys.private_keys);
            PRIVATE_KEYS.as_ref().unwrap()
        }
    }
}

pub fn get_config() -> &'static Config {
    unsafe {
        if let Some(config) = &CONFIG {
            config
        } else {
            CONFIG_JSON = read_object("./config.json").unwrap();
            let config: Config = serde_json::from_value(CONFIG_JSON.clone()).unwrap();

            let private_keys = get_private_keys();

            if private_keys.len() < config.workers {
                eprintln!(
                    "{} keys: {}, workers: {}",
                    "Not enough private keys configured, should be not less than workers".red(),
                    private_keys.len(),
                    config.workers
                );
                process::exit(1);
            }

            if config.buy.percent_of_total_supply == 0.0 && config.buy.method == 1 {
                eprintln!(
                    "{} percentOfTotalSupply: {}, method: {}",
                    "Percent of total supply cannot be 0 when using method 1".red(),
                    config.buy.percent_of_total_supply,
                    config.buy.method
                );
                process::exit(1);
            }
            CONFIG = Some(config);
            CONFIG.as_ref().unwrap()
        }
    }
}

fn read_object<'a, T>(path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned + Debug,
{
    let data = std::fs::read_to_string(path)?;
    let t: T = serde_json::from_str(&data)?;

    Ok(t)
}

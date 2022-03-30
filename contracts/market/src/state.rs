use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, ReadonlySingleton, Singleton};

pub const KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";
const DEPOSITS: &[u8] = b"deposit";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub stable_denom: String,
    pub aterra_code_id: u64,
    pub interest: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub contract_addr: CanonicalAddr,
    pub aterra_contract: CanonicalAddr,
    pub stable_denom: String,
    pub interest_rate: Decimal,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub aterra_contract: String,
    pub stable_denom: String,
    pub interest_rate: Decimal,
}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositInfo {
    pub balance: Uint128,
    pub interested_balance: Uint128,
    pub last_interaction: u64,
    pub initial_interaction: u64,
}

pub fn store_deposit_info(
    storage: &mut dyn Storage,
    ident: &CanonicalAddr,
    deposit: &DepositInfo,
) -> StdResult<()> {
    bucket(storage, DEPOSITS).save(ident.as_slice(), deposit)
}

pub fn read_deposit_info(storage: &dyn Storage, ident: &CanonicalAddr) -> DepositInfo {
    match bucket_read(storage, DEPOSITS).load(ident.as_slice()) {
        Ok(v) => v,
        _ => DepositInfo {
            balance: Uint128::zero(),
            interested_balance: Uint128::zero(),
            last_interaction: 0,
            initial_interaction: 0,
        },
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub accrued_interest_payments: Uint128,
}

pub fn store_state(storage: &mut dyn Storage, data: &State) -> StdResult<()> {
    Singleton::new(storage, KEY_STATE).save(data)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    ReadonlySingleton::new(storage, KEY_STATE).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    Ident { address: String },
}

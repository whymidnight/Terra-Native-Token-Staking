use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

pub const KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";

const PREFIX_LIABILITY: &[u8] = b"liability";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub contract_addr: CanonicalAddr,
    pub aterra_contract: CanonicalAddr,
    pub stable_denom: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub aterra_contract: String,
    pub stable_denom: String,
}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

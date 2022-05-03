use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, ReadonlySingleton, Singleton};
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;

pub const KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";
const DEPOSITS: &[u8] = b"deposit";
const TVLS: &[u8] = b"tvl_history";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

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
    pub last_interaction: u64,
    pub last_balance: Uint128,
    pub accrued_interest: Uint128,
    pub initial_interaction: u64,
    pub sum_deposits: Uint128,
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
            last_interaction: 0,
            last_balance: Uint128::zero(),
            accrued_interest: Uint128::zero(),
            initial_interaction: 0,
            sum_deposits: Uint128::zero(),
        },
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub tvl: Uint128,
    pub tvl_indices: i64,
    pub accrued_interest_payments: Uint128,
}

pub fn store_state(storage: &mut dyn Storage, data: &State) -> StdResult<()> {
    Singleton::new(storage, KEY_STATE).save(data)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    ReadonlySingleton::new(storage, KEY_STATE).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Tvl {
    pub tvl: Uint128,
    pub epoch: u64,
}

pub fn store_tvl_indice(
    storage: &mut dyn Storage,
    data: &mut Tvl,
    direction: usize,
) -> Result<(), ContractError> {
    let mut state: State = ReadonlySingleton::new(storage, KEY_STATE).load()?;
    let epoch_counter = state.tvl_indices.clone();
    state.tvl_indices += 1;

    if direction == 1 {
        state.tvl += data.tvl;
        data.tvl = state.tvl;
    } else if direction == 0 {
        state.tvl -= data.tvl;
        data.tvl = state.tvl;
    }
    Singleton::new(storage, KEY_STATE).save(&state)?;

    match bucket_read(storage, TVLS).load(&epoch_counter.to_le_bytes()) {
        Ok(()) => Err(ContractError::Overflow {}),
        _ => {
            bucket(storage, TVLS).save(&epoch_counter.to_le_bytes(), data)?;
            Ok(())
        }
    }
}

pub fn read_tvl_indice(storage: &dyn Storage, indice: i64) -> Tvl {
    match bucket_read(storage, TVLS).load(&indice.to_le_bytes()) {
        Ok(v) => v,
        _ => Tvl {
            tvl: Uint128::zero(),
            epoch: 0,
        },
    }
}

pub fn read_tvl_indices(storage: &dyn Storage, tvl_indices: i64) -> StdResult<Vec<Tvl>> {
    let mut tvls = vec![
        Tvl {
            tvl: Uint128::zero(),
            epoch: 0,
        };
        tvl_indices as usize
    ];

    let mut epoch_counter: i64 = 0;
    while epoch_counter < tvl_indices {
        /*
        tvls[epoch_counter as usize] = Tvl {
            tvl: Uint128::zero(),
            epoch: 69420,
        };
        */
        tvls[epoch_counter as usize] = read_tvl_indice(storage, epoch_counter);
        epoch_counter += 1;
    }

    Ok(tvls)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    Ident { address: String, epoch: u64 },
    Tvl { indice: i64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemNStable {},
    RedeemAllStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    DepositStable {},
    ClaimRewards { to: Option<String> },
}


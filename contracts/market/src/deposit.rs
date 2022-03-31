use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    Uint128, WasmMsg,
};

use crate::error::ContractError;
use crate::helpers::*;
use crate::state::{
    read_config, read_deposit_info, read_state, store_deposit_info, store_state, Config, State,
};

use cw20::Cw20ExecuteMsg;

// TODO: CHANGE TO 24 HOURS
const DURATION: u64 = 30;

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let time = env.block.time.seconds();
    // we need to _un_account global accrued_interest_payments by previous interested_balance
    // to maintain congruency of how much to fund contract wallet bank since we must only add
    // to global accrued_interest_payments once per ident.
    let mut state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;
    let ident = info.sender;
    let ident_raw = deps.api.addr_canonicalize(ident.as_str())?;

    // Check base denom deposit
    let deposit_amount: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(ContractError::ZeroDeposit(config.stable_denom));
    }
    let mut deposit = read_deposit_info(deps.storage, &ident_raw);
    if deposit.initial_interaction == 0 {
        deposit.initial_interaction = time;
    }
    if deposit.last_interaction == 0 {
        deposit.last_interaction = time;
    }
    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);
    deposit.last_interaction = time;

    let mut accrued_interest = Uint128::zero();
    if days > 0 {
        accrued_interest = calculate_accrued_interest(&config, &deposit, days)?;
        state.accrued_interest_payments += accrued_interest;
    }
    deposit.last_balance += deposit_amount;
    deposit.interested_balance += accrued_interest;

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    store_state(deps.storage, &state)?;
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: ident.to_string(),
                amount: deposit_amount + accrued_interest,
            })?,
        }))
        .add_attributes(vec![
            attr("action", "deposit_stable"),
            attr("depositor", ident),
            attr("days", days.to_string()),
            attr("duration", duration.to_string()),
            attr("deposit_amount", deposit_amount),
            attr("accrued_interest", accrued_interest),
            attr("interested_balance", deposit.interested_balance),
            attr("last_balance", deposit.last_balance),
        ]))
}

pub fn redeem_all_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    withdraw_amount: Uint128,
) -> Result<Response, ContractError> {
    let time = env.block.time.seconds();
    // let mut state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;
    let ident = sender;
    let ident_raw = deps.api.addr_canonicalize(ident.as_str())?;

    let mut deposit = read_deposit_info(deps.storage, &ident_raw);
    if deposit.last_balance != (withdraw_amount - deposit.interested_balance) {
        return Err(ContractError::Std(StdError::parse_err(
            "InvalidWithdrawal",
            "Must withdraw all to use this fn",
        )));
    }

    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);

    deposit.last_interaction = time;
    let mut withdraw_amount = Uint128::zero();
    if days > 0 {
        withdraw_amount =
            calculate_accrued_interest(&config, &deposit, days)? + deposit.last_balance.clone();
    }
    deposit.interested_balance = Uint128::zero();
    deposit.last_interaction = 0;
    deposit.interested_balance = Uint128::zero();
    deposit.initial_interaction = 0;

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: withdraw_amount,
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: ident.to_string(),
                amount: vec![Coin {
                    denom: config.stable_denom,
                    amount: withdraw_amount,
                }],
            }),
        ])
        .add_attributes(vec![
            attr("action", "redeem_all_stable"),
            attr("days", days.to_string()),
            attr("duration", duration.to_string()),
            attr("withdraw_amount", withdraw_amount),
            attr("interested_balance", deposit.interested_balance),
            attr("last_balance", deposit.last_balance),
        ]))
}

pub fn redeem_n_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    withdraw_amount: Uint128,
) -> Result<Response, ContractError> {
    let time = env.block.time.seconds();
    let mut state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;
    let ident = sender;
    let ident_raw = deps.api.addr_canonicalize(ident.as_str())?;

    let mut deposit = read_deposit_info(deps.storage, &ident_raw);
    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);

    deposit.last_interaction = time;
    let mut accrued_interest = Uint128::zero();
    if days > 0 {
        accrued_interest = calculate_accrued_interest(&config, &deposit, days)?;
        state.accrued_interest_payments += accrued_interest;
        deposit.interested_balance += accrued_interest;
    }
    deposit.last_balance -= withdraw_amount;

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: withdraw_amount,
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: ident.to_string(),
                amount: vec![Coin {
                    denom: config.stable_denom,
                    amount: withdraw_amount,
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: ident.to_string(),
                    amount: accrued_interest,
                })?,
            }),
        ])
        .add_attributes(vec![
            attr("action", "redeem_n_stable"),
            attr("days", days.to_string()),
            attr("duration", duration.to_string()),
            attr("withdraw_amount", withdraw_amount),
            attr("interested_balance", deposit.interested_balance),
            attr("last_balance", deposit.last_balance),
        ]))
}

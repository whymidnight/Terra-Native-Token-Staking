use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
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
    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);
    deposit.last_interaction = time;

    let mut accrued_interest = Uint128::zero();
    if days > 0 {
        accrued_interest = calculate_accrued_interest(&config, &deposit, days)?;
        state.accrued_interest_payments -= deposit.interested_balance;
        state.accrued_interest_payments += accrued_interest;
    }
    deposit.balance += deposit_amount;
    deposit.interested_balance = accrued_interest;

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    store_state(deps.storage, &state)?;
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: ident.to_string(),
                amount: deposit_amount,
            })?,
        }))
        .add_attributes(vec![
            attr("action", "deposit_stable"),
            attr("depositor", ident),
            attr("days", days.to_string()),
            attr("duration", duration.to_string()),
            attr("deposit_amount", deposit_amount),
            attr("accrued_interest", accrued_interest),
        ]))
}

pub fn redeem_stable(
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
    deposit.balance -= withdraw_amount;
    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);

    let mut accrued_interest = Uint128::zero();
    if days > 0 {
        accrued_interest = calculate_accrued_interest(&config, &deposit, days)?;
        state.accrued_interest_payments -= deposit.interested_balance;
        state.accrued_interest_payments += accrued_interest;
    }

    deposit.interested_balance = accrued_interest;
    deposit.last_interaction = time;
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
            attr("action", "redeem_stable"),
            attr("withdraw_amount", withdraw_amount),
        ]))
}

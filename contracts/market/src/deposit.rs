use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};

use crate::contract::DURATION;
use crate::error::ContractError;
use crate::helpers::*;
use crate::state::{
    read_config, read_deposit_info, read_state, store_deposit_info, store_state, store_tvl_indice,
    Config, State, Tvl,
};

use cw20::Cw20ExecuteMsg;

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let time = env.block.time.seconds();
    let config: Config = read_config(deps.storage)?;
    let ident = info.sender;
    let ident_raw = deps.api.addr_canonicalize(ident.as_str())?;

    let deposit_amount: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

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
        let mut state: State = read_state(deps.storage)?;
        accrued_interest = calculate_accrued_interest(&deposit, config.interest_rate, days)?;
        state.accrued_interest_payments += accrued_interest;
        store_state(deps.storage, &state)?;
    }
    let _tvl_put = store_tvl_indice(
        deps.storage,
        &mut Tvl {
            epoch: time,
            tvl: deposit_amount + accrued_interest,
        },
        1,
    )
    .unwrap();
    deposit.accrued_interest += accrued_interest;
    deposit.last_balance += deposit_amount + accrued_interest;
    deposit.sum_deposits += deposit_amount;

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: ident.to_string(),
                amount: deposit_amount + accrued_interest,
            })?,
        }))
        .add_attributes(vec![attr("action", "deposit_stable")]))
}

pub fn redeem_n_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    withdraw_amount: Uint128,
) -> Result<Response, ContractError> {
    let time = env.block.time.seconds();
    let config: Config = read_config(deps.storage)?;
    let ident = sender;
    let ident_raw = deps.api.addr_canonicalize(ident.as_str())?;
    let mut response_ixs: Vec<CosmosMsg> = Vec::new();

    let mut deposit = read_deposit_info(deps.storage, &ident_raw);
    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);

    deposit.last_interaction = time;
    let mut accrued_interest = Uint128::zero();
    if days > 0 {
        accrued_interest = calculate_accrued_interest(&deposit, config.interest_rate, days)?;
        let mut state: State = read_state(deps.storage)?;
        state.accrued_interest_payments += accrued_interest;
        store_state(deps.storage, &state)?;
        deposit.last_balance += accrued_interest;

        response_ixs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: ident.to_string(),
                amount: accrued_interest,
            })?,
        }))
    }
    let _tvl_put = store_tvl_indice(
        deps.storage,
        &mut Tvl {
            epoch: time,
            tvl: withdraw_amount,
        },
        0,
    )
    .unwrap();
    deposit.accrued_interest += accrued_interest;
    deposit.last_balance -= withdraw_amount;
    response_ixs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: withdraw_amount,
        })?,
    }));
    response_ixs.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: ident.to_string(),
        amount: vec![Coin {
            denom: config.stable_denom,
            amount: withdraw_amount,
        }],
    }));

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    Ok(Response::new()
        .add_messages(response_ixs)
        .add_attributes(vec![attr("action", "redeem_n_stable")]))
}

pub fn redeem_all_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    _withdrawal: Uint128,
) -> Result<Response, ContractError> {
    let time = env.block.time.seconds();
    let config: Config = read_config(deps.storage)?;
    let ident = sender;
    let ident_raw = deps.api.addr_canonicalize(ident.as_str())?;

    let mut deposit = read_deposit_info(deps.storage, &ident_raw);
    let last_interaction = deposit.last_interaction;
    let duration = time - last_interaction;
    let days = duration / (DURATION);
    deposit.last_interaction = time;

    let burn_amount = deposit.last_balance.clone();
    let mut withdraw_amount = deposit.last_balance.clone();
    withdraw_amount += calculate_accrued_interest(&deposit, config.interest_rate, days)?;
    let _tvl_put = store_tvl_indice(
        deps.storage,
        &mut Tvl {
            epoch: time,
            tvl: burn_amount,
        },
        0,
    )
    .unwrap();
    deposit.accrued_interest = Uint128::zero();
    deposit.initial_interaction = 0;
    deposit.last_balance = Uint128::zero();
    deposit.last_interaction = 0;
    deposit.sum_deposits = Uint128::zero();

    store_deposit_info(deps.storage, &ident_raw, &deposit)?;
    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: burn_amount,
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
        .add_attributes(vec![attr("action", "redeem_all_stable")]))
}

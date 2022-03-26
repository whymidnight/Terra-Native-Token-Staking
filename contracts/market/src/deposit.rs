use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};

use crate::error::ContractError;
use crate::state::{read_config, Config};

use cw20::Cw20ExecuteMsg;
use moneymarket::querier::{deduct_tax, query_balance};

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Check base denom deposit
    let deposit_amount: Uint256 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(ContractError::ZeroDeposit(config.stable_denom));
    }

    let mint_amount = deposit_amount;
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: mint_amount.into(),
            })?,
        }))
        .add_attributes(vec![
            attr("action", "deposit_stable"),
            attr("depositor", info.sender),
            attr("mint_amount", mint_amount),
            attr("deposit_amount", deposit_amount),
        ]))
}

pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    burn_amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let current_balance = query_balance(
        deps.as_ref(),
        env.contract.address,
        config.stable_denom.to_string(),
    )?;

    // Assert redeem amount
    // assert_redeem_amount(&config, &state, current_balance, redeem_amount)?;

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
                to_address: sender.to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: config.stable_denom,
                        amount: Uint256::from(0 as u128).into(),
                    },
                )?],
            }),
        ])
        .add_attributes(vec![
            attr("action", "redeem_stable"),
            attr("burn_amount", burn_amount),
            // attr("redeem_amount", redeem_amount),
        ]))
}

fn assert_redeem_amount(
    config: &Config,
    // state: &State,
    current_balance: Uint256,
    redeem_amount: Uint256,
) -> Result<(), ContractError> {
    let current_balance = Decimal256::from_uint256(current_balance);
    let redeem_amount = Decimal256::from_uint256(redeem_amount);
    if redeem_amount + Decimal256::from_uint256(0u128) > current_balance {
        return Err(ContractError::NoStableAvailable(
            config.stable_denom.clone(),
        ));
    }

    Ok(())
}

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::deposit::{deposit_stable, redeem_stable};
use crate::error::ContractError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_config, store_config, Config, ConfigResponse};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg, MinterResponse};

use moneymarket::common::optional_addr_validate;
use moneymarket::interest_model::BorrowRateResponse;
use moneymarket::market::{
    Cw20HookMsg, EpochStateResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};
use moneymarket::querier::{deduct_tax, query_balance, query_supply};
use protobuf::Message;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

pub const INITIAL_DEPOSIT_AMOUNT: u128 = 1000000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let initial_deposit = info
        .funds
        .iter()
        .find(|c| c.denom == msg.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if initial_deposit != Uint128::from(INITIAL_DEPOSIT_AMOUNT) {
        return Err(ContractError::InitialFundsNotDeposited(
            INITIAL_DEPOSIT_AMOUNT,
            msg.stable_denom,
        ));
    }

    store_config(
        deps.storage,
        &Config {
            contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
            aterra_contract: CanonicalAddr::from(vec![]),
            stable_denom: msg.stable_denom.clone(),
        },
    )?;

    Ok(
        Response::new().add_submessages(vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: msg.aterra_code_id,
                funds: vec![],
                label: "".to_string(),
                msg: to_binary(&TokenInstantiateMsg {
                    name: format!("Anchor Terra {}", msg.stable_denom[1..].to_uppercase()),
                    symbol: format!(
                        "a{}T",
                        msg.stable_denom[1..(msg.stable_denom.len() - 1)].to_uppercase()
                    ),
                    decimals: 6u8,
                    initial_balances: vec![Cw20Coin {
                        address: env.contract.address.to_string(),
                        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
                    }],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })?,
            }),
            1,
        )]),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::DepositStable {} => deposit_stable(deps, env, info),
        ExecuteMsg::RegisterContracts {
            overseer_contract,
            interest_model,
            distribution_model,
            collector_contract,
            distributor_contract,
        } => Ok(Response::default()),
        ExecuteMsg::UpdateConfig {
            owner_addr,
            interest_model,
            distribution_model,
            max_borrow_factor,
        } => Ok(Response::default()),
        ExecuteMsg::ExecuteEpochOperations {
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            distributed_interest,
        } => Ok(Response::default()),
        ExecuteMsg::BorrowStable { borrow_amount, to } => Ok(Response::default()),
        ExecuteMsg::RepayStable {} => Ok(Response::default()),
        ExecuteMsg::RepayStableFromLiquidation {
            borrower,
            prev_balance,
        } => Ok(Response::default()),
        ExecuteMsg::ClaimRewards { to } => Ok(Response::default()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => {
            // get new token's contract address
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data",
                ))
            })?;
            let token_addr = Addr::unchecked(res.get_contract_address());

            register_aterra(deps, token_addr)
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender;
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::RedeemStable {}) => {
            // only asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != config.aterra_contract {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            redeem_stable(deps, env, cw20_sender_addr, cw20_msg.amount)
        }
        _ => Err(ContractError::MissingRedeemStableHook {}),
    }
}

pub fn register_aterra(deps: DepsMut, token_addr: Addr) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if config.aterra_contract != CanonicalAddr::from(vec![]) {
        return Err(ContractError::Unauthorized {});
    }

    config.aterra_contract = deps.api.addr_canonicalize(token_addr.as_str())?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("aterra", token_addr)]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_height } => to_binary(&()),
        QueryMsg::EpochState {
            block_height,
            distributed_interest,
        } => to_binary(&()),
        QueryMsg::BorrowerInfo {
            borrower,
            block_height,
        } => to_binary(&()),
        QueryMsg::BorrowerInfos { start_after, limit } => to_binary(&()),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        aterra_contract: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
        stable_denom: config.stable_denom,
    })
}

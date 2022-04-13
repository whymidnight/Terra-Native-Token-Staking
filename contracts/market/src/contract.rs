#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::deposit::{deposit_stable, redeem_all_stable, redeem_n_stable};
use crate::error::ContractError;
use crate::helpers::{calculate_accrued_interest, get_decimals};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{
    read_config, read_deposit_info, read_state, read_tvl_indice, read_tvl_indices, store_config,
    store_state, store_tvl_indice, Config, ConfigResponse, DepositInfo, InstantiateMsg, QueryMsg,
    State, Tvl,
};

use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg, MinterResponse};

use crate::state::{Cw20HookMsg, ExecuteMsg};
use protobuf::Message;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

// TODO: CHANGE TO 24 HOURS
pub const DURATION: u64 = 30;
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
        .find(|c| c.denom == "uusd".to_string())
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if initial_deposit != Uint128::from(INITIAL_DEPOSIT_AMOUNT) {
        return Err(ContractError::InitialFundsNotDeposited(
            INITIAL_DEPOSIT_AMOUNT,
            "uusd".to_string(),
        ));
    }

    store_config(
        deps.storage,
        &Config {
            contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
            aterra_contract: CanonicalAddr::from(vec![]),
            stable_denom: msg.stable_denom.clone(),
            interest_rate: get_decimals(msg.interest)?,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            tvl: Uint128::zero(),
            tvl_indices: 0,
            accrued_interest_payments: Uint128::zero(),
        },
    )?;

    let _tvl_store = store_tvl_indice(
        deps.storage,
        &mut Tvl {
            tvl: Uint128::zero(),
            epoch: 0,
        },
        0,
    )
    .unwrap();

    Ok(
        Response::new().add_submessages(vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: msg.aterra_code_id,
                funds: vec![],
                label: "".to_string(),
                msg: to_binary(&TokenInstantiateMsg {
                    name: format!("yxz {}", msg.stable_denom[1..].to_uppercase()),
                    symbol: format!(
                        "xyz{}T",
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
        ExecuteMsg::ClaimRewards { to: _ } => Ok(Response::default()),
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
        Ok(Cw20HookMsg::RedeemNStable {}) => {
            let config: Config = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != config.aterra_contract {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            redeem_n_stable(deps, env, cw20_sender_addr, cw20_msg.amount)
        }
        Ok(Cw20HookMsg::RedeemAllStable {}) => {
            let config: Config = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != config.aterra_contract {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            redeem_all_stable(deps, env, cw20_sender_addr, cw20_msg.amount)
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
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::Ident { address, epoch } => to_binary(&query_ident(deps, address, epoch)?),
        QueryMsg::Tvl { indice } => to_binary(&query_tvl(deps, indice)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        aterra_contract: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
        stable_denom: config.stable_denom,
        interest_rate: config.interest_rate,
    })
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    let state = read_state(deps.storage)?;
    Ok(state.clone())
}

pub fn query_ident(deps: Deps, ident: String, epoch: u64) -> StdResult<DepositInfo> {
    let config: Config = read_config(deps.storage)?;
    let mut depositor = read_deposit_info(deps.storage, &deps.api.addr_canonicalize(&ident)?);
    if depositor.initial_interaction != 0 && epoch > depositor.last_interaction {
        let duration = epoch - depositor.last_interaction.clone();
        let days = duration / (DURATION);
        let accrued_interest = calculate_accrued_interest(&depositor, config.interest_rate, days)?;
        depositor.accrued_interest = accrued_interest;
    }
    Ok(depositor.clone())
}

pub fn query_tvl(deps: Deps, indice: i64) -> StdResult<Vec<Tvl>> {
    let tvls: Vec<Tvl>;
    if indice == -1 {
        let state = read_state(deps.storage)?;
        tvls = read_tvl_indices(deps.storage, state.tvl_indices)?;
    } else {
        tvls = vec![read_tvl_indice(deps.storage, indice)];
    }
    Ok(tvls.clone())
}

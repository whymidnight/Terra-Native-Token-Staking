use crate::contract::{execute, instantiate, query, reply, INITIAL_DEPOSIT_AMOUNT};
use crate::error::ContractError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::InstantiateMsg;
use crate::testing::mock_querier::mock_dependencies;

use anchor_token::distributor::ExecuteMsg as FaucetExecuteMsg;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Reply,
    SubMsg, SubMsgExecutionResponse, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use moneymarket::market::{
    BorrowerInfoResponse, ConfigResponse, Cw20HookMsg, ExecuteMsg, QueryMsg, StateResponse,
};
use moneymarket::querier::deduct_tax;
use protobuf::Message;
use serde_json::json;
use std::str::FromStr;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

#[test]
fn redeem_stable() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
    }]);

    let msg = InstantiateMsg {
        stable_denom: "uusd".to_string(),
        aterra_code_id: 123u64,
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Deposit 1000000
    let msg = ExecuteMsg::DepositStable {};
    let info = mock_info(
        "terra1gd3v0lsxhe98f05xr5yd67cjg26vvwhpmmyfer",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000000u128),
        }],
    );

    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(2000000u128))],
    )]);

    // Redeem 1000000
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
        msg: to_binary(&Cw20HookMsg::RedeemStable {}).unwrap(),
    });
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("AT-uusd", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "AT-uusd".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(1000000u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "addr0000".to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(1000000u128),
                    }
                )
                .unwrap(),]
            }))
        ]
    );

    let _uusd_string = "uusd";
    println!("{:?}", res);

    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(600000u128),
        }],
    );

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "AT-uusd".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::from(1000000u128),
                })
                .unwrap()
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "addr0000".to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: "uusd".to_string(),
                        amount: Uint128::from(500000u128),
                    }
                )
                .unwrap(),]
            }))
        ]
    );
}

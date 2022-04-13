use crate::helpers::calculate_accrued_interest;
use crate::state::DepositInfo;

use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Reply,
    StdError, StdResult, SubMsg, SubMsgExecutionResponse, Uint128, WasmMsg,
};
use serde_json::*;
use std::str::FromStr;

#[test]
fn redeem_stable() {
    let last_balance: u64 = 8000000;
    let rate: String = "0.000382982750338989".to_string();
    let decimals = Decimal::from_str(&rate).unwrap();
    println!("{:?}", decimals);
    let accrued_interest = calculate_accrued_interest(
        &DepositInfo {
            interested_balance: Uint128::zero(),
            last_interaction: 1,
            last_balance: Uint128::from(last_balance),
            initial_interaction: 1,
        },
        decimals,
        0,
    );

    // println!("{}", json!(accrued_interest).to_string());
    println!("{:?}", accrued_interest);
}


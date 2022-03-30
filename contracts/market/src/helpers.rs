use crate::state::{Config, DepositInfo};
use cosmwasm_std::{Decimal, StdError, StdResult, Uint128};
use std::str::FromStr;

// https://gist.github.com/whymidnight/ed98dd59a73036038785e899be8c3ba9
pub fn calculate_accrued_interest(
    config: &Config,
    deposit: &DepositInfo,
    days: u64,
) -> StdResult<Uint128> {
    let mut compounded_yield = Uint128::zero();
    for _day in 0..days {
        // += (1.000 * 0.145) + 1
        // += (1.145 * 0.145) + 1
        // += (2.311 * 0.145) + 1
        compounded_yield += deposit.balance.clone() * config.interest_rate;
    }
    Ok(compounded_yield)
}

pub fn get_decimals(value: String) -> StdResult<Decimal> {
    let parts: &[&str] = &*value.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal::zero()),
        2 => {
            let decimals = Decimal::from_str(&*("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(StdError::generic_err("Unexpected number of dots")),
    }
}


use crate::state::DepositInfo;
use cosmwasm_std::{Decimal, StdError, StdResult, Uint128};
use std::str::FromStr;

/*
    let updatedTotal = userTotal
    for each day in daysSinceLastInteraction{
        updatedTotal += ( updatedTotal * InterestRate )
    }

    InterestDifferenceAmount = ( updatedtotal - userTotal )
    atokenMintAmt = (interestDifferenceAmount + Deposit)
    MintAtokens(atokenMintAmt)
    userTotal = updatedTotal
*/
pub fn calculate_accrued_interest(
    deposit: &DepositInfo,
    interest_rate: Decimal,
    days: u64,
) -> StdResult<Uint128> {
    if days == 0 {
        return Ok(Uint128::zero());
    }
    let mut interested_balance = deposit.last_balance.clone();
    let mut counter: u64 = 0;
    while counter < days {
        interested_balance += interested_balance * interest_rate;
        counter += 1;
    }
    Ok(interested_balance - deposit.last_balance.clone())
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

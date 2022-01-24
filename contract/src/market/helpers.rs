use crate::market::constants::*;
use crate::market::ext_interface::ft_token;
use near_sdk::{env, require, AccountId, Gas, Promise};

pub fn assert_owner() {
    require!(
        env::current_account_id() == env::predecessor_account_id(),
        ERR04_PERMISSION_DENIED
    )
}

#[inline]
pub fn get_next_gas() -> Gas {
    env::prepaid_gas() - env::used_gas() - FT_TRANSFER_TGAS - RESERVE_TGAS
}

pub fn compose_key(sell_token: &AccountId, buy_token: &AccountId) -> String {
    let mut key = String::from(sell_token.as_str());
    key.push_str("#");
    key.push_str(buy_token.as_str());
    key
}

#[inline]
pub fn ft_transfer<S: AsRef<str>>(
    receiver_id: &AccountId,
    amount: u128,
    msg: S,
    token: &AccountId,
) -> Promise {
    ft_token::ft_transfer(
        receiver_id.clone(),
        amount.into(),
        msg.as_ref().to_string(),
        token.clone(),
        ONE_YOCTO,
        FT_TRANSFER_TGAS,
    )
}

#[macro_export]
macro_rules! log {
    () => (near_sdk::env::log_str("\n"));
    ($($arg:tt)*) => ({
        near_sdk::env::log_str(&format!("{}", format_args!($($arg)*)));
    })
}

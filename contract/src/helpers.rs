use crate::market::{FT_TRANSFER_TGAS, ONE_YOCTO};
use crate::{ft_token, ERR04_PERMISSION_DENIED};
use near_sdk::json_types::U128;
use near_sdk::{env, AccountId, Promise, PromiseOrValue};

pub fn assert_owner() {
    assert_eq!(
        env::current_account_id(),
        env::predecessor_account_id(),
        "{}",
        ERR04_PERMISSION_DENIED
    );
}

#[inline]
pub fn ft_transfer<S: AsRef<str>>(
    receiver_id: AccountId,
    amount: u128,
    msg: S,
    token: &AccountId,
) -> Promise {
    let a = format_args!("{}", "a");
    ft_token::ft_transfer(
        receiver_id,
        amount.into(),
        msg.as_ref().to_string(),
        token.clone(),
        ONE_YOCTO,
        FT_TRANSFER_TGAS,
    )
}

// #[macro_export]
// macro_rules! log {
//     () => (near_sdk::env::log_str("\n"));
//     ($($arg:tt)*) => ({
//         near_sdk::env::log_str(&format!("{}", format_args!($($arg)*)));
//     })
// }

use near_sdk::env;
use crate::ERR04_PERMISSION_DENIED;

pub fn assert_owner() {
    assert_eq!(
        env::current_account_id(),
        env::predecessor_account_id(),
        "{}",
        ERR04_PERMISSION_DENIED
    );
}
use crate::market::fees::Fee;
use crate::market::Market;
use near_sdk::borsh::BorshSerialize;
use near_sdk::BorshStorageKey;
use near_sdk::{borsh, AccountId};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Orders,
    OrderIdToOrder,
    FeesByAccountIds,
}

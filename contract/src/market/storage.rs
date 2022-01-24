use near_sdk::borsh;
use near_sdk::borsh::BorshSerialize;
use near_sdk::BorshStorageKey;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Orders,
    OrderIdToOrder,
    FeesByAccountIds,
}

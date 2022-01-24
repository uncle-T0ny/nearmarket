use crate::market::order_id::OrderId;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{borsh, AccountId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NewOrderAction {
    pub sell_token: AccountId,
    pub sell_amount: U128,
    pub buy_token: AccountId,
    pub buy_amount: U128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderView {
    pub order: Order,
    pub order_id: OrderId,
}

#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Hash,
)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub maker: AccountId,
    pub sell_token: AccountId,
    pub sell_amount: U128,
    pub buy_token: AccountId,
    pub buy_amount: U128,
}

impl Order {
    pub fn get_price_for_key(&self) -> u128 {
        (self.sell_amount.0 + 1000000000000000000000000000000) / self.buy_amount.0
    }

    pub fn get_id(&self) -> OrderId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();

        OrderId(self.get_price_for_key(), hash)
    }

    pub fn from_action(action: NewOrderAction, sender: AccountId) -> Self {
        Order {
            maker: sender,
            sell_token: action.sell_token,
            sell_amount: action.sell_amount,
            buy_token: action.buy_token,
            buy_amount: action.buy_amount,
        }
    }
}

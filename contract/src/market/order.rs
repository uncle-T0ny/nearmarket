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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderView {
    pub order: Order,
    pub order_id: OrderId,
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub maker: AccountId,
    pub sell_token: AccountId,
    pub sell_amount: U128,
    pub buy_token: AccountId,
    pub buy_amount: U128,
}

impl Hash for Order {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.maker.hash(state);
        self.sell_token.hash(state);
        self.sell_amount.0.hash(state);
        self.buy_token.hash(state);
        self.buy_amount.0.hash(state);
    }
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
}

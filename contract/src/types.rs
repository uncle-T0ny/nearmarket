use near_sdk::{
    borsh,
    borsh::{BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    AccountId,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// #[derive(Serialize, Deserialize, Clone, PartialEq)]
// #[serde(crate = "near_sdk::serde")]
// pub enum OrderActions {
//     // Cancel,
//     Match,
// }


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum TokenReceiverMessage {
    Match {
        order_id: U64,
    },
    NewOrderAction {
        sell_token: AccountId,
        sell_amount: U128,
        buy_token: AccountId,
        buy_amount: U128,
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NewOrderAction {
    pub sell_token: AccountId,
    pub sell_amount: U128,
    pub buy_token: AccountId,
    pub buy_amount: U128,
}

// #[derive(Serialize, Deserialize, Clone)]
// #[serde(crate = "near_sdk::serde")]
// pub struct OrderAction {
//     pub order_id: U64,
//     pub order_action: OrderActions,
// }
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderView {
    pub order: Order,
    pub order_id: U64,
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
    pub fn get_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        hash
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

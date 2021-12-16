use near_sdk::{serde::{Serialize, Deserialize}, AccountId, json_types::U128};


#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderActions {
    Cancel,
    Match,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NewOrderAction {
    pub sell_token: AccountId, 
    pub sell_amount: U128, 
    pub buy_token: AccountId, 
    pub buy_amount: U128,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderAction {
    pub order_id: U128,
    pub order_action: OrderActions,
}
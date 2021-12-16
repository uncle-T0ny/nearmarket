use std::ops::Bound;

use crate::types::*;

use errors::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{TreeMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::serde_json::Error;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;
use near_sdk::{env, AccountId, PromiseOrValue};

mod errors;
mod ext_interfaces;
mod types;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    OrdersById,
    MapByOrderId,
    Orders,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
pub struct Market {
    version: u8,
    orders: UnorderedMap<String, TreeMap<u64, Order>>,
}

#[near_bindgen]
impl FungibleTokenReceiver for Market {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id();
        env::log_str(&format!(
            "Transfered {:?} {} from {}",
            amount, token, sender_id
        ));
        env::log_str(&format!("transfer msg: {}", msg));

        let new_order_action: Result<NewOrderAction, Error> = near_sdk::serde_json::from_str(&msg);
        let order_action: Result<OrderAction, Error> = near_sdk::serde_json::from_str(&msg);

        if new_order_action.is_ok() {
            let new_order_action = new_order_action.unwrap();

            // handle new order
            return PromiseOrValue::Value(U128(0));
        }

        if order_action.is_ok() {
            let order_action = order_action.unwrap();
            if order_action.order_action == OrderActions::Match {
                // handle match

                return PromiseOrValue::Value(U128(0));
            }

            if order_action.order_action == OrderActions::Cancel {
                // handle cancel

                return PromiseOrValue::Value(U128(0));
            }
        }

        // refund tokens
        PromiseOrValue::Value(amount)
    }
}

#[near_bindgen]
impl Market {
    #[init]
    pub fn new(version: u8) -> Self {
        let this = Self { 
            version,
            orders: UnorderedMap::new(StorageKey::Orders), 
        };
        this
    }

    pub fn add_order(&mut self, action: NewOrderAction, sender: AccountId) {
        let new_order = Order::from_action(action, sender);

        let key = compose_key(&new_order.sell_token, &new_order.buy_token);
        let order_by_key = self.orders.get(&key);
        let mut orders_map;
        if order_by_key.is_none() {
            orders_map = TreeMap::new(StorageKey::MapByOrderId);
        } else {
            orders_map = order_by_key.unwrap();
        }

        let order_id = new_order.get_id();
        if orders_map.contains_key(&order_id) {
            env::panic_str(ERR02_ORDER_ALREADY_EXISTS);
        }

        orders_map.insert(&order_id, &new_order);

        self.orders.insert(&key, &orders_map);
    }
    
    pub fn remove_order(&mut self, sell_token: AccountId, buy_token: AccountId, order_id: u64) {
        let key = compose_key(&sell_token, &buy_token);
        let order_by_key = self.orders.get(&key);

        if order_by_key.is_none() {
            env::panic_str(ERR03_ORDER_NOT_FOUND);
        } 

        let mut orders_map = order_by_key.unwrap();
        let order = orders_map.get(&order_id);

        if order.is_none() {
            env::panic_str(ERR03_ORDER_NOT_FOUND);
        }

        let maker = order.unwrap().maker;
        if maker != env::predecessor_account_id() {
            env::panic_str(ERR04_PERMISSION_DENIED)
        }

        orders_map.remove(&order_id);  
        
        if orders_map.len() == 0 {
            self.orders.remove(&key);
        } else {
            self.orders.insert(&key, &orders_map);
        }
    }

    pub fn get_orders(&self, sell_token: AccountId, buy_token: AccountId) -> Option<Vec<Order>> {
        let key = compose_key(&sell_token, &buy_token);
        let order_by_key = self.orders.get(&key);

        if order_by_key.is_none() {
            return None;
        } 

        let mut res =  vec![];
        
        let orders = order_by_key.unwrap();
        let order_iter = orders.iter().take(5);
        for order in order_iter {
            res.push(order.1)
        }
        
        return Some(res);
    }

    pub fn get_pairs(&self) -> Vec<String> {
       let keys = self.orders.keys_as_vector();
       keys.to_vec()
    } 
}

fn compose_key(sell_token: &AccountId, buy_token: &AccountId) -> String {
    let mut key = String::from(sell_token.as_str());
    key.push_str("#");
    key.push_str(buy_token.as_str());
    key
}

#[cfg(test)]
mod tests {
    use near_sdk::{collections::{LookupSet, TreeMap, LookupMap}, testing_env, test_utils::VMContextBuilder};

    use super::*;


    #[test]
    fn test_order_hash() {
        let order = Order {
            maker: AccountId::new_unchecked(String::from("maker.near")),
            sell_token: AccountId::new_unchecked(String::from("token2.near")),
            sell_amount: U128(20),
            buy_token: AccountId::new_unchecked(String::from("token1.near")),
            buy_amount: U128(300),
        };

        let order1 = Order {
            maker: AccountId::new_unchecked(String::from("maker.near")),
            sell_token: AccountId::new_unchecked(String::from("token2.near")),
            sell_amount: U128(20),
            buy_token: AccountId::new_unchecked(String::from("token1.near")),
            buy_amount: U128(300),
        };

        assert_eq!(order.get_id(), order1.get_id());

        let order2 = Order {
            maker: AccountId::new_unchecked(String::from("maker.near")),
            sell_token: AccountId::new_unchecked(String::from("token2.near")),
            sell_amount: U128(1000), // param changed
            buy_token: AccountId::new_unchecked(String::from("token1.near")),
            buy_amount: U128(300),
        };

        assert_ne!(order.get_id(), order2.get_id());
    }

    #[test]
    fn test_add_order() {
        let mut contract = Market {
            orders: UnorderedMap::new(StorageKey::Orders),
            version: 1,
        };
        let mut builder = VMContextBuilder::new();
        testing_env!(builder
            .storage_usage(env::storage_usage())
            .attached_deposit(0)
            .predecessor_account_id(AccountId::new_unchecked(String::from("maker.near")))
            .build());

        let new_order_action = NewOrderAction {
            sell_token: AccountId::new_unchecked(String::from("token2.near")),
            sell_amount: U128(20),
            buy_token: AccountId::new_unchecked(String::from("token1.near")),
            buy_amount: U128(300),
        };
       
        contract.add_order(new_order_action.clone(), AccountId::new_unchecked(String::from("maker.near")));

        // check get pairs 
        assert!(contract.get_pairs().len() != 0);

        // check get orders 
        let orders = contract.get_orders(new_order_action.sell_token.clone(), new_order_action.buy_token.clone()).unwrap();
        assert!(orders.len() != 0);

        // check remove order
        let order_id = orders.get(0).unwrap().get_id();
        contract.remove_order(new_order_action.sell_token.clone(), new_order_action.buy_token.clone(), order_id);
        
        assert!(contract.get_pairs().len() == 0);
    }
}

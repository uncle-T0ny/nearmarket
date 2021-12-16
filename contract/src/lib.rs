use crate::ext_interfaces::*;
use crate::types::*;
use errors::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::near_bindgen;
use near_sdk::serde_json;
use near_sdk::serde_json::Error;
use near_sdk::BorshStorageKey;
use near_sdk::Gas;
use near_sdk::PanicOnDefault;
use near_sdk::{env, AccountId, PromiseOrValue};

mod errors;
mod ext_interfaces;
mod types;

pub const ONE_YOCTO: u128 = 1;
pub const FT_TRANSFER_TGAS: Gas = Gas(50_000_000_000_000);

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    OrdersById,
    MapByOrderId,
    Orders,
    OrderIdToOrder,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
pub struct Market {
    version: u8,
    orders: UnorderedMap<String, TreeMap<u64, Order>>,
    order_id_to_order: LookupMap<u64, Order>,
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
        if msg.is_empty() {
            return PromiseOrValue::Value(amount);
        } else {
            let message =
                serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR07_WRONG_MSG_FORMAT);
            match message {
                TokenReceiverMessage::NewOrderAction {
                    sell_token,
                    sell_amount,
                    buy_token,
                    buy_amount,
                } => {
                    env::log_str("its new_order_action");

                    let new_order_action = NewOrderAction {
                        sell_token,
                        sell_amount,
                        buy_token,
                        buy_amount,
                    };

                    self.add_order(new_order_action, sender_id);
                    return PromiseOrValue::Value(U128(0));
                }
                TokenReceiverMessage::Match { order_id } => {
                    env::log_str("its order match ");

                    self.match_order(sender_id, order_id.0, amount, token);
                    return PromiseOrValue::Value(U128(0));
                }
            }
        }
    }
}

#[near_bindgen]
impl Market {
    #[init]
    pub fn new(version: u8) -> Self {
        let this = Self {
            version,
            orders: UnorderedMap::new(StorageKey::Orders),
            order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
        };
        this
    }

    fn match_order(&mut self, sender_id: AccountId, order_id: u64, amount: U128, token: AccountId) {
        env::log_str(&format!("match_order: {}, {:?}, {}", order_id, amount, token));
        let existed_order = self.order_id_to_order.get(&order_id);
        if existed_order.is_none() {
            env::panic_str(ERR03_ORDER_NOT_FOUND);
        }
        let existed_order = existed_order.unwrap();
       
        if amount != existed_order.buy_amount {
            env::panic_str(ERR05_NOT_VALID_AMOUNT);
        }

        if token != existed_order.buy_token {
            env::panic_str(ERR06_NOT_VALID_TOKEN);
        }

        // todo: check storage deposit
        // complete order
        ft_token::ft_transfer( // todo add callback and check that tokens transfered before removing 
            sender_id,
            existed_order.sell_amount,
            "".to_string(),
            existed_order.sell_token.clone(),
            ONE_YOCTO,
            FT_TRANSFER_TGAS,
        );

        ft_token::ft_transfer( // todo add callback and check that tokens transfered before removing 
            existed_order.maker,
            existed_order.buy_amount,
            "".to_string(),
            existed_order.buy_token.clone(),
            ONE_YOCTO,
            FT_TRANSFER_TGAS,
        );

        self.remove_order(existed_order.sell_token, existed_order.buy_token, order_id);
    }

    fn add_order(&mut self, action: NewOrderAction, sender: AccountId) {
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

        self.order_id_to_order.insert(&order_id, &new_order);
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

        // let maker = order.unwrap().maker; // create internal remove 
        // if maker != env::predecessor_account_id() {
        //     env::panic_str(ERR04_PERMISSION_DENIED)
        // }

        orders_map.remove(&order_id);

        if orders_map.len() == 0 {
            self.orders.remove(&key);
        } else {
            self.orders.insert(&key, &orders_map);
        }

        self.order_id_to_order.remove(&order_id);
    }

    pub fn get_order(&self, order_id: U64) -> Option<Order> {
        self.order_id_to_order.get(&order_id.0)    
    }

    pub fn get_orders(
        &self,
        sell_token: AccountId,
        buy_token: AccountId,
    ) -> Option<Vec<OrderView>> {
        let key = compose_key(&sell_token, &buy_token);
        let order_by_key = self.orders.get(&key);

        if order_by_key.is_none() {
            return None;
        }

        let mut res = vec![];

        let orders = order_by_key.unwrap();
        let order_iter = orders.iter().take(5);
        for order in order_iter {
            env::log_str(&format!("order: {:?}", order.1));
            env::log_str(&format!("order id: {}", order.0));
            res.push(OrderView {
                order: order.1.clone(),
                order_id: U64(order.0),
            })
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
    use near_sdk::{test_utils::VMContextBuilder, testing_env};

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
            order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
            version: 1,
        };
        let mut builder = VMContextBuilder::new();
        testing_env!(builder
            .storage_usage(env::storage_usage())
            .attached_deposit(0)
            .predecessor_account_id(AccountId::new_unchecked(String::from("aromankov.testnet")))
            .build());

        let new_order_action = NewOrderAction {
            sell_token: AccountId::new_unchecked(String::from("xabr.allbridge.testnet")),
            sell_amount: U128(1000000000000000000000000),
            buy_token: AccountId::new_unchecked(String::from("abr.allbridge.testnet")),
            buy_amount: U128(1000000000000000000000000),
        };

        contract.add_order(
            new_order_action.clone(),
            AccountId::new_unchecked(String::from("aromankov.testnet")),
        );

        // check get pairs
        assert!(contract.get_pairs().len() != 0);

        // check get orders
        let orders = contract
            .get_orders(
                new_order_action.sell_token.clone(),
                new_order_action.buy_token.clone(),
            )
            .unwrap();
        assert!(orders.len() != 0);

        // check remove order
        let order_id = orders.get(0).unwrap().order_id.0;

        println!("order id: {}", order_id);
        assert!(contract.get_order(U64(order_id)).is_some());

        contract.remove_order(
            new_order_action.sell_token.clone(),
            new_order_action.buy_token.clone(),
            order_id,
        );

        assert!(contract.get_pairs().len() == 0);
        assert!(contract.get_order(U64(order_id)).is_none());
    }
}

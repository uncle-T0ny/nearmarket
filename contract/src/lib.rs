mod market;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}

// use crate::helpers::*;
// use crate::types::*;
// use errors::*;
//
// use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
// use near_sdk::borsh;
// use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
// use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap};
// use near_sdk::json_types::U128;
// use near_sdk::json_types::U64;
// use near_sdk::near_bindgen;
// use near_sdk::serde_json;
// use near_sdk::BorshStorageKey;
// use near_sdk::Gas;
// use near_sdk::PanicOnDefault;
// use near_sdk::PromiseResult;
// use near_sdk::{env, AccountId, PromiseOrValue};
//
// mod errors;
// mod helpers;
// mod market;
// mod orders;
// mod types;
//
// pub const ONE_YOCTO: u128 = 1;
// pub const HUNDRED_PERCENT: u16 = 10000;
// pub const FT_TRANSFER_TGAS: Gas = Gas(50_000_000_000_000);
// pub const RESERVE_TGAS: Gas = Gas(15_000_000_000_000);
//
// #[derive(BorshSerialize, BorshStorageKey)]
// pub enum StorageKey {
//     OrdersById,
//     MapByOrderId,
//     Orders,
//     OrderIdToOrder,
//     FeesByAccountIds,
// }
//
// #[near_bindgen]
// #[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
// pub struct Market {
//     version: u8,
//     orders: UnorderedMap<String, TreeMap<OrderId, Order>>,
//     order_id_to_order: LookupMap<OrderId, Order>,
//     fees: LookupMap<AccountId, Fee>,
// }
//
// #[near_bindgen]
// impl FungibleTokenReceiver for Market {
//     fn ft_on_transfer(
//         &mut self,
//         sender_id: AccountId,
//         amount: U128,
//         msg: String,
//     ) -> PromiseOrValue<U128> {
//         let token = env::predecessor_account_id();
//         env::log_str(&format!(
//             "Transfered {:?} {} from {}",
//             amount, token, sender_id
//         ));
//         env::log_str(&format!("transfer msg: {}", msg));
//         if msg.is_empty() {
//             return PromiseOrValue::Value(amount);
//         } else {
//             let message =
//                 serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR07_WRONG_MSG_FORMAT);
//             match message {
//                 TokenReceiverMessage::NewOrderAction {
//                     sell_token,
//                     sell_amount,
//                     buy_token,
//                     buy_amount,
//                 } => {
//                     env::log_str("its new_order_action");
//
//                     let new_order_action = NewOrderAction {
//                         sell_token,
//                         sell_amount,
//                         buy_token,
//                         buy_amount,
//                     };
//
//                     self.add_order(new_order_action, sender_id);
//                     return PromiseOrValue::Value(U128(0));
//                 }
//                 TokenReceiverMessage::Match { order_id } => {
//                     env::log_str("its order match ");
//
//                     self.match_order(sender_id, order_id, amount, token);
//                     return PromiseOrValue::Value(U128(0));
//                 }
//             }
//         }
//     }
// }
//
// #[near_bindgen]
// impl Market {
//     #[init]
//     pub fn new(version: u8) -> Self {
//         let this = Self {
//             version,
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//         this
//     }
//
//     fn match_order(
//         &mut self,
//         sender_id: AccountId,
//         order_id: OrderId,
//         amount: U128,
//         token: AccountId,
//     ) {
//         env::log_str(&format!(
//             "match_order: {}, {:?}, {}",
//             order_id, amount, token
//         ));
//         let existed_order = self.order_id_to_order.get(&order_id);
//         if existed_order.is_none() {
//             env::panic_str(ERR03_ORDER_NOT_FOUND);
//         }
//
//         let order = existed_order.unwrap();
//
//         if amount != order.buy_amount {
//             env::panic_str(ERR05_NOT_VALID_AMOUNT);
//         }
//
//         if token != order.buy_token {
//             env::panic_str(ERR06_NOT_VALID_TOKEN);
//         }
//
//         let gas_for_next_callback =
//             env::prepaid_gas() - env::used_gas() - FT_TRANSFER_TGAS - RESERVE_TGAS;
//
//         ft_token::ft_transfer(
//             order.maker,
//             order.buy_amount,
//             "".to_string(),
//             order.buy_token.clone(),
//             ONE_YOCTO,
//             FT_TRANSFER_TGAS,
//         )
//         .then(ext_self::callback_on_send_tokens_to_maker(
//             sender_id, // matcher
//             order.sell_amount,
//             order.sell_token,
//             order.buy_token,
//             order_id,
//             env::current_account_id(),
//             0,
//             gas_for_next_callback,
//         ));
//     }
//
//     fn get_or_create_fee_info(&mut self, sell_token: &AccountId) -> Fee {
//         match self.fees.get(sell_token) {
//             Some(fee) => fee,
//             None => {
//                 let fee = Fee {
//                     // 1 / 100 = 0.01%
//                     // 100% = HUNDRED_PERCENT = 10000
//                     percent: 100,
//                     earned: 0,
//                 };
//
//                 self.fees.insert(sell_token, &fee);
//                 fee
//             }
//         }
//     }
//
//     fn take_fee(&mut self, amount: u128, sell_token: &AccountId) -> u128 {
//         let fee_value = self.get_or_create_fee_info(sell_token).percent;
//         let fee = amount * ((HUNDRED_PERCENT - fee_value) as u128) / (HUNDRED_PERCENT as u128);
//         fee
//     }
//
//     pub fn set_fee(&mut self, token: AccountId, percent: u16) {
//         assert_owner();
//         assert!(percent <= HUNDRED_PERCENT);
//         assert!(percent >= 1);
//
//         let earned = match self.fees.get(&token) {
//             Some(v) => v.earned,
//             None => 0,
//         };
//
//         self.fees.insert(&token, &Fee::new(percent, earned));
//     }
//
//     pub fn transfer_earned_fees(&mut self, token: AccountId, amount: u128, receiver: AccountId) {
//         assert_owner();
//
//         let fee_info = self.fees.get(&token).expect(ERR10_NOT_ENOUGH);
//
//         if fee_info.earned == 0 {
//             env::panic_str("no need to transfer zero amount");
//         }
//
//         if amount > fee_info.earned {
//             env::panic_str(ERR10_NOT_ENOUGH);
//         }
//
//         let gas_for_next_callback =
//             env::prepaid_gas() - env::used_gas() - FT_TRANSFER_TGAS - RESERVE_TGAS;
//
//         ft_token::ft_transfer(
//             receiver.clone(),
//             U128(amount),
//             "transfer from contract".to_string(),
//             token.clone(),
//             ONE_YOCTO,
//             FT_TRANSFER_TGAS,
//         )
//         .then(ext_self::callback_on_send_tokens_to_ext_account(
//             token,
//             receiver,
//             U128(amount),
//             env::current_account_id(),
//             0,
//             gas_for_next_callback,
//         ));
//     }
//
//     #[private]
//     fn callback_on_send_tokens_to_ext_account(
//         &mut self,
//         token: AccountId,
//         receiver: AccountId,
//         amount: U128,
//     ) {
//         assert_eq!(
//             env::promise_results_count(),
//             1,
//             "{}",
//             ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT
//         );
//
//         match env::promise_result(0) {
//             PromiseResult::Failed => env::log_str("failed to transfer tokens to receiver"),
//             PromiseResult::Successful(_) => {
//                 env::log_str("tokens successfully transferred to receiver");
//
//                 let mut fee_info = match self.fees.get(&token) {
//                     Some(v) => v,
//                     None => env::panic_str("failed to get fee info for token"),
//                 };
//
//                 fee_info.earned = fee_info.earned.saturating_sub(amount.0);
//                 self.fees.insert(&token, &fee_info);
//             }
//             _ => unreachable!(),
//         }
//     }
//
//     #[private]
//     pub fn callback_on_send_tokens_to_maker(
//         &mut self,
//         sender_id: AccountId,
//         sell_amount: U128,
//         sell_token: AccountId,
//         buy_token: AccountId,
//         order_id: OrderId,
//     ) {
//         assert_eq!(
//             env::promise_results_count(),
//             1,
//             "{}",
//             ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT
//         );
//         let is_promise_success: bool = match env::promise_result(0) {
//             PromiseResult::NotReady => unreachable!(),
//             PromiseResult::Successful(_) => true,
//             PromiseResult::Failed => false,
//         };
//
//         if is_promise_success {
//             let gas_for_next_callback =
//                 env::prepaid_gas() - env::used_gas() - FT_TRANSFER_TGAS - RESERVE_TGAS;
//
//             let fee = self.take_fee(sell_amount.0, &sell_token);
//
//             // check storage deposit
//             ft_token::ft_transfer(
//                 sender_id,
//                 U128(fee),
//                 "".to_string(),
//                 sell_token.clone(),
//                 ONE_YOCTO,
//                 FT_TRANSFER_TGAS,
//             )
//             .then(ext_self::callback_after_deposit(
//                 U128(fee),
//                 sell_token,
//                 buy_token,
//                 order_id,
//                 env::current_account_id(),
//                 0,
//                 gas_for_next_callback,
//             ));
//         } else {
//             // for example maker did not registred buy_token
//             env::panic_str(ERR01_INTERNAL);
//         }
//     }
//
//     #[private]
//     pub fn callback_after_deposit(
//         &mut self,
//         fee: U128,
//         sell_token: AccountId,
//         buy_token: AccountId,
//         order_id: OrderId,
//     ) {
//         assert_eq!(
//             env::promise_results_count(),
//             1,
//             "{}",
//             ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT
//         );
//
//         if let PromiseResult::Failed = env::promise_result(0) {
//             env::log_str("failed to transfer token to sender")
//         } else {
//             env::log_str("transfer token to sender completed successfully");
//             let mut fee_info = self.get_or_create_fee_info(&sell_token);
//             fee_info.earned += fee.0;
//
//             self.fees.insert(&sell_token, &fee_info);
//         }
//
//         let key = compose_key(&sell_token, &buy_token);
//         let orders_map = self
//             .orders
//             .get(&key)
//             .unwrap_or_else(|| env::panic_str(ERR01_INTERNAL));
//         self.internal_remove_order(&key, orders_map, order_id);
//     }
//
//     fn add_order(&mut self, action: NewOrderAction, sender: AccountId) {
//         let new_order = Order::from_action(action, sender);
//
//         let key = compose_key(&new_order.sell_token, &new_order.buy_token);
//         let mut orders_map = self
//             .orders
//             .get(&key)
//             .unwrap_or(TreeMap::new(key.as_bytes()));
//
//         let order_id = new_order.get_id();
//         if orders_map.contains_key(&order_id) {
//             env::panic_str(ERR02_ORDER_ALREADY_EXISTS);
//         }
//
//         orders_map.insert(&OrderId::from_order(&new_order), &new_order);
//
//         self.order_id_to_order.insert(&order_id, &new_order);
//         self.orders.insert(&key, &orders_map);
//     }
//
//     pub fn remove_order(&mut self, sell_token: AccountId, buy_token: AccountId, order_id: OrderId) {
//         let key = compose_key(&sell_token, &buy_token);
//         let order_by_key = self.orders.get(&key);
//
//         if order_by_key.is_none() {
//             env::panic_str(ERR03_ORDER_NOT_FOUND);
//         }
//
//         let orders_map = order_by_key.unwrap();
//         let order = orders_map.get(&order_id);
//
//         if order.is_none() {
//             env::panic_str(ERR03_ORDER_NOT_FOUND);
//         }
//
//         let order = order.unwrap();
//         let maker = order.maker;
//         if maker != env::predecessor_account_id() {
//             env::panic_str(ERR04_PERMISSION_DENIED)
//         }
//
//         self.internal_remove_order(&key, orders_map, order_id);
//
//         ft_token::ft_transfer(
//             maker,
//             order.sell_amount,
//             "".to_string(),
//             order.sell_token.clone(),
//             ONE_YOCTO,
//             FT_TRANSFER_TGAS,
//         );
//     }
//
//     fn internal_remove_order(
//         &mut self,
//         key: &String,
//         mut orders_map: TreeMap<OrderId, Order>,
//         order_id: OrderId,
//     ) {
//         orders_map.remove(&order_id);
//
//         if orders_map.len() == 0 {
//             self.orders.remove(key);
//         } else {
//             self.orders.insert(key, &orders_map);
//         }
//
//         self.order_id_to_order.remove(&order_id);
//     }
//
//     pub fn get_order(&self, order_id: OrderId) -> Option<Order> {
//         self.order_id_to_order.get(&order_id)
//     }
//
//     pub fn get_orders(
//         &self,
//         sell_token: AccountId,
//         buy_token: AccountId,
//     ) -> Option<Vec<OrderView>> {
//         let key = compose_key(&sell_token, &buy_token);
//         let order_by_key = self.orders.get(&key);
//
//         if order_by_key.is_none() {
//             return None;
//         }
//
//         let mut res = vec![];
//
//         let orders = order_by_key.unwrap();
//         // println!("orders.contains_key: {}", orders.contains_key(&6459152053938679878));
//         let order_iter = orders.iter().take(5);
//         for order in order_iter {
//             res.push(OrderView {
//                 order: order.1.clone(),
//                 order_id: order.0.clone(),
//             })
//         }
//
//         return Some(res);
//     }
//
//     pub fn get_pairs(&self) -> Vec<String> {
//         let keys = self.orders.keys_as_vector();
//         keys.to_vec()
//     }
// }
//
// fn compose_key(sell_token: &AccountId, buy_token: &AccountId) -> String {
//     let mut key = String::from(sell_token.as_str());
//     key.push_str("#");
//     key.push_str(buy_token.as_str());
//     key
// }
//
// #[cfg(test)]
// mod tests {
//     use near_sdk::{test_utils::VMContextBuilder, testing_env};
//
//     use super::*;
//
//     fn create_test_order(sell_amount: u128, buy_amount: u128) -> Order {
//         Order {
//             maker: AccountId::new_unchecked(String::from("maker.near")),
//             sell_token: AccountId::new_unchecked(String::from("xabr.allbridge.testnet")),
//             sell_amount: U128(sell_amount),
//             buy_token: AccountId::new_unchecked(String::from("abr.allbridge.testnet")),
//             buy_amount: U128(buy_amount),
//         }
//     }
//
//     fn fee_test(contract: &mut Market, token: &str, amount: u128, expect: u128) {
//         assert_eq!(contract.take_fee(amount, &(token.parse().unwrap())), expect);
//     }
//
//     #[test]
//     #[should_panic]
//     fn test_fee_overflow() {
//         let mut contract = Market {
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             version: 1,
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//
//         contract.set_fee("sometoken.near".parse().unwrap(), HUNDRED_PERCENT + 1)
//     }
//
//     #[test]
//     #[should_panic]
//     fn test_fee_too_low() {
//         let mut contract = Market {
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             version: 1,
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//
//         contract.set_fee("sometoken.near".parse().unwrap(), 0)
//     }
//
//     #[test]
//     fn test_fee() {
//         let mut contract = Market {
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             version: 1,
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//
//         let mut builder = VMContextBuilder::new();
//         testing_env!(builder
//             .storage_usage(env::storage_usage())
//             .attached_deposit(0)
//             .predecessor_account_id(AccountId::new_unchecked(String::from("aromankov.testnet")))
//             .current_account_id(AccountId::new_unchecked(String::from("aromankov.testnet")))
//             .build());
//
//         fee_test(&mut contract, "sometoken.near", 100, 99);
//         fee_test(&mut contract, "sometoken.near", 23200, 22968);
//         fee_test(&mut contract, "sometoken.near", 1111111, 1099999);
//         fee_test(
//             &mut contract,
//             "sometoken.near",
//             1000000000000000000000000000,
//             990000000000000000000000000,
//         );
//
//         contract.set_fee("sometoken2.near".parse().unwrap(), 500);
//
//         fee_test(&mut contract, "sometoken2.near", 100, 95);
//         fee_test(&mut contract, "sometoken2.near", 23200, 22040);
//         fee_test(&mut contract, "sometoken2.near", 1111111, 1055555);
//         fee_test(
//             &mut contract,
//             "sometoken2.near",
//             1000000000000000000000000000,
//             950000000000000000000000000,
//         );
//     }
//
//     #[test]
//     #[should_panic]
//     fn test_fee_wrong_permissions() {
//         let mut contract = Market {
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             version: 1,
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//
//         let mut builder = VMContextBuilder::new();
//         testing_env!(builder
//             .storage_usage(env::storage_usage())
//             .attached_deposit(0)
//             .predecessor_account_id(AccountId::new_unchecked(String::from("aromankov.testnet")))
//             .build());
//         contract.set_fee("sometoken2.near".parse().unwrap(), 500);
//     }
//
//     #[test]
//     fn test_order_hash() {
//         let order = create_test_order(1000000000000000000000000, 1000000000000000000000000);
//         let order1 = create_test_order(1000000000000000000000000, 1000000000000000000000000);
//
//         assert_eq!(order.get_id(), order1.get_id());
//
//         let order2 = Order {
//             maker: AccountId::new_unchecked(String::from("maker.near")),
//             sell_token: AccountId::new_unchecked(String::from("abr.allbridge.testnet")),
//             sell_amount: U128(1000000000000000000000000), // param changed
//             buy_token: AccountId::new_unchecked(String::from("xbr.allbridge.testnet")),
//             buy_amount: U128(1000000000000000000000000),
//         };
//
//         assert_ne!(order.get_id(), order2.get_id());
//     }
//
//     #[test]
//     fn test_add_order() {
//         let mut contract = Market {
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             version: 1,
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//
//         let mut builder = VMContextBuilder::new();
//         testing_env!(builder
//             .storage_usage(env::storage_usage())
//             .attached_deposit(0)
//             .predecessor_account_id(AccountId::new_unchecked(String::from("aromankov.testnet")))
//             .build());
//
//         let new_order_action_1 = NewOrderAction {
//             sell_token: AccountId::new_unchecked(String::from("xabr.allbridge.testnet")),
//             sell_amount: U128(1000000000000000000000000),
//             buy_token: AccountId::new_unchecked(String::from("abr.allbridge.testnet")),
//             buy_amount: U128(1000000000000000000000000),
//         };
//
//         contract.add_order(
//             new_order_action_1.clone(),
//             AccountId::new_unchecked(String::from("aromankov.testnet")),
//         );
//
//         let new_order_action_2 = NewOrderAction {
//             sell_token: AccountId::new_unchecked(String::from("abr.allbridge.testnet")),
//             sell_amount: U128(1000000000000000000000000),
//             buy_token: AccountId::new_unchecked(String::from("xabr.allbridge.testnet")),
//             buy_amount: U128(1000000000000000000000000),
//         };
//
//         contract.add_order(
//             new_order_action_2.clone(),
//             AccountId::new_unchecked(String::from("aromankov.testnet")),
//         );
//
//         // check get pairs
//         assert!(contract.get_pairs().len() != 0);
//
//         // check get orders
//         let orders_1 = contract
//             .get_orders(
//                 new_order_action_1.sell_token.clone(),
//                 new_order_action_1.buy_token.clone(),
//             )
//             .unwrap();
//         assert!(orders_1.len() == 1);
//
//         let orders_2 = contract
//             .get_orders(
//                 new_order_action_2.sell_token.clone(),
//                 new_order_action_2.buy_token.clone(),
//             )
//             .unwrap();
//         assert!(orders_2.len() == 1);
//
//         let order_2 = orders_2.get(0).unwrap();
//         let order_id_2 = order_2.order_id;
//         assert_eq!(
//             *order_2,
//             OrderView {
//                 order: Order {
//                     buy_amount: new_order_action_2.buy_amount.clone(),
//                     sell_amount: new_order_action_2.sell_amount.clone(),
//                     buy_token: new_order_action_2.buy_token.clone(),
//                     sell_token: new_order_action_2.sell_token.clone(),
//                     maker: AccountId::new_unchecked(String::from("aromankov.testnet"))
//                 },
//                 order_id: order_id_2
//             }
//         );
//
//         let order_1 = orders_1.get(0).unwrap();
//         let order_id_1 = order_1.order_id;
//
//         assert_eq!(
//             *order_1,
//             OrderView {
//                 order: Order {
//                     buy_amount: new_order_action_1.buy_amount.clone(),
//                     sell_amount: new_order_action_1.sell_amount.clone(),
//                     buy_token: new_order_action_1.buy_token.clone(),
//                     sell_token: new_order_action_1.sell_token.clone(),
//                     maker: AccountId::new_unchecked(String::from("aromankov.testnet"))
//                 },
//                 order_id: order_id_1
//             }
//         );
//
//         // check remove order
//         assert!(contract.get_order(order_id_1).is_some());
//
//         contract.remove_order(
//             new_order_action_1.sell_token.clone(),
//             new_order_action_1.buy_token.clone(),
//             order_id_1,
//         );
//
//         contract.remove_order(
//             new_order_action_2.sell_token.clone(),
//             new_order_action_2.buy_token.clone(),
//             order_id_2,
//         );
//
//         assert!(contract.get_pairs().len() == 0);
//         assert!(contract.get_order(order_id_1).is_none());
//     }
//
//     #[test]
//     fn test_orders_ordered_by_price() {
//         let mut contract = Market {
//             orders: UnorderedMap::new(StorageKey::Orders),
//             order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
//             version: 1,
//             fees: LookupMap::new(StorageKey::FeesByAccountIds),
//         };
//         let mut builder = VMContextBuilder::new();
//         testing_env!(builder
//             .storage_usage(env::storage_usage())
//             .attached_deposit(0)
//             .predecessor_account_id(AccountId::new_unchecked(String::from("aromankov.testnet")))
//             .build());
//
//         let account1: AccountId = "xabr.allbridge.testnet".parse().unwrap();
//         let account2: AccountId = "abr.allbridge.testnet".parse().unwrap();
//
//         let new_order_action_1 = NewOrderAction {
//             sell_token: account1.clone(),
//             sell_amount: U128(2),
//             buy_token: account2.clone(),
//             buy_amount: U128(1),
//         };
//
//         contract.add_order(
//             new_order_action_1.clone(),
//             AccountId::new_unchecked(String::from("aromankov.testnet")),
//         );
//
//         let new_order_action_2 = NewOrderAction {
//             sell_token: account1.clone(),
//             sell_amount: U128(1),
//             buy_token: account2.clone(),
//             buy_amount: U128(1),
//         };
//
//         contract.add_order(
//             new_order_action_2.clone(),
//             AccountId::new_unchecked(String::from("aromankov.testnet")),
//         );
//
//         let new_order_action_2 = NewOrderAction {
//             sell_token: account1.clone(),
//             sell_amount: U128(1),
//             buy_token: account2.clone(),
//             buy_amount: U128(2),
//         };
//
//         let new_order_action_3 = NewOrderAction {
//             sell_token: account1.clone(),
//             sell_amount: U128(1),
//             buy_token: account2.clone(),
//             buy_amount: U128(1),
//         };
//
//         contract.add_order(
//             new_order_action_2.clone(),
//             AccountId::new_unchecked(String::from("aromankov.testnet")),
//         );
//
//         let orders = contract
//             .get_orders(account1, account2)
//             .unwrap()
//             .into_iter()
//             .map(|i| i.order.get_price_for_key())
//             .collect::<Vec<_>>();
//
//         assert!(orders[0] < orders[1]);
//         assert!(orders[1] < orders[2]);
//     }
// }

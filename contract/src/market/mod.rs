use crate::market::constants::*;
use crate::market::ext_interface::ext_self;
use crate::market::fees::Fee;
use crate::market::helpers::assert_owner;
use crate::market::helpers::{ft_transfer, get_next_gas};
use crate::market::order::{NewOrderAction, Order};
use crate::market::order_id::OrderId;
use crate::market::storage::StorageKey;
use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap};
use near_sdk::{borsh, borsh::*, *};

mod callbacks;
mod constants;
mod ext_interface;
mod fees;
mod ft_receiver_market;
mod helpers;
mod order;
mod order_id;
mod storage;
mod view;

#[near_bindgen]
#[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
pub struct Market {
    version: u8,
    orders: UnorderedMap<String, TreeMap<OrderId, Order>>,
    order_id_to_order: LookupMap<OrderId, Order>,
    fees: LookupMap<AccountId, Fee>,
}

#[near_bindgen]
impl Market {
    #[init]
    #[payable]
    pub fn new(version: u8) -> Self {
        let this = Self {
            version,
            orders: UnorderedMap::new(StorageKey::Orders),
            order_id_to_order: LookupMap::new(StorageKey::OrderIdToOrder),
            fees: LookupMap::new(StorageKey::FeesByAccountIds),
        };
        this
    }

    pub fn set_fee(&mut self, token: AccountId, percent: u16) {
        assert_owner();
        self.internal_set_fee(token, percent)
    }

    pub fn remove_order(&mut self, sell_token: AccountId, buy_token: AccountId, order_id: OrderId) -> Promise {
        let key = helpers::compose_key(&sell_token, &buy_token);
        let order_by_key = self.orders.get(&key);

        let mut orders_map = order_by_key.expect(ERR03_ORDER_NOT_FOUND);
        let order = orders_map.get(&order_id).expect(ERR03_ORDER_NOT_FOUND);
        let maker = order.maker;

        require!(
            env::predecessor_account_id() == maker,
            ERR04_PERMISSION_DENIED
        );

        self.remove_order_from_tree(&key, &order_id, &mut orders_map);
        
        helpers::ft_transfer(&maker, order.sell_amount.into(), "".to_string(), &order.sell_token)
    }
}

impl Market {
    // todo: subject for optimisation. Too many mem-copies for containers
    pub(crate) fn add_order(&mut self, action: NewOrderAction, sender: AccountId) {
        let order = Order {
            maker: sender,
            sell_token: action.sell_token,
            sell_amount: action.sell_amount,
            buy_token: action.buy_token,
            buy_amount: action.buy_amount,
        };

        let key = helpers::compose_key(&order.sell_token, &order.buy_token);
        let mut orders_map = self
            .orders
            .get(&key)
            .unwrap_or(TreeMap::new(key.as_bytes()));

        let order_id = order.get_id();
        require!(
            !orders_map.contains_key(&order_id),
            ERR02_ORDER_ALREADY_EXISTS
        );

        orders_map.insert(&order_id, &order);

        self.order_id_to_order.insert(&order_id, &order);
        self.orders.insert(&key, &orders_map);
    }

    pub(crate) fn match_order(
        &mut self,
        sender_id: &AccountId,
        order_id: &OrderId,
        amount: u128,
        token: &AccountId,
    ) -> Promise {
        log!("match order: {}, {:?}. {}", order_id, amount, token);
        let order = self
            .order_id_to_order
            .get(order_id)
            .expect(ERR03_ORDER_NOT_FOUND);

        require!(order.buy_amount.0 == amount, ERR05_NOT_VALID_AMOUNT);
        require!(order.buy_token == *token, ERR06_NOT_VALID_TOKEN);

        helpers::ft_transfer(
            &order.maker,
            order.buy_amount.0,
            "".to_string(),
            &order.buy_token,
        )
        .then(ext_self::callback_on_send_tokens_to_maker(
            sender_id.clone(),
            order.sell_amount,
            order.sell_token,
            order.buy_token,
            order_id.clone(),
            env::current_account_id(),
            0,
            get_next_gas(),
        ))
    }

    pub(crate) fn internal_on_tokens_sent_to_ext_account(
        &mut self,
        token: &AccountId,
        amount: u128,
    ) {
        log!("tokens successfully transferred to receiver");

        let mut fee_info = self
            .fees
            .get(&token)
            .expect("failed to get fee info for token");

        fee_info.earned = fee_info.earned.saturating_sub(amount);

        self.fees.insert(&token, &fee_info);
    }

    pub(crate) fn on_success_deposit(&mut self, fee: u128, sell_token: &AccountId) {
        let mut fee_info = self.get_or_create_fee_info(&sell_token);
        fee_info.earned += fee;
        self.fees.insert(&sell_token, &fee_info);
    }

    // todo: subject for optimisation
    //       removing order will cost too much if orders_map will be large
    pub(crate) fn internal_remove_order(
        &mut self,
        sell_token: &AccountId,
        buy_token: &AccountId,
        order_id: &OrderId,
    ) {
        let key = helpers::compose_key(&sell_token, &buy_token);
        let mut orders_map = self
            .orders
            .get(&key)
            .unwrap_or_else(|| env::panic_str(ERR01_INTERNAL));

        self.remove_order_from_tree(&key, &order_id, &mut orders_map);
    }

    pub(crate) fn remove_order_from_tree(
        &mut self,
        key: &String,
        order_id: &OrderId,
        orders_map: &mut TreeMap<OrderId, Order>,
    ) {
        orders_map.remove(&order_id);

        if orders_map.len() == 0 {
            self.orders.remove(key);
        } else {
            self.orders.insert(key, &orders_map);
        }

        self.order_id_to_order.remove(&order_id);
    }
}

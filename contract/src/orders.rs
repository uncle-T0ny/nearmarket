use crate::market::NearMarket;
use crate::OrderId;
use near_sdk::*;

#[near_bindgen]
impl NearMarket {
    pub(crate) fn match_order(
        &mut self,
        sender_id: AccountId,
        order_id: OrderId,
        amount: u128,
        token: AccountId,
    ) -> Promise {
        log!("match_order: {}, {:?}, {}", order_id, amount, token);

        let existing_order = self
            .order_id_to_account
            .get(&order_id)
            .expect(ERR03_ORDER_NOT_FOUND);

        let order = self
            .orders
            .get(&existing_order)
            .expect(ERR01_INTERNAL)
            .get(&order_id)
            .expect(ERR01_INTERNAL);

        if token != order.info.buy_token {
            env::panic_str(ERR06_NOT_VALID_TOKEN);
        }

        let remaining_amount = order.info.amount.0 - order.info.filled_amount.0;
        if remaining_amount < amount {
            env::panic_str(ERR05_NOT_VALID_AMOUNT);
        }

        log!("remaining_amount: {}", remaining_amount);

        helpers::ft_transfer(order.maker, amount, "", &order.info.buy_token).then(
            ext_self::callback_on_send_tokens_to_maker(
                sender_id,
                amount.into(),
                order.info.sell_token,
                order.info.buy_token,
                order_id,
                env::current_account_id(),
                0,
                self.get_gas(),
            ),
        )
    }

    fn add_order(&mut self, info: OrderInfo, sender: AccountId) {
        let key = info.get_key().try_to_vec().unwrap();
        let new_order = Order {
            maker: sender.clone(),
            info,
        };

        let mut orders = self.orders.get(&key).unwrap_or(TreeMap::new(key));
    }

    // fn add_order(&mut self, action: NewOrderAction, sender: AccountId) {
    //     let new_order = Order::from_action(action, sender);
    //
    //     let key = compose_key(&new_order.sell_token, &new_order.buy_token);
    //     let mut orders_map = self.orders.get(&key)
    //         .unwrap_or(TreeMap::new(key.as_bytes()));
    //
    //     let order_id = new_order.get_id();
    //     if orders_map.contains_key(&order_id) {
    //         env::panic_str(ERR02_ORDER_ALREADY_EXISTS);
    //     }
    //
    //     orders_map.insert(&OrderId::from_order(&new_order), &new_order);
    //
    //     self.order_id_to_order.insert(&order_id, &new_order);
    //     self.orders.insert(&key, &orders_map);
    // }
}

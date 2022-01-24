use crate::market::helpers;
use crate::market::order::*;
use crate::market::order_id::OrderId;
use crate::market::*;

#[allow(dead_code)]
#[near_bindgen]
impl Market {
    pub fn get_order(&self, order_id: OrderId) -> Option<Order> {
        self.order_id_to_order.get(&order_id)
    }

    pub fn get_orders(
        &self,
        sell_token: AccountId,
        buy_token: AccountId,
    ) -> Option<Vec<OrderView>> {
        let key = helpers::compose_key(&sell_token, &buy_token);
        let orders = self.orders.get(&key)?;

        let mut res = vec![];

        let order_iter = orders.iter().take(5);
        for order in order_iter {
            res.push(OrderView {
                order: order.1.clone(),
                order_id: order.0,
            })
        }

        Some(res)
    }

    pub fn get_pairs(&self) -> Vec<String> {
        let keys = self.orders.keys_as_vector();
        keys.to_vec()
    }
}

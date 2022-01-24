use crate::market::constants::ERR07_WRONG_MSG_FORMAT;
use crate::market::order::{NewOrderAction, Order};
use crate::market::order_id::OrderId;
use crate::market::Market;
use crate::{log, serde_json};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, PromiseOrValue};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiveMessage {
    Match { order_id: OrderId },
    NewOrder(NewOrderAction),
}

impl FungibleTokenReceiver for Market {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id();
        log!("transferred {:?} {} from {}", amount, token, sender_id);
        log!("transfer message: {}", msg);

        if msg.is_empty() {
            return PromiseOrValue::Value(amount);
        }

        let message: TokenReceiveMessage =
            serde_json::from_str(&msg).expect(ERR07_WRONG_MSG_FORMAT);

        match message {
            TokenReceiveMessage::Match { order_id } => self
                .match_order(&sender_id, &order_id, amount.0, &token)
                .into(),
            TokenReceiveMessage::NewOrder(v) => {
                self.add_order(v, sender_id);
                PromiseOrValue::Value(amount)
            }
        }
    }
}

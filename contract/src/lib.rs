use crate::errors::*;
use crate::ext_interfaces::*;
use crate::types::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::PanicOnDefault;
use near_sdk::serde_json::Error;
use near_sdk::{env, AccountId, PromiseOrValue};

mod errors;
mod ext_interfaces;
mod types;

#[near_bindgen]
#[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
pub struct Market {
    version: u8,
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
        };
        this
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_mint_amount() {
       
         assert_eq!(1 + 1, 2);
    }
}

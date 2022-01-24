use crate::market::constants::*;
use crate::market::*;
use near_sdk::json_types::U128;

#[allow(dead_code)]
#[near_bindgen]
impl Market {
    #[private]
    fn callback_on_send_tokens_to_ext_account(&mut self, token: AccountId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT
        );

        match env::promise_result(0) {
            PromiseResult::Failed => log!("failed to transfer tokens to receiver"),
            PromiseResult::Successful(_) => {
                self.internal_on_tokens_sent_to_ext_account(&token, amount.0)
            }
            _ => unreachable!(),
        }
    }

    #[private]
    pub fn callback_on_send_tokens_to_maker(
        &mut self,
        sender_id: AccountId,
        sell_amount: U128,
        sell_token: AccountId,
        buy_token: AccountId,
        order_id: OrderId,
    ) {
        require!(
            env::promise_results_count() == 1,
            ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT
        );

        require!(
            matches!(env::promise_result(0), PromiseResult::Successful(..)),
            ERR01_INTERNAL
        );

        let fee = self.take_fee(sell_amount.0, &sell_token);

        helpers::ft_transfer(&sender_id, fee.into(), "".to_string(), &sell_token).then(
            ext_self::callback_after_deposit(
                fee.into(),
                sell_token,
                buy_token,
                order_id,
                env::current_account_id(),
                0,
                get_next_gas(),
            ),
        );
    }

    #[private]
    pub fn callback_after_deposit(
        &mut self,
        fee: U128,
        sell_token: AccountId,
        buy_token: AccountId,
        order_id: OrderId,
    ) {
        require!(
            env::promise_results_count() == 1,
            ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT
        );

        if let PromiseResult::Failed = env::promise_result(0) {
            env::log_str("failed to transfer token to sender")
        } else {
            env::log_str("transfer token to sender completed successfully");

            self.on_success_deposit(fee.0, &sell_token);
        }

        self.internal_remove_order(&sell_token, &buy_token, &order_id);
    }
}

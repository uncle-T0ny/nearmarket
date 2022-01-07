use near_sdk::{AccountId, ext_contract};
use near_sdk::json_types::{U128, U64};
use crate::OrderId;

#[ext_contract(ft_token)]
pub trait FtToken {
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn ft_total_supply(&self) -> U128;
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn callback_on_send_tokens_to_maker(
        &self,
        sender_id: AccountId,
        sell_amount: U128,
        sell_token: AccountId,
        buy_token: AccountId,
        order_id: OrderId,
    );

    fn callback_after_deposit(
        &self,
        sell_token: AccountId,
        buy_token: AccountId,
        order_id: OrderId
    );
}

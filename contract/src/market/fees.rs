use crate::market::constants::HUNDRED_PERCENT;
use crate::market::*;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::{borsh, *};

#[derive(Copy, Clone, BorshSerialize, BorshDeserialize)]
pub struct Fee {
    pub percent: u16,
    pub earned: u128,
}

impl Fee {
    pub fn new(percent: u16, earned: u128) -> Self {
        Self { percent, earned }
    }
}

impl Market {
    pub(crate) fn get_or_create_fee_info(&mut self, sell_token: &AccountId) -> Fee {
        match self.fees.get(sell_token) {
            Some(fee) => fee,
            None => {
                let fee = Fee {
                    // 1 / 100 = 0.01%
                    // 100% = HUNDRED_PERCENT = 10000
                    percent: 100,
                    earned: 0,
                };

                self.fees.insert(sell_token, &fee);
                fee
            }
        }
    }

    pub(crate) fn take_fee(&mut self, amount: u128, sell_token: &AccountId) -> u128 {
        let fee_value = self.get_or_create_fee_info(sell_token).percent;
        let fee = amount * ((HUNDRED_PERCENT - fee_value) as u128) / (HUNDRED_PERCENT as u128);
        fee
    }

    pub(crate) fn internal_set_fee(&mut self, token: AccountId, percent: u16) {
        require!(percent <= HUNDRED_PERCENT);
        require!(percent >= 1);

        let earned = match self.fees.get(&token) {
            Some(v) => v.earned,
            None => 0,
        };

        self.fees.insert(&token, &Fee::new(percent, earned));
    }
}

#[near_bindgen]
impl Market {
    pub fn transfer_earned_fees(
        &mut self,
        token: AccountId,
        amount: U128,
        receiver: AccountId,
    ) -> Promise {
        assert_owner();

        let fee_info = self.fees.get(&token).expect(ERR10_NOT_ENOUGH);
        require!(fee_info.earned != 0, "no need to transfer zero amount");
        require!(amount.0 <= fee_info.earned, ERR10_NOT_ENOUGH);

        helpers::ft_transfer(
            &receiver,
            amount.into(),
            "transfer from contract".to_string(),
            &token,
        )
        .then(ext_self::callback_on_send_tokens_to_ext_account(
            token,
            receiver,
            amount.into(),
            env::current_account_id(),
            0,
            get_next_gas(),
        ))
    }
}

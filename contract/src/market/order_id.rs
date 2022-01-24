use crate::market::order::Order;
use near_sdk::{borsh, borsh::*, serde::*};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(
    Debug, Ord, PartialEq, Clone, Copy, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderId(pub u128, pub u64);

impl OrderId {
    pub fn from_order(order: &Order) -> Self {
        let mut hasher = DefaultHasher::new();
        order.hash(&mut hasher);

        Self(order.get_price_for_key(), hasher.finish())
    }
}

impl Eq for OrderId {}

impl PartialOrd for OrderId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Display for OrderId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

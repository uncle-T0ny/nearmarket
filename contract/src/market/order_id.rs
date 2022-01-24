use near_sdk::{borsh, borsh::*, serde::*};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(
    Debug, Ord, PartialEq, Clone, Copy, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderId(pub u128, pub u64);

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

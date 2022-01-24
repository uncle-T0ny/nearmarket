use near_sdk::Gas;

pub const ONE_YOCTO: u128 = 1;
pub const HUNDRED_PERCENT: u16 = 10000;
pub const FT_TRANSFER_TGAS: Gas = Gas(50_000_000_000_000);
pub const RESERVE_TGAS: Gas = Gas(15_000_000_000_000);

pub const ERR01_INTERNAL: &str = "E01: internal issue";
pub const ERR02_ORDER_ALREADY_EXISTS: &str = "E02: order already exists";
pub const ERR03_ORDER_NOT_FOUND: &str = "E03: order not found";
pub const ERR04_PERMISSION_DENIED: &str = "E04: permission denied";
pub const ERR05_NOT_VALID_AMOUNT: &str = "E05: not valid amount";
pub const ERR06_NOT_VALID_TOKEN: &str = "E06: not valid token";
pub const ERR07_WRONG_MSG_FORMAT: &str = "E07: wrong msg format";
pub const ERR08_NOT_CORRECT_PROMISE_RESULT_COUNT: &str = "E08: not correct promise result count";
// pub const ERR09_DEPOSIT_FAILED: &str = "E09: deposit failed";
pub const ERR10_NOT_ENOUGH: &str = "E10: not enough FT";

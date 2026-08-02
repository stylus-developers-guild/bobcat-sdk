use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_SUPPLY = b"supply(address,uint256)",
    SEL_WITHDRAW = b"withdraw(address,uint256)"
}

pub const fn make_fn_supply(asset: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_SUPPLY, leftpad_addr(asset), amount.0)
}

pub const fn make_fn_withdraw(asset: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW, leftpad_addr(asset), amount.0)
}

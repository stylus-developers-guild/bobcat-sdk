use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_u16};
use bobcat_maths::U;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_SUPPLY = b"supply(address,uint256,address,uint16)",
    SEL_WITHDRAW = b"withdraw(address,uint256,address)"
}

pub const fn make_fn_supply(
    asset: Address,
    amount: &U,
    on_behalf_of: Address,
    referral_code: u16,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_SUPPLY,
        leftpad_addr(asset),
        amount.0,
        leftpad_addr(on_behalf_of),
        leftpad_u16(referral_code)
    )
}

pub const fn make_fn_withdraw(asset: Address, amount: &U, to: Address) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_WITHDRAW,
        leftpad_addr(asset),
        amount.0,
        leftpad_addr(to)
    )
}

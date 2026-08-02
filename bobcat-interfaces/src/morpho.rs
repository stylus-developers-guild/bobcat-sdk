use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(uint256,address)",
    SEL_WITHDRAW = b"withdraw(uint256,address,address)"
}

pub const fn make_fn_deposit(amount: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, amount.0, leftpad_addr(receiver))
}

pub const fn make_fn_withdraw(
    amount: &U,
    receiver: Address,
    owner: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_WITHDRAW,
        amount.0,
        leftpad_addr(receiver),
        leftpad_addr(owner)
    )
}

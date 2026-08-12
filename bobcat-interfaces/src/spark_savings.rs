use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(uint256,address)",
    SEL_REDEEM = b"redeem(uint256,address,address)"
}

/// Encode `deposit(assets, receiver)` for a Spark Savings vault.
pub const fn make_fn_deposit(assets: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, assets.0, leftpad_addr(receiver))
}

/// Encode `redeem(shares, receiver, owner)` for a Spark Savings vault.
pub const fn make_fn_redeem(shares: &U, receiver: Address, owner: Address) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM,
        shares.0,
        leftpad_addr(receiver),
        leftpad_addr(owner)
    )
}

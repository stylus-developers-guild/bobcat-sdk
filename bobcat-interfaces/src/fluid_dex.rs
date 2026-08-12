//! Narrow Fluid DEX T1 pool interface for core user flows.
//!
//! The builders encode calls to an individual Fluid DEX T1 pool. They cover exact-input
//! swaps and non-perfect collateral liquidity deposit/withdrawal only. Exact-output swaps,
//! callback variants, debt liquidity, perfect-ratio operations, estimates, and administration
//! are intentionally outside this module.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_bool};
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_SWAP_IN = b"swapIn(bool,uint256,uint256,address)",
    SEL_DEPOSIT = b"deposit(uint256,uint256,uint256,bool)",
    SEL_WITHDRAW = b"withdraw(uint256,uint256,uint256,address)",
}

/// Encode `swapIn(swap0to1, amountIn, amountOutMin, to)` for an exact-input swap.
pub const fn make_fn_swap_in(
    swap_0_to_1: bool,
    amount_in: &U,
    amount_out_min: &U,
    to: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_SWAP_IN,
        leftpad_bool(swap_0_to_1),
        amount_in.0,
        amount_out_min.0,
        leftpad_addr(to)
    )
}

/// Encode `deposit(token0Amt, token1Amt, minSharesAmt, false)` for execution.
pub const fn make_fn_deposit(
    token_0_amount: &U,
    token_1_amount: &U,
    min_shares_amount: &U,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT,
        token_0_amount.0,
        token_1_amount.0,
        min_shares_amount.0,
        leftpad_bool(false)
    )
}

/// Encode `withdraw(token0Amt, token1Amt, maxSharesAmt, to)`.
pub const fn make_fn_withdraw(
    token_0_amount: &U,
    token_1_amount: &U,
    max_shares_amount: &U,
    to: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_WITHDRAW,
        token_0_amount.0,
        token_1_amount.0,
        max_shares_amount.0,
        leftpad_addr(to)
    )
}

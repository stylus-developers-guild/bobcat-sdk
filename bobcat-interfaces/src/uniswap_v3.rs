//! Calldata builders for Uniswap V3's canonical `ISwapRouter`.
//!
//! This module supports the deadline-bearing, single-hop `exactInputSingle` and
//! `exactOutputSingle` entrypoints. It does not target the different
//! `SwapRouter02` ABI.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_u24};
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];
pub type U24 = [u8; 3];
pub type U160 = [u8; 20];

selectors! {
    SEL_EXACT_INPUT_SINGLE = b"exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))",
    SEL_EXACT_OUTPUT_SINGLE = b"exactOutputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))",
}

/// Encode an exact-input swap through one Uniswap V3 pool.
#[allow(clippy::too_many_arguments)]
pub const fn make_fn_exact_input_single(
    token_in: Address,
    token_out: Address,
    fee: U24,
    recipient: Address,
    deadline: U,
    amount_in: U,
    amount_out_minimum: U,
    sqrt_price_limit_x96: U160,
) -> [u8; 4 + 32 * 8] {
    concat_arrays!(
        SEL_EXACT_INPUT_SINGLE,
        leftpad_addr(token_in),
        leftpad_addr(token_out),
        leftpad_u24(fee),
        leftpad_addr(recipient),
        deadline.0,
        amount_in.0,
        amount_out_minimum.0,
        leftpad_addr(sqrt_price_limit_x96)
    )
}

/// Encode an exact-output swap through one Uniswap V3 pool.
#[allow(clippy::too_many_arguments)]
pub const fn make_fn_exact_output_single(
    token_in: Address,
    token_out: Address,
    fee: U24,
    recipient: Address,
    deadline: U,
    amount_out: U,
    amount_in_maximum: U,
    sqrt_price_limit_x96: U160,
) -> [u8; 4 + 32 * 8] {
    concat_arrays!(
        SEL_EXACT_OUTPUT_SINGLE,
        leftpad_addr(token_in),
        leftpad_addr(token_out),
        leftpad_u24(fee),
        leftpad_addr(recipient),
        deadline.0,
        amount_out.0,
        amount_in_maximum.0,
        leftpad_addr(sqrt_price_limit_x96)
    )
}

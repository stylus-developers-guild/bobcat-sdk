//! Narrow Pendle V2 router calldata builders for core end-user flows.
//!
//! These builders cover PT/YT swaps and single-token liquidity using tokens
//! accepted directly by a market's SY. External swap aggregators and limit-order
//! fills are intentionally unsupported.

use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

/// Pendle's on-chain approximation bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApproxParams {
    pub guess_min: U,
    pub guess_max: U,
    pub guess_offchain: U,
    pub max_iteration: U,
    pub eps: U,
}

/// Input accepted directly by an SY, without an external swap aggregator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimpleTokenInput {
    pub token: Address,
    pub amount: U,
}

/// Output redeemed directly from an SY, without an external swap aggregator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimpleTokenOutput {
    pub token: Address,
    pub min_amount: U,
}

selectors! {
    SEL_SWAP_EXACT_TOKEN_FOR_PT = b"swapExactTokenForPt(address,address,uint256,(uint256,uint256,uint256,uint256,uint256),(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))",
    SEL_SWAP_EXACT_PT_FOR_TOKEN = b"swapExactPtForToken(address,address,uint256,(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))",
    SEL_SWAP_EXACT_TOKEN_FOR_YT = b"swapExactTokenForYt(address,address,uint256,(uint256,uint256,uint256,uint256,uint256),(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))",
    SEL_SWAP_EXACT_YT_FOR_TOKEN = b"swapExactYtForToken(address,address,uint256,(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))",
    SEL_ADD_LIQUIDITY_SINGLE_TOKEN = b"addLiquiditySingleToken(address,address,uint256,(uint256,uint256,uint256,uint256,uint256),(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))",
    SEL_REMOVE_LIQUIDITY_SINGLE_TOKEN = b"removeLiquiditySingleToken(address,address,uint256,(address,uint256,address,address,(uint8,address,bytes,bool)),(address,uint256,((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],((uint256,uint256,uint256,uint8,address,address,address,address,uint256,uint256,uint256,bytes),bytes,uint256)[],bytes))"
}

pub const TOKEN_IN_CALLDATA_LEN: usize = 4 + 32 * 28;
pub const TOKEN_OUT_CALLDATA_LEN: usize = 4 + 32 * 23;

fn put_word(out: &mut [u8], word: usize, value: &[u8; 32]) {
    let start = 4 + word * 32;
    out[start..start + 32].copy_from_slice(value);
}

fn put_address(out: &mut [u8], word: usize, value: Address) {
    let start = 4 + word * 32 + 12;
    out[start..start + 20].copy_from_slice(&value);
}

fn put_usize(out: &mut [u8], word: usize, value: usize) {
    let bytes = value.to_be_bytes();
    let start = 4 + word * 32 + 32 - bytes.len();
    out[start..start + bytes.len()].copy_from_slice(&bytes);
}

fn put_approx(out: &mut [u8], first_word: usize, approx: &ApproxParams) {
    put_word(out, first_word, &approx.guess_min.0);
    put_word(out, first_word + 1, &approx.guess_max.0);
    put_word(out, first_word + 2, &approx.guess_offchain.0);
    put_word(out, first_word + 3, &approx.max_iteration.0);
    put_word(out, first_word + 4, &approx.eps.0);
}

// Encodes createTokenInputSimple/createTokenOutputSimple: tokenMintSy or
// tokenRedeemSy equals token, SwapType::NONE, empty extCalldata, no scaling.
fn put_simple_token(out: &mut [u8], first_word: usize, token: Address, amount: &U) {
    put_address(out, first_word, token);
    put_word(out, first_word + 1, &amount.0);
    put_address(out, first_word + 2, token);
    // first_word + 3 is the zero pendleSwap address.
    put_usize(out, first_word + 4, 32 * 5);
    // The first two SwapData words are zero (SwapType::NONE, extRouter).
    put_usize(out, first_word + 7, 32 * 4);
    // needScale and the empty bytes length are already zero.
}

// Encodes createEmptyLimitOrderData(). All three dynamic values are empty.
fn put_empty_limit(out: &mut [u8], first_word: usize) {
    // limitRouter and epsSkipMarket are zero.
    put_usize(out, first_word + 2, 32 * 5);
    put_usize(out, first_word + 3, 32 * 6);
    put_usize(out, first_word + 4, 32 * 7);
    // Array lengths and bytes length are already zero.
}

fn make_token_in_call(
    selector: [u8; 4],
    receiver: Address,
    market: Address,
    minimum_out: &U,
    approx: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    let mut out = [0_u8; TOKEN_IN_CALLDATA_LEN];
    out[..4].copy_from_slice(&selector);
    put_address(&mut out, 0, receiver);
    put_address(&mut out, 1, market);
    put_word(&mut out, 2, &minimum_out.0);
    put_approx(&mut out, 3, approx);
    put_usize(&mut out, 8, 32 * 10);
    put_usize(&mut out, 9, 32 * 20);
    put_simple_token(&mut out, 10, input.token, &input.amount);
    put_empty_limit(&mut out, 20);
    out
}

fn make_token_out_call(
    selector: [u8; 4],
    receiver: Address,
    market: Address,
    exact_in: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    let mut out = [0_u8; TOKEN_OUT_CALLDATA_LEN];
    out[..4].copy_from_slice(&selector);
    put_address(&mut out, 0, receiver);
    put_address(&mut out, 1, market);
    put_word(&mut out, 2, &exact_in.0);
    put_usize(&mut out, 3, 32 * 5);
    put_usize(&mut out, 4, 32 * 15);
    put_simple_token(&mut out, 5, output.token, &output.min_amount);
    put_empty_limit(&mut out, 15);
    out
}

/// Encode a swap from an SY-supported token to PT.
pub fn make_fn_swap_exact_token_for_pt(
    receiver: Address,
    market: Address,
    min_pt_out: &U,
    guess_pt_out: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    make_token_in_call(
        SEL_SWAP_EXACT_TOKEN_FOR_PT,
        receiver,
        market,
        min_pt_out,
        guess_pt_out,
        input,
    )
}

/// Encode a swap from PT to an SY-supported token.
pub fn make_fn_swap_exact_pt_for_token(
    receiver: Address,
    market: Address,
    exact_pt_in: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    make_token_out_call(
        SEL_SWAP_EXACT_PT_FOR_TOKEN,
        receiver,
        market,
        exact_pt_in,
        output,
    )
}

/// Encode a swap from an SY-supported token to YT.
pub fn make_fn_swap_exact_token_for_yt(
    receiver: Address,
    market: Address,
    min_yt_out: &U,
    guess_yt_out: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    make_token_in_call(
        SEL_SWAP_EXACT_TOKEN_FOR_YT,
        receiver,
        market,
        min_yt_out,
        guess_yt_out,
        input,
    )
}

/// Encode a swap from YT to an SY-supported token.
pub fn make_fn_swap_exact_yt_for_token(
    receiver: Address,
    market: Address,
    exact_yt_in: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    make_token_out_call(
        SEL_SWAP_EXACT_YT_FOR_TOKEN,
        receiver,
        market,
        exact_yt_in,
        output,
    )
}

/// Encode adding liquidity from one SY-supported token.
pub fn make_fn_add_liquidity_single_token(
    receiver: Address,
    market: Address,
    min_lp_out: &U,
    guess_pt_received_from_sy: &ApproxParams,
    input: &SimpleTokenInput,
) -> [u8; TOKEN_IN_CALLDATA_LEN] {
    make_token_in_call(
        SEL_ADD_LIQUIDITY_SINGLE_TOKEN,
        receiver,
        market,
        min_lp_out,
        guess_pt_received_from_sy,
        input,
    )
}

/// Encode removing liquidity into one SY-supported token.
pub fn make_fn_remove_liquidity_single_token(
    receiver: Address,
    market: Address,
    net_lp_to_remove: &U,
    output: &SimpleTokenOutput,
) -> [u8; TOKEN_OUT_CALLDATA_LEN] {
    make_token_out_call(
        SEL_REMOVE_LIQUIDITY_SINGLE_TOKEN,
        receiver,
        market,
        net_lp_to_remove,
        output,
    )
}

//! Calldata builders for Basin Exchange (`IWell`) core end-user flows.
//!
//! Basin is a composable EVM-native DEX protocol where liquidity pools are
//! called Wells. Each Well is an ERC-20 LP token backed by an arbitrary well
//! function (e.g. constant product, stable swap). All swap and liquidity
//! operations are performed directly on the Well contract.
//!
//! This module covers the five core end-user functions:
//! - `swapFrom` — exact-input swap
//! - `swapTo` — exact-output swap
//! - `addLiquidity` — deposit multiple tokens for LP tokens
//! - `removeLiquidity` — burn LP tokens for all underlying tokens (balanced)
//! - `removeLiquidityOneToken` — burn LP tokens for a single underlying token
//!
//! The swap and single-token remove builders are fully static and return
//! fixed-size arrays. The `addLiquidity` and `removeLiquidity` builders
//! contain dynamic `uint256[]` parameters and write into a caller-supplied
//! buffer, matching the convention used by other bobcat interface modules
//! with dynamic ABI types.
//!
//! All builders are `no_std` and allocation-free.
//!
//! ABI reference:
//! <https://github.com/BeanstalkFarms/Basin/blob/master/src/interfaces/IWell.sol>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Maximum number of tokens supported by a Well.
pub const MAX_WELL_TOKENS: usize = 8;

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Token count is zero or exceeds [`MAX_WELL_TOKENS`].
    InvalidTokenCount,
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
}

selectors! {
    SEL_SWAP_FROM = b"swapFrom(address,address,uint256,uint256,address,uint256)",
    SEL_SWAP_TO = b"swapTo(address,address,uint256,uint256,address,uint256)",
    SEL_ADD_LIQUIDITY = b"addLiquidity(uint256[],uint256,address,uint256)",
    SEL_REMOVE_LIQUIDITY = b"removeLiquidity(uint256,uint256[],address,uint256)",
    SEL_REMOVE_LIQUIDITY_ONE_TOKEN = b"removeLiquidityOneToken(uint256,address,uint256,address,uint256)",
}

// ---------------------------------------------------------------------------
// Mutable-slice helpers
// ---------------------------------------------------------------------------

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

fn put_word(output: &mut [u8], offset: usize, value: &[u8; 32]) {
    output[offset..offset + 32].copy_from_slice(value);
}

fn put_addr(output: &mut [u8], offset: usize, value: Address) {
    put_word(output, offset, &leftpad_addr(value));
}

// ---------------------------------------------------------------------------
// swapFrom — exact-input swap (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a Basin `swapFrom` call: swap an exact amount of `fromToken` for at
/// least `minAmountOut` of `toToken`.
pub const fn make_fn_swap_from(
    from_token: Address,
    to_token: Address,
    amount_in: &U,
    min_amount_out: &U,
    recipient: Address,
    deadline: &U,
) -> [u8; 4 + 32 * 6] {
    concat_arrays!(
        SEL_SWAP_FROM,
        leftpad_addr(from_token),
        leftpad_addr(to_token),
        amount_in.0,
        min_amount_out.0,
        leftpad_addr(recipient),
        deadline.0
    )
}

// ---------------------------------------------------------------------------
// swapTo — exact-output swap (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a Basin `swapTo` call: swap at most `maxAmountIn` of `fromToken`
/// for exactly `amountOut` of `toToken`.
pub const fn make_fn_swap_to(
    from_token: Address,
    to_token: Address,
    max_amount_in: &U,
    amount_out: &U,
    recipient: Address,
    deadline: &U,
) -> [u8; 4 + 32 * 6] {
    concat_arrays!(
        SEL_SWAP_TO,
        leftpad_addr(from_token),
        leftpad_addr(to_token),
        max_amount_in.0,
        amount_out.0,
        leftpad_addr(recipient),
        deadline.0
    )
}

// ---------------------------------------------------------------------------
// addLiquidity — deposit (uint256[], uint256, address, uint256)
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_add_liquidity`].
///
/// `addLiquidity(uint256[],uint256,address,uint256)` encodes as:
/// selector(4) + offset(32) + minLpAmountOut(32) + recipient(32) + deadline(32)
/// + length(32) + N words.
pub fn add_liquidity_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_WELL_TOKENS {
        return Err(EncodeError::InvalidTokenCount);
    }
    4_usize
        .checked_add(32 * 5)
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::InvalidTokenCount)
}

/// Encode a Basin `addLiquidity` call into `output`. Returns the number of
/// bytes written.
///
/// `token_amounts_in` is the per-token deposit amount slice; its length
/// determines the Well's token count (1 through [`MAX_WELL_TOKENS`]). The
/// caller must supply a buffer of at least
/// [`add_liquidity_calldata_len`]`(token_amounts_in.len())` bytes.
pub fn make_fn_add_liquidity(
    output: &mut [u8],
    token_amounts_in: &[U],
    min_lp_amount_out: &U,
    recipient: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let n = token_amounts_in.len();
    let required = add_liquidity_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_ADD_LIQUIDITY);
    // head: offset to uint256[], minLpAmountOut, recipient, deadline
    // offset = 4 head words * 32 = 128 = 0x80
    put_usize(out, 4, 128);
    put_word(out, 36, &min_lp_amount_out.0);
    put_addr(out, 68, recipient);
    put_word(out, 100, &deadline.0);
    // dynamic array tail: length prefix + N amount words
    let arr = 4 + 32 * 4; // 132
    put_usize(out, arr, n);
    for (i, amount) in token_amounts_in.iter().enumerate() {
        put_word(out, arr + 32 + i * 32, &amount.0);
    }
    Ok(required)
}

// ---------------------------------------------------------------------------
// removeLiquidity — balanced withdrawal (uint256, uint256[], address, uint256)
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_remove_liquidity`].
///
/// `removeLiquidity(uint256,uint256[],address,uint256)` encodes as:
/// selector(4) + lpAmountIn(32) + offset(32) + recipient(32) + deadline(32)
/// + length(32) + N words.
pub fn remove_liquidity_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_WELL_TOKENS {
        return Err(EncodeError::InvalidTokenCount);
    }
    4_usize
        .checked_add(32 * 5)
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::InvalidTokenCount)
}

/// Encode a Basin `removeLiquidity` call (balanced withdrawal) into `output`.
/// Returns the number of bytes written.
///
/// `min_token_amounts_out` is the per-token minimum output floor; its length
/// determines the Well's token count (1 through [`MAX_WELL_TOKENS`]). The
/// caller must supply a buffer of at least
/// [`remove_liquidity_calldata_len`]`(min_token_amounts_out.len())` bytes.
pub fn make_fn_remove_liquidity(
    output: &mut [u8],
    lp_amount_in: &U,
    min_token_amounts_out: &[U],
    recipient: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let n = min_token_amounts_out.len();
    let required = remove_liquidity_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_REMOVE_LIQUIDITY);
    // head: lpAmountIn, offset to uint256[], recipient, deadline
    // offset = 4 head words * 32 = 128 = 0x80
    put_word(out, 4, &lp_amount_in.0);
    put_usize(out, 36, 128);
    put_addr(out, 68, recipient);
    put_word(out, 100, &deadline.0);
    // dynamic array tail: length prefix + N min_amount words
    let arr = 4 + 32 * 4; // 132
    put_usize(out, arr, n);
    for (i, amount) in min_token_amounts_out.iter().enumerate() {
        put_word(out, arr + 32 + i * 32, &amount.0);
    }
    Ok(required)
}

// ---------------------------------------------------------------------------
// removeLiquidityOneToken — single-token withdrawal (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a Basin `removeLiquidityOneToken` call: burn LP tokens to receive a
/// single underlying token.
pub const fn make_fn_remove_liquidity_one_token(
    lp_amount_in: &U,
    token_out: Address,
    min_token_amount_out: &U,
    recipient: Address,
    deadline: &U,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_REMOVE_LIQUIDITY_ONE_TOKEN,
        lp_amount_in.0,
        leftpad_addr(token_out),
        min_token_amount_out.0,
        leftpad_addr(recipient),
        deadline.0
    )
}

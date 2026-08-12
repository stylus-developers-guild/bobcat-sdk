//! Calldata builders for SushiSwap V2 Router02 exact-input swaps.
//!
//! SushiSwap on Arbitrum deploys a V2 router whose interface is
//! ABI-identical to Uniswap's `IUniswapV2Router02`.  The three
//! exact-input entrypoints share a dynamic `address[]` path parameter;
//! builders write into a caller-supplied buffer and return the number
//! of bytes written, matching the convention used by other bobcat
//! interface modules with dynamic ABI types.
//!
//! Only core end-user swap functionality is exposed.  Token approvals
//! and any native token call value remain the caller's responsibility.
//!
//! ABI reference:
//! - IUniswapV2Router01: <https://github.com/Uniswap/v2-periphery/blob/master/contracts/interfaces/IUniswapV2Router01.sol>
//! - IUniswapV2Router02: <https://github.com/Uniswap/v2-periphery/blob/master/contracts/interfaces/IUniswapV2Router02.sol>
//! - SushiSwap is a V2 fork; the router implements the same interface.

use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Path must contain at least 2 addresses.
    PathTooShort,
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
}

selectors! {
    SEL_SWAP_EXACT_TOKENS_FOR_TOKENS = b"swapExactTokensForTokens(uint256,uint256,address[],address,uint256)",
    SEL_SWAP_EXACT_ETH_FOR_TOKENS = b"swapExactETHForTokens(uint256,address[],address,uint256)",
    SEL_SWAP_EXACT_TOKENS_FOR_ETH = b"swapExactTokensForETH(uint256,uint256,address[],address,uint256)",
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn put_word(output: &mut [u8], offset: usize, value: &[u8; 32]) {
    output[offset..offset + 32].copy_from_slice(value);
}

fn put_addr(output: &mut [u8], offset: usize, value: Address) {
    put_word(output, offset, &leftpad_addr(value));
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

/// Encode the dynamic `address[]` path tail into `output` at `offset`.
/// Returns the number of bytes written (32 for length + 32 per address).
fn put_path(output: &mut [u8], offset: usize, path: &[Address]) -> usize {
    put_usize(output, offset, path.len());
    for (i, addr) in path.iter().enumerate() {
        put_addr(output, offset + 32 + i * 32, *addr);
    }
    32 + path.len() * 32
}

// ---------------------------------------------------------------------------
// swapExactTokensForTokens
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_swap_exact_tokens_for_tokens`].
///
/// `swapExactTokensForTokens(uint256,uint256,address[],address,uint256)`
/// encodes as: selector(4) + 5 head words + length(32) + N path words.
pub fn swap_exact_tokens_for_tokens_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n < 2 {
        return Err(EncodeError::PathTooShort);
    }
    4_usize
        .checked_add(32 * 5)
        .and_then(|s| s.checked_add(32))
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::PathTooShort)
}

/// Encode an exact-input token-for-token swap into `output`. Returns the
/// number of bytes written.
///
/// `path` must contain at least 2 addresses. The caller must supply a buffer
/// of at least [`swap_exact_tokens_for_tokens_calldata_len`]`(path.len())`
/// bytes.
pub fn make_fn_swap_exact_tokens_for_tokens(
    output: &mut [u8],
    amount_in: &U,
    amount_out_min: &U,
    path: &[Address],
    to: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let n = path.len();
    let required = swap_exact_tokens_for_tokens_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_SWAP_EXACT_TOKENS_FOR_TOKENS);
    // head: amountIn, amountOutMin, offset, to, deadline
    put_word(out, 4, &amount_in.0);
    put_word(out, 36, &amount_out_min.0);
    put_usize(out, 68, 160); // offset to path: 5 words * 32
    put_addr(out, 100, to);
    put_word(out, 132, &deadline.0);
    // tail: path array
    let _ = put_path(out, 160, path);
    Ok(required)
}

// ---------------------------------------------------------------------------
// swapExactETHForTokens
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_swap_exact_eth_for_tokens`].
///
/// `swapExactETHForTokens(uint256,address[],address,uint256)` encodes as:
/// selector(4) + 4 head words + length(32) + N path words.
pub fn swap_exact_eth_for_tokens_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n < 2 {
        return Err(EncodeError::PathTooShort);
    }
    4_usize
        .checked_add(32 * 4)
        .and_then(|s| s.checked_add(32))
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::PathTooShort)
}

/// Encode an exact-input native-asset-for-tokens swap into `output`. Returns
/// the number of bytes written. The caller must send `msg.value` equal to
/// the desired input amount.
///
/// `path[0]` must be the WETH address. The caller must supply a buffer of at
/// least [`swap_exact_eth_for_tokens_calldata_len`]`(path.len())` bytes.
pub fn make_fn_swap_exact_eth_for_tokens(
    output: &mut [u8],
    amount_out_min: &U,
    path: &[Address],
    to: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let n = path.len();
    let required = swap_exact_eth_for_tokens_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_SWAP_EXACT_ETH_FOR_TOKENS);
    // head: amountOutMin, offset, to, deadline
    put_word(out, 4, &amount_out_min.0);
    put_usize(out, 36, 128); // offset to path: 4 words * 32
    put_addr(out, 68, to);
    put_word(out, 100, &deadline.0);
    // tail: path array
    let _ = put_path(out, 128, path);
    Ok(required)
}

// ---------------------------------------------------------------------------
// swapExactTokensForETH
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_swap_exact_tokens_for_eth`].
///
/// `swapExactTokensForETH(uint256,uint256,address[],address,uint256)` encodes
/// as: selector(4) + 5 head words + length(32) + N path words.
pub fn swap_exact_tokens_for_eth_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n < 2 {
        return Err(EncodeError::PathTooShort);
    }
    4_usize
        .checked_add(32 * 5)
        .and_then(|s| s.checked_add(32))
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::PathTooShort)
}

/// Encode an exact-input tokens-for-native-asset swap into `output`. Returns
/// the number of bytes written.
///
/// `path[path.len()-1]` must be the WETH address. The caller must supply a
/// buffer of at least [`swap_exact_tokens_for_eth_calldata_len`]`(path.len())`
/// bytes.
pub fn make_fn_swap_exact_tokens_for_eth(
    output: &mut [u8],
    amount_in: &U,
    amount_out_min: &U,
    path: &[Address],
    to: Address,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let n = path.len();
    let required = swap_exact_tokens_for_eth_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_SWAP_EXACT_TOKENS_FOR_ETH);
    // head: amountIn, amountOutMin, offset, to, deadline
    put_word(out, 4, &amount_in.0);
    put_word(out, 36, &amount_out_min.0);
    put_usize(out, 68, 160); // offset to path: 5 words * 32
    put_addr(out, 100, to);
    put_word(out, 132, &deadline.0);
    // tail: path array
    let _ = put_path(out, 160, path);
    Ok(required)
}

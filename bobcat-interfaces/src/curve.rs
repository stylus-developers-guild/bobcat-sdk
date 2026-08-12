//! Narrow Curve calldata builders for current router and pool core flows.
//!
//! Router swaps target Curve Router NG's explicit-receiver `exchange` overload.
//! Direct liquidity builders target StableSwap-NG dynamic coin arrays and the
//! two- and three-coin CryptoSwap-NG pool ABIs. Token approvals and any native
//! token call value remain the caller's responsibility.
//!
//! All builders are `no_std` and allocation-free. Fixed-ABI functions return
//! fixed-size arrays; the StableSwap-NG `add_liquidity` and `remove_liquidity`
//! builders (which contain dynamic `uint256[]` parameters) write into a
//! caller-supplied buffer and return the number of bytes written, matching the
//! convention used by other bobcat interface modules with dynamic ABI types.
//!
//! ABI references:
//! - <https://github.com/curvefi/curve-router-ng/blob/master/contracts/Router.vy>
//! - <https://github.com/curvefi/stableswap-ng/blob/main/contracts/main/CurveStableSwapNG.vy>
//! - <https://github.com/curvefi/twocrypto-ng/blob/main/contracts/main/Twocrypto.vy>
//! - <https://github.com/curvefi/tricrypto-ng/blob/main/contracts/main/CurveTricryptoOptimized.vy>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Maximum number of coins accepted by StableSwap-NG pools.
pub const MAX_STABLE_COINS: usize = 8;

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Coin count is zero or exceeds [`MAX_STABLE_COINS`].
    InvalidCoinCount,
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
}

selectors! {
    SEL_ROUTER_EXCHANGE = b"exchange(address[11],uint256[5][5],uint256,uint256,address[5],address)",
    SEL_STABLE_EXCHANGE = b"exchange(int128,int128,uint256,uint256,address)",
    SEL_STABLE_ADD_LIQUIDITY = b"add_liquidity(uint256[],uint256,address)",
    SEL_STABLE_REMOVE_LIQUIDITY = b"remove_liquidity(uint256,uint256[],address)",
    SEL_STABLE_REMOVE_LIQUIDITY_ONE_COIN = b"remove_liquidity_one_coin(uint256,int128,uint256,address)",
    SEL_CRYPTO_EXCHANGE = b"exchange(uint256,uint256,uint256,uint256,address)",
    SEL_CRYPTO_ADD_LIQUIDITY_2 = b"add_liquidity(uint256[2],uint256,address)",
    SEL_CRYPTO_ADD_LIQUIDITY_3 = b"add_liquidity(uint256[3],uint256,address)",
    SEL_CRYPTO_REMOVE_LIQUIDITY_2 = b"remove_liquidity(uint256,uint256[2],address)",
    SEL_CRYPTO_REMOVE_LIQUIDITY_3 = b"remove_liquidity(uint256,uint256[3],address)",
    SEL_CRYPTO_REMOVE_LIQUIDITY_ONE_COIN = b"remove_liquidity_one_coin(uint256,uint256,uint256,address)",
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
// Curve Router NG — exchange (fully static ABI)
// ---------------------------------------------------------------------------

/// Calldata length for [`make_fn_router_exchange`].
///
/// `exchange(address[11],uint256[5][5],uint256,uint256,address[5],address)`
/// is fully static: 11 + 25 + 2 + 5 + 1 = 44 words.
pub const ROUTER_EXCHANGE_CALLDATA_LEN: usize = 4 + 32 * 44;

/// Encode a Curve Router NG swap of up to five hops.
///
/// Each swap parameter row is `[i, j, swap_type, pool_type, n_coins]`. Unused
/// route, parameter, and pool entries should be zeroed as required by Router NG.
pub fn make_fn_router_exchange(
    route: &[Address; 11],
    swap_params: &[[U; 5]; 5],
    amount: &U,
    min_dy: &U,
    pools: &[Address; 5],
    receiver: Address,
) -> [u8; ROUTER_EXCHANGE_CALLDATA_LEN] {
    let mut out = [0u8; ROUTER_EXCHANGE_CALLDATA_LEN];
    out[..4].copy_from_slice(&SEL_ROUTER_EXCHANGE);
    let mut o = 4;
    for addr in route {
        put_addr(&mut out, o, *addr);
        o += 32;
    }
    for row in swap_params {
        for value in row {
            put_word(&mut out, o, &value.0);
            o += 32;
        }
    }
    put_word(&mut out, o, &amount.0);
    o += 32;
    put_word(&mut out, o, &min_dy.0);
    o += 32;
    for pool in pools {
        put_addr(&mut out, o, *pool);
        o += 32;
    }
    put_addr(&mut out, o, receiver);
    out
}

// ---------------------------------------------------------------------------
// StableSwap-NG — exchange
// ---------------------------------------------------------------------------

/// Encode a StableSwap-NG exact-input exchange.
pub const fn make_fn_stable_exchange(
    coin_in: u8,
    coin_out: u8,
    amount_in: &U,
    min_amount_out: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_STABLE_EXCHANGE,
        bobcat_cd::leftpad_u8(coin_in),
        bobcat_cd::leftpad_u8(coin_out),
        amount_in.0,
        min_amount_out.0,
        leftpad_addr(receiver)
    )
}

/// Encode a CryptoSwap-NG exact-input exchange.
pub const fn make_fn_crypto_exchange(
    coin_in: u8,
    coin_out: u8,
    amount_in: &U,
    min_amount_out: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_CRYPTO_EXCHANGE,
        bobcat_cd::leftpad_u8(coin_in),
        bobcat_cd::leftpad_u8(coin_out),
        amount_in.0,
        min_amount_out.0,
        leftpad_addr(receiver)
    )
}

// ---------------------------------------------------------------------------
// StableSwap-NG — add_liquidity (uint256[], uint256, address)
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_stable_add_liquidity`].
///
/// `add_liquidity(uint256[],uint256,address)` encodes as:
/// selector(4) + offset(32) + min_mint(32) + receiver(32) + length(32) + N words.
pub fn stable_add_liquidity_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_STABLE_COINS {
        return Err(EncodeError::InvalidCoinCount);
    }
    4_usize
        .checked_add(32 * 4)
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::InvalidCoinCount)
}

/// Encode a StableSwap-NG deposit into `output`. Returns the number of bytes
/// written.
///
/// `amounts` is the per-coin deposit amount slice; its length determines the
/// pool's coin count (1 through [`MAX_STABLE_COINS`]). The caller must supply a
/// buffer of at least [`stable_add_liquidity_calldata_len`]`(amounts.len())`
/// bytes.
pub fn make_fn_stable_add_liquidity(
    output: &mut [u8],
    amounts: &[U],
    min_mint_amount: &U,
    receiver: Address,
) -> Result<usize, EncodeError> {
    let n = amounts.len();
    let required = stable_add_liquidity_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_STABLE_ADD_LIQUIDITY);
    // head: offset to uint256[], min_mint_amount, receiver
    put_usize(out, 4, 96);
    put_word(out, 36, &min_mint_amount.0);
    put_addr(out, 68, receiver);
    // dynamic array tail: length prefix + N amount words
    let arr = 4 + 32 * 3;
    put_usize(out, arr, n);
    for (i, amount) in amounts.iter().enumerate() {
        put_word(out, arr + 32 + i * 32, &amount.0);
    }
    Ok(required)
}

// ---------------------------------------------------------------------------
// StableSwap-NG — remove_liquidity (uint256, uint256[], address)
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_stable_remove_liquidity`].
///
/// `remove_liquidity(uint256,uint256[],address)` encodes as:
/// selector(4) + burn(32) + offset(32) + receiver(32) + length(32) + N words.
pub fn stable_remove_liquidity_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_STABLE_COINS {
        return Err(EncodeError::InvalidCoinCount);
    }
    4_usize
        .checked_add(32 * 4)
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::InvalidCoinCount)
}

/// Encode a proportional StableSwap-NG withdrawal into `output`. Returns the
/// number of bytes written.
///
/// `min_amounts` is the per-coin minimum output floor; its length determines
/// the pool's coin count (1 through [`MAX_STABLE_COINS`]). The caller must
/// supply a buffer of at least
/// [`stable_remove_liquidity_calldata_len`]`(min_amounts.len())` bytes.
pub fn make_fn_stable_remove_liquidity(
    output: &mut [u8],
    burn_amount: &U,
    min_amounts: &[U],
    receiver: Address,
) -> Result<usize, EncodeError> {
    let n = min_amounts.len();
    let required = stable_remove_liquidity_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_STABLE_REMOVE_LIQUIDITY);
    // head: burn_amount, offset to uint256[], receiver
    put_word(out, 4, &burn_amount.0);
    put_usize(out, 36, 96);
    put_addr(out, 68, receiver);
    // dynamic array tail: length prefix + N min_amount words
    let arr = 4 + 32 * 3;
    put_usize(out, arr, n);
    for (i, amount) in min_amounts.iter().enumerate() {
        put_word(out, arr + 32 + i * 32, &amount.0);
    }
    Ok(required)
}

// ---------------------------------------------------------------------------
// StableSwap-NG — remove_liquidity_one_coin
// ---------------------------------------------------------------------------

/// Encode a single-coin StableSwap-NG withdrawal.
pub const fn make_fn_stable_remove_liquidity_one_coin(
    burn_amount: &U,
    coin: u8,
    min_received: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_STABLE_REMOVE_LIQUIDITY_ONE_COIN,
        burn_amount.0,
        bobcat_cd::leftpad_u8(coin),
        min_received.0,
        leftpad_addr(receiver)
    )
}

// ---------------------------------------------------------------------------
// CryptoSwap-NG — add_liquidity (fixed [2] / [3])
// ---------------------------------------------------------------------------

/// Encode a two-coin CryptoSwap-NG deposit.
pub const fn make_fn_crypto_add_liquidity_2(
    amounts: &[U; 2],
    min_mint_amount: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_CRYPTO_ADD_LIQUIDITY_2,
        amounts[0].0,
        amounts[1].0,
        min_mint_amount.0,
        leftpad_addr(receiver)
    )
}

/// Encode a three-coin CryptoSwap-NG deposit.
pub const fn make_fn_crypto_add_liquidity_3(
    amounts: &[U; 3],
    min_mint_amount: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_CRYPTO_ADD_LIQUIDITY_3,
        amounts[0].0,
        amounts[1].0,
        amounts[2].0,
        min_mint_amount.0,
        leftpad_addr(receiver)
    )
}

// ---------------------------------------------------------------------------
// CryptoSwap-NG — remove_liquidity (fixed [2] / [3])
// ---------------------------------------------------------------------------

/// Encode a proportional two-coin CryptoSwap-NG withdrawal.
pub const fn make_fn_crypto_remove_liquidity_2(
    burn_amount: &U,
    min_amounts: &[U; 2],
    receiver: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_CRYPTO_REMOVE_LIQUIDITY_2,
        burn_amount.0,
        min_amounts[0].0,
        min_amounts[1].0,
        leftpad_addr(receiver)
    )
}

/// Encode a proportional three-coin CryptoSwap-NG withdrawal.
pub const fn make_fn_crypto_remove_liquidity_3(
    burn_amount: &U,
    min_amounts: &[U; 3],
    receiver: Address,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_CRYPTO_REMOVE_LIQUIDITY_3,
        burn_amount.0,
        min_amounts[0].0,
        min_amounts[1].0,
        min_amounts[2].0,
        leftpad_addr(receiver)
    )
}

// ---------------------------------------------------------------------------
// CryptoSwap-NG — remove_liquidity_one_coin
// ---------------------------------------------------------------------------

/// Encode a single-coin CryptoSwap-NG withdrawal.
pub const fn make_fn_crypto_remove_liquidity_one_coin(
    burn_amount: &U,
    coin: u8,
    min_received: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_CRYPTO_REMOVE_LIQUIDITY_ONE_COIN,
        burn_amount.0,
        bobcat_cd::leftpad_u8(coin),
        min_received.0,
        leftpad_addr(receiver)
    )
}

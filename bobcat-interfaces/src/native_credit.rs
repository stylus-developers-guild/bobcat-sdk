//! Calldata builders for Native Credit Pool core lending flows on Arbitrum.
//!
//! Native is an on-chain price-discovery and execution system whose credit
//! layer — the **Native Credit Pool** — lets liquidity providers (LPs) supply
//! assets to a `CreditVault` and lets authorised market makers borrow those
//! assets against collateral.  The protocol is deployed across multiple
//! chains; on Arbitrum the `CreditVault` and per-asset `NativeLPToken`
//! contracts implement the core lending primitives.
//!
//! Two contracts are involved:
//!
//! - **`NativeLPToken`** — a yield-bearing ERC-20 that wraps a single
//!   underlying asset.  LPs call `deposit` to supply underlying tokens and
//!   receive LP shares, and `redeem` to burn shares and receive the
//!   corresponding underlying amount back from the vault.
//!
//! - **`CreditVault`** — the custodial contract that holds all supplied
//!   assets.  Market makers open and adjust borrowed positions via
//!   `settle` (which requires an off-chain signature from the protocol
//!   signer) and close or reduce them via `repay`.
//!
//! ## End-user functions exposed
//!
//! - `deposit(uint256 amount)` — supply `amount` of the underlying token
//!   to the `NativeLPToken` contract and receive LP shares at the current
//!   exchange rate.
//! - `redeem(uint256 sharesToBurn)` — burn `sharesToBurn` LP shares and
//!   receive the corresponding underlying amount from the vault.
//! - `settle(SettlementRequest request, bytes signature)` — open, modify,
//!   or close a market-maker position.  A negative `positionUpdates` entry
//!   moves tokens *out* of the vault (borrowing); a positive entry moves
//!   tokens *into* the vault.  The `signature` must be produced by the
//!   protocol's authorised signer.
//! - `repay(TokenAmountInt[] positionUpdates, address trader)` — repay
//!   one or more of `trader`'s short positions.  Each `amount` must be
//!   positive; the corresponding token is pulled from `msg.sender`.
//!
//! ## Permission constraints
//!
//! Only the four core lending functions are exposed.  Administration
//! (`supportMarket`, `setCreditPool`, `setAllowance`, `setTrader`,
//! `setLiquidator`, `setSigner`, `setEpochUpdater`, `setFeeWithdrawer`,
//! `setRebalanceCap`), fee withdrawal (`withdrawReserve`), liquidation
//! (`liquidate`), epoch funding updates (`epochUpdate`), collateral
//! removal with signature (`removeCollateral`), the internal `swapCallback`
//! and `pay` helpers, and the `NativeLPToken` admin/pause functions
//! (`setMinDeposit`, `setMinRedeemInterval`, `setEarlyWithdrawFeeBips`,
//! `setTrustedOperator`, `setRedeemCooldownExempt`, `withdrawEarlyFees`,
//! `pauseDeposit`, `unpauseDeposit`, `pauseRedeem`, `unpauseRedeem`,
//! `depositFor`, `redeemTo`, `transferShares`, `distributeYield`) are
//! intentionally omitted.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official Native
//! contract source at
//! <https://github.com/Native-org/native-v2-core>
//! (`src/CreditVault.sol`, `src/NativeLPToken.sol`,
//! `src/interfaces/ICreditVault.sol`).
//!
//! `deposit` and `redeem` are fully static (single `uint256` argument), so
//! they return fixed-size arrays.  `repay` encodes a dynamic array of
//! `(address, int256)` tuples, and `settle` encodes a dynamic tuple
//! containing a nested dynamic array plus a `bytes` signature; both use
//! the caller-supplied mutable-slice pattern employed by other bobcat
//! interface modules with dynamic ABI types.
//!
//! All builders are `no_std` and allocation-free.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Maximum number of position updates supported in a single `repay` or
/// `settle` call.
pub const MAX_POSITION_UPDATES: usize = 16;

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Position-update count is zero or exceeds [`MAX_POSITION_UPDATES`].
    InvalidPositionCount,
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall {
        /// Minimum buffer length needed.
        required: usize,
    },
}

// --------------------------------------------------------------------------- //
// Selectors
// --------------------------------------------------------------------------- //

selectors! {
    SEL_DEPOSIT = b"deposit(uint256)",
    SEL_REDEEM = b"redeem(uint256)",
    SEL_REPAY = b"repay((address,int256)[],address)",
    SEL_SETTLE = b"settle((uint256,uint256,address,(address,int256)[]),bytes)",
}

// --------------------------------------------------------------------------- //
// Shared types
// --------------------------------------------------------------------------- //

/// A signed token-amount pair, matching the Solidity `TokenAmountInt` struct
/// `{ address token; int256 amount; }`.
///
/// Used in `repay` and inside `SettlementRequest.positionUpdates` for
/// `settle`.  `amount` is a raw 32-byte big-endian value; for `repay` it must
/// be non-negative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenAmountInt {
    /// The ERC-20 token address.
    pub token: Address,
    /// The signed amount (big-endian `[u8; 32]`).
    pub amount: U,
}

/// A settlement request, matching the Solidity `SettlementRequest` struct
/// `{ uint256 nonce; uint256 deadline; address trader; TokenAmountInt[] positionUpdates; }`.
///
/// Used as the first parameter of `settle`.  `nonce` is a replay-protection
/// identifier, `deadline` is the timestamp after which the request expires,
/// `trader` is the address whose positions are being settled, and
/// `position_updates` is the array of position changes (negative = borrow,
/// positive = repay/deposit).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettlementRequest<'a> {
    /// Unique nonce to prevent replay.
    pub nonce: U,
    /// Timestamp after which the request expires.
    pub deadline: U,
    /// The trader whose positions are being settled.
    pub trader: Address,
    /// Position changes to apply.
    pub position_updates: &'a [TokenAmountInt],
}

// --------------------------------------------------------------------------- //
// Mutable-slice helpers
// --------------------------------------------------------------------------- //

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

// --------------------------------------------------------------------------- //
// deposit — supply underlying tokens to NativeLPToken (fully static ABI)
// --------------------------------------------------------------------------- //

/// Encode `deposit(amount)` for a `NativeLPToken` contract.
///
/// Supplies `amount` of the underlying token to the LP pool.  The caller
/// must have approved the `NativeLPToken` contract to spend `amount` of
/// the underlying token.  Shares are minted to `msg.sender` at the current
/// exchange rate.  A minimum deposit threshold is enforced by the contract;
/// deposits below `minDeposit` revert.
///
/// `amount` is the raw underlying token amount (matching the underlying
/// token's decimals) to deposit.
pub const fn make_fn_deposit(amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_DEPOSIT, amount.0)
}

// --------------------------------------------------------------------------- //
// redeem — withdraw underlying tokens from NativeLPToken (fully static ABI)
// --------------------------------------------------------------------------- //

/// Encode `redeem(sharesToBurn)` for a `NativeLPToken` contract.
///
/// Burns `sharesToBurn` LP shares belonging to `msg.sender` and transfers
/// the corresponding underlying amount from the `CreditVault` back to the
/// caller.  An early-withdrawal fee may apply if the redeem occurs within
/// the minimum redeem interval since the caller's last deposit, unless the
/// caller is exempt.
///
/// `shares_to_burn` is the amount of LP shares to redeem (in share units,
/// not underlying units).  Convert underlying amounts to shares via the
/// `getSharesByUnderlying` view on `NativeLPToken`.
pub const fn make_fn_redeem(shares_to_burn: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_REDEEM, shares_to_burn.0)
}

// --------------------------------------------------------------------------- //
// repay — repay a trader's short positions (dynamic array of structs)
// --------------------------------------------------------------------------- //

/// Required output length for [`make_fn_repay`].
///
/// `repay((address,int256)[],address)` encodes as:
/// selector(4) + offset(32) + trader(32) + length(32) + N * (token + amount)
/// = 4 + 32*2 + 32 + 32*2*N
pub fn repay_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_POSITION_UPDATES {
        return Err(EncodeError::InvalidPositionCount);
    }
    4_usize
        .checked_add(32 * 2) // head: offset + trader
        .and_then(|s| s.checked_add(32)) // array length
        .and_then(|s| s.checked_add(32 * 2 * n)) // N * (address, int256)
        .ok_or(EncodeError::InvalidPositionCount)
}

/// Encode a `CreditVault` `repay` call into `output`.  Returns the number of
/// bytes written.
///
/// Repays one or more of `trader`'s short positions.  For each entry in
/// `position_updates`, the `amount` must be positive (the contract reverts
/// on negative values) and the corresponding `token` is pulled from
/// `msg.sender` via `transferFrom`.  Only the trader or an authorised
/// settler may call this function on the trader's behalf.
///
/// The caller must supply a buffer of at least
/// [`repay_calldata_len`]`(position_updates.len())` bytes.
pub fn make_fn_repay(
    output: &mut [u8],
    position_updates: &[TokenAmountInt],
    trader: Address,
) -> Result<usize, EncodeError> {
    let n = position_updates.len();
    let required = repay_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_REPAY);

    // Head: offset to positionUpdates array, trader
    // offset = 2 head words * 32 = 64 = 0x40
    put_usize(out, 4, 64);
    put_addr(out, 36, trader);

    // Array tail: length prefix + N * (address, int256) tuples
    let arr = 4 + 32 * 2; // 68
    put_usize(out, arr, n);
    for (i, entry) in position_updates.iter().enumerate() {
        let base = arr + 32 + i * 64;
        put_addr(out, base, entry.token);
        put_word(out, base + 32, &entry.amount.0);
    }

    Ok(required)
}

// --------------------------------------------------------------------------- //
// settle — open/modify/close a market-maker position (complex nested ABI)
// --------------------------------------------------------------------------- //

/// Required output length for [`make_fn_settle`].
///
/// `settle((uint256,uint256,address,(address,int256)[]),bytes)` encodes as:
/// ```text
/// selector(4)
/// + head: offset_to_request(32) + offset_to_signature(32)
/// + request tuple: nonce(32) + deadline(32) + trader(32)
///   + offset_to_positionUpdates(32)
///   + array: length(32) + N * (address + int256)
/// + signature: length(32) + ceil(sig_len/32) * 32
/// ```
pub fn settle_calldata_len(n: usize, sig_len: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_POSITION_UPDATES {
        return Err(EncodeError::InvalidPositionCount);
    }
    let sig_padded = sig_len.div_ceil(32) * 32;
    4_usize
        .checked_add(32 * 2) // head: 2 offsets
        .and_then(|s| s.checked_add(32 * 4)) // tuple: nonce, deadline, trader, array offset
        .and_then(|s| s.checked_add(32)) // array length
        .and_then(|s| s.checked_add(32 * 2 * n)) // N * (address, int256)
        .and_then(|s| s.checked_add(32 + sig_padded)) // signature: length + padded data
        .ok_or(EncodeError::InvalidPositionCount)
}

/// Encode a `CreditVault` `settle` call into `output`.  Returns the number
/// of bytes written.
///
/// Opens, modifies, or closes a market-maker position.  Each entry in
/// `request.position_updates` represents a token-amount delta: a positive
/// `amount` moves tokens from `msg.sender` into the vault (increasing
/// the trader's long position or reducing a short); a negative `amount`
/// moves tokens from the vault to the trader's configured recipient
/// (borrowing / closing a long).  The `signature` must be an EIP-712
/// signature over the `SettlementRequest` produced by the protocol's
/// authorised signer.
///
/// Only the trader or an authorised settler may call this function.  The
/// caller must supply a buffer of at least
/// [`settle_calldata_len`]`(request.position_updates.len(), signature.len())`
/// bytes.
pub fn make_fn_settle(
    output: &mut [u8],
    request: &SettlementRequest,
    signature: &[u8],
) -> Result<usize, EncodeError> {
    let n = request.position_updates.len();
    let required = settle_calldata_len(n, signature.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_SETTLE);

    // --- Head: two offsets ---
    // offset to request tuple = 2 head words * 32 = 64 = 0x40
    // offset to signature = 64 + tuple_size
    //   tuple_size = 4 words (nonce, deadline, trader, arr_offset)
    //              + 1 word (array length)
    //              + 2*N words (array data)
    //              = (5 + 2*N) * 32
    let tuple_words = 5 + 2 * n;
    let offset_request = 64_usize;
    let offset_signature = 64 + tuple_words * 32;

    put_usize(out, 4, offset_request);
    put_usize(out, 36, offset_signature);

    // --- Request tuple (at offset 4 + 64 = 68) ---
    let tuple_base = 4 + offset_request;
    // nonce
    put_word(out, tuple_base, &request.nonce.0);
    // deadline
    put_word(out, tuple_base + 32, &request.deadline.0);
    // trader
    put_addr(out, tuple_base + 64, request.trader);
    // offset to positionUpdates (relative to start of tuple) = 4 * 32 = 128
    put_usize(out, tuple_base + 96, 128);

    // --- positionUpdates array (at tuple_base + 128) ---
    let arr_base = tuple_base + 128;
    put_usize(out, arr_base, n);
    for (i, entry) in request.position_updates.iter().enumerate() {
        let base = arr_base + 32 + i * 64;
        put_addr(out, base, entry.token);
        put_word(out, base + 32, &entry.amount.0);
    }

    // --- Signature bytes (at 4 + offset_signature) ---
    let sig_base = 4 + offset_signature;
    put_usize(out, sig_base, signature.len());
    let sig_data_start = sig_base + 32;
    out[sig_data_start..sig_data_start + signature.len()].copy_from_slice(signature);

    Ok(required)
}

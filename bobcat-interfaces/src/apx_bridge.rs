//! Core end-user calldata for the APX / Aster Deposit Bridge on Arbitrum.
//!
//! APX Finance (now Aster, after merging with Astherus) operates an on-chain
//! custody vault — the `AstherusVault` contract — on Arbitrum One. Users deposit
//! ERC-20 tokens or native ETH into the vault to fund their Aster exchange
//! accounts; withdrawals are initiated off-chain and finalized by designated
//! protocol operators.
//!
//! This module exposes only the three end-user deposit entrypoints and two
//! read-only view helpers:
//!
//! - [`make_fn_deposit`] — `deposit(address,uint256,uint256)`: transfer ERC-20
//!   into the vault. The caller must have approved the vault to spend `amount`.
//! - [`make_fn_deposit_native`] — `depositNative(uint256)`: deposit native ETH.
//!   The caller must send the ETH amount as `msg.value`.
//! - [`make_fn_deposit_for`] — `depositFor(address,address,uint256,uint256)`:
//!   deposit ERC-20 or native ETH on behalf of another address. For native ETH,
//!   `amount` must equal `msg.value`.
//! - [`make_fn_balance`] — `balance(address)`: view the vault's ERC-20 balance
//!   for a given token.
//! - [`make_fn_fees`] — `fees(address)`: view accumulated withdraw fees for a
//!   given token.
//!
//! All withdraw, validator, pause, token-management, fee-withdrawal, and
//! upgrade functions are restricted to protocol-operator roles and are
//! intentionally excluded.
//!
//! # ABI
//!
//! Verified against the deployed `AstherusVault` implementation on Arbitrum One
//! at `0x896A363007ba67f677EB2D1e8CC00f8B61697d1A` (Sourcify exact match,
//! solc 0.8.25, verified 2026-07-03). The proxy is an ERC-1967 proxy at
//! `0x9E36CB86a159d479cEd94Fa05036f235Ac40E1d5`.
//!
//! Sources:
//! - <https://www.asterdex.com/en/docs/overview/smart-contracts>
//! - Sourcify: `0x896A363007ba67f677EB2D1e8CC00f8B61697d1A` on chain 42161

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(address,uint256,uint256)",
    SEL_DEPOSIT_NATIVE = b"depositNative(uint256)",
    SEL_DEPOSIT_FOR = b"depositFor(address,address,uint256,uint256)",
    SEL_BALANCE = b"balance(address)",
    SEL_FEES = b"fees(address)",
}

// ---------------------------------------------------------------------------
// deposit(address currency, uint256 amount, uint256 broker)
// ---------------------------------------------------------------------------

/// Encode `deposit(address,uint256,uint256)`.
///
/// This is the standard ERC-20 deposit flow: the caller transfers `amount` of
/// `currency` into the vault. The caller must have approved the vault contract
/// to spend `amount` of `currency` before sending this calldata.
///
/// `broker` is an identifier for the broker channel the deposit should be
/// credited through.
pub const fn make_fn_deposit(currency: Address, amount: &U, broker: &U) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_DEPOSIT,
        leftpad_addr(currency),
        amount.0,
        broker.0
    )
}

// ---------------------------------------------------------------------------
// depositNative(uint256 broker)
// ---------------------------------------------------------------------------

/// Encode `depositNative(uint256)`.
///
/// This is the native ETH deposit flow. The caller must send the ETH amount as
/// `msg.value`. `broker` is an identifier for the broker channel.
pub const fn make_fn_deposit_native(broker: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_DEPOSIT_NATIVE, broker.0)
}

// ---------------------------------------------------------------------------
// depositFor(address currency, address forAddress, uint256 amount, uint256 broker)
// ---------------------------------------------------------------------------

/// Encode `depositFor(address,address,uint256,uint256)`.
///
/// Deposit `amount` of `currency` on behalf of `for_address`. For ERC-20
/// tokens, the caller must have approved the vault to spend `amount`. For
/// native ETH, `amount` must equal `msg.value`.
///
/// `broker` is an identifier for the broker channel.
pub const fn make_fn_deposit_for(
    currency: Address,
    for_address: Address,
    amount: &U,
    broker: &U,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT_FOR,
        leftpad_addr(currency),
        leftpad_addr(for_address),
        amount.0,
        broker.0
    )
}

// ---------------------------------------------------------------------------
// balance(address currency) — view
// ---------------------------------------------------------------------------

/// Encode `balance(address)`.
///
/// View function returning the vault's ERC-20 balance for `currency`.
pub const fn make_fn_balance(currency: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_BALANCE, leftpad_addr(currency))
}

// ---------------------------------------------------------------------------
// fees(address) — view
// ---------------------------------------------------------------------------

/// Encode `fees(address)`.
///
/// View function returning the accumulated withdraw fees for `currency`.
pub const fn make_fn_fees(currency: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_FEES, leftpad_addr(currency))
}

//! Calldata builders for BlackRock BUIDL (BlackRock USD Institutional Digital
//! Liquidity Fund) on Arbitrum.
//!
//! BUIDL is a Securitize-issued ERC-20 token deployed behind an EIP-1967
//! transparent proxy. The on-chain contract exposes standard ERC-20 transfer
//! and approval flows for token holders, plus a small set of view helpers.
//!
//! ## Permission constraints
//!
//! Subscription (issuance) and redemption (burn) are **not** end-user flows.
//! `issueTokens` / `issueTokensCustom` / `issueTokensWithMultipleLocks` /
//! `issueTokensWithNoCompliance` require `ROLE_ISSUER`, and `burn` /
//! `omnibusBurn` / `seize` / `omnibusSeize` require `ROLE_TRANSFER_AGENT` or
//! equivalent operator roles. These are protocol-operator actions performed by
//! Securitize as transfer agent — a regular BUIDL holder cannot call them. They
//! are intentionally NOT exposed here.
//!
//! The functions below are the core holder-facing calls: transferring BUIDL,
//! approving spenders, and querying balances/allowances/metadata. All of them
//! produce fixed-size calldata with no dynamic types, so the entire module is
//! `no_std` without `alloc`.
//!
//! ## Token
//!
//! - Name:    BlackRock USD Institutional Digital Liquidity Fund
//! - Symbol:  BUIDL
//! - Decimals: 6
//! - Proxy:   `0xA6525Ae43eDCd03dC08E775774dCAbd3bb925872`
//!
//! ABI verified against the on-chain implementation via Sourcify.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_TRANSFER = b"transfer(address,uint256)",
    SEL_TRANSFER_FROM = b"transferFrom(address,address,uint256)",
    SEL_APPROVE = b"approve(address,uint256)",
    SEL_INCREASE_APPROVAL = b"increaseApproval(address,uint256)",
    SEL_DECREASE_APPROVAL = b"decreaseApproval(address,uint256)",
    SEL_BALANCE_OF = b"balanceOf(address)",
    SEL_ALLOWANCE = b"allowance(address,address)",
    SEL_TOTAL_SUPPLY = b"totalSupply()",
    SEL_DECIMALS = b"decimals()",
    SEL_IS_PAUSED = b"isPaused()",
    SEL_PRE_TRANSFER_CHECK = b"preTransferCheck(address,address,uint256)",
    SEL_CAP = b"cap()",
    SEL_TOTAL_ISSUED = b"totalIssued()",
    SEL_SUPPORTED_FEATURES = b"supportedFeatures()",
}

// ---------------------------------------------------------------------------
// Write calls (nonpayable)
// ---------------------------------------------------------------------------

/// Encode `transfer(to, value)` — move BUIDL from the caller to `to`.
///
/// This is the primary holder-facing transfer. The contract enforces
/// Securitize compliance checks internally; a transfer can revert if either
/// party is not an approved/verified investor, or if the token is paused.
pub const fn make_fn_transfer(to: Address, value: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_TRANSFER, leftpad_addr(to), value.0)
}

/// Encode `transferFrom(from, to, value)` — move BUIDL on behalf of `from`.
///
/// The caller must have a sufficient allowance from `from`.
pub const fn make_fn_transfer_from(
    from: Address,
    to: Address,
    value: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(SEL_TRANSFER_FROM, leftpad_addr(from), leftpad_addr(to), value.0)
}

/// Encode `approve(spender, value)` — set a flat allowance for `spender`.
pub const fn make_fn_approve(spender: Address, value: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_APPROVE, leftpad_addr(spender), value.0)
}

/// Encode `increaseApproval(spender, addedValue)` — increase an allowance.
pub const fn make_fn_increase_approval(
    spender: Address,
    added_value: &U,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_INCREASE_APPROVAL, leftpad_addr(spender), added_value.0)
}

/// Encode `decreaseApproval(spender, subtractedValue)` — decrease an allowance.
pub const fn make_fn_decrease_approval(
    spender: Address,
    subtracted_value: &U,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DECREASE_APPROVAL, leftpad_addr(spender), subtracted_value.0)
}

// ---------------------------------------------------------------------------
// Read calls (view)
// ---------------------------------------------------------------------------

/// Encode `balanceOf(owner)`.
pub const fn make_fn_balance_of(owner: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_BALANCE_OF, leftpad_addr(owner))
}

/// Encode `allowance(owner, spender)`.
pub const fn make_fn_allowance(owner: Address, spender: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_ALLOWANCE, leftpad_addr(owner), leftpad_addr(spender))
}

/// Encode `totalSupply()`.
pub const fn make_fn_total_supply() -> [u8; 4] {
    SEL_TOTAL_SUPPLY
}

/// Encode `decimals()`.
pub const fn make_fn_decimals() -> [u8; 4] {
    SEL_DECIMALS
}

/// Encode `isPaused()`.
pub const fn make_fn_is_paused() -> [u8; 4] {
    SEL_IS_PAUSED
}

/// Encode `preTransferCheck(from, to, value)`.
///
/// Returns a status code and human-readable reason string, allowing a caller
/// to simulate whether a transfer would succeed without committing state.
pub const fn make_fn_pre_transfer_check(
    from: Address,
    to: Address,
    value: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_PRE_TRANSFER_CHECK,
        leftpad_addr(from),
        leftpad_addr(to),
        value.0
    )
}

/// Encode `cap()`.
pub const fn make_fn_cap() -> [u8; 4] {
    SEL_CAP
}

/// Encode `totalIssued()`.
pub const fn make_fn_total_issued() -> [u8; 4] {
    SEL_TOTAL_ISSUED
}

/// Encode `supportedFeatures()`.
pub const fn make_fn_supported_features() -> [u8; 4] {
    SEL_SUPPORTED_FEATURES
}

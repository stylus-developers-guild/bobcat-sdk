//! Calldata builders for Midas RWA issuance and redemption vaults.
//!
//! Midas is an asset-tokenisation protocol that issues yield-bearing Liquid Yield
//! Tokens (LYTs) — mTBILL, mBASIS, mBTC, mEDGE, mMEV, mRe7YIELD — backed by
//! institutional-grade real-world assets.  Each LYT is an ERC-20 deployed behind an
//! EIP-1967 transparent proxy with role-based access control
//! (`DEPOSIT_VAULT_ADMIN_ROLE`, `REDEMPTION_VAULT_ADMIN_ROLE`, greenlist / blacklist
//! operators, etc.).
//!
//! Tokens are **minted** (issued) through an *Issuance Vault* and **redeemed**
//! (burned) through a *Redemption Vault*.  Two redemption flavours exist:
//!
//! - **Instant** — the vault holds a liquidity buffer and pays out immediately,
//!   subject to a daily limit and per-token capacity.
//! - **Standard** — the request is queued and processed off-chain within 1–7
//!   business days; the user's mTokens are burned up front and the underlying is
//!   airdropped once the request is fulfilled.
//!
//! ## End-user functions exposed
//!
//! ### Issuance (mint)
//! - `depositInstant(address paymentToken, uint256 amount, uint256 minMTokenAmount, bytes32 txId)`
//! - `depositRequest(address paymentToken, uint256 amount, bytes32 txId, address receiver)`
//!
//! ### Redemption (burn)
//! - `redeemInstant(address mToken, uint256 amount, uint256 minPaymentAmount)`
//! - `redeemRequest(address mToken, uint256 amount, address receiver)`
//!
//! ## Permission constraints
//!
//! Only the four functions above are exposed.  All admin / operator calls —
//! `approveRequest`, `safeApproveRequest`, `rejectRequest`, `setMinAmount`,
//! `setInstantFee`, `changeTokenFee`, `addPaymentToken`, `removePaymentToken`,
//! `setGreenlistEnable`, `pauseFn`, `unpauseFn`, `withdrawToken`,
//! `setMaxSupplyCap`, `setFeeReceiver`, `setSanctionsList`, etc. — are
//! protocol-operator actions and are intentionally omitted.
//!
//! ## ABI source
//!
//! Selectors verified against the deployed Ethereum implementations:
//! - Issuance vault impl `0xc8af8477f3caa89f60fe9d1f48eee5433c55982b`
//! - Standard redemption vault impl `0x2f1372244cedcaf8ee1759d2f02435628f14975f`
//! - Instant redemption vault impl `0x489a797714708cf088d158714a376d8ff740d701`
//!
//! The token and vault contracts use the same ABI across all chains where Midas
//! deploys (Ethereum, Base, and other supported chains); the selectors below are
//! chain-agnostic.
//!
//! All function arguments are static ABI types (address, uint256, bytes32), so the
//! entire module is `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    // Issuance vault
    SEL_DEPOSIT_INSTANT = b"depositInstant(address,uint256,uint256,bytes32)",
    SEL_DEPOSIT_REQUEST = b"depositRequest(address,uint256,uint256,bytes32,address)",

    // Standard redemption vault
    SEL_REDEEM_REQUEST = b"redeemRequest(address,uint256,address)",

    // Instant redemption vault
    SEL_REDEEM_INSTANT = b"redeemInstant(address,uint256,uint256)",
}

// ---------------------------------------------------------------------------
// Issuance (mint)
// ---------------------------------------------------------------------------

/// Encode `depositInstant(paymentToken, amount, minMTokenAmount, txId)` for a
/// Midas issuance vault.
///
/// This is the instant issuance flow: the caller transfers `amount` of
/// `paymentToken` to the vault and receives mTokens immediately at the current
/// price, subject to a minimum output of `minMTokenAmount`.  The caller must have
/// approved the vault to spend `paymentToken` before sending this calldata.
///
/// `txId` is a caller-supplied unique identifier (typically a hash of off-chain
/// order details) used for idempotency / tracking.
pub const fn make_fn_deposit_instant(
    payment_token: Address,
    amount: &U,
    min_m_token_amount: &U,
    tx_id: [u8; 32],
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT_INSTANT,
        leftpad_addr(payment_token),
        amount.0,
        min_m_token_amount.0,
        tx_id
    )
}

/// Encode `depositRequest(paymentToken, amount, txId, receiver)` for a Midas
/// issuance vault.
///
/// This is the standard (queued) issuance flow: the caller transfers `amount` of
/// `paymentToken` to the vault and the mTokens are airdropped to `receiver` once
/// the request is processed off-chain (typically within two business days).  The
/// caller must have approved the vault to spend `paymentToken`.
///
/// `txId` is a caller-supplied unique identifier for tracking.
pub const fn make_fn_deposit_request(
    payment_token: Address,
    amount: &U,
    tx_id: [u8; 32],
    receiver: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT_REQUEST,
        leftpad_addr(payment_token),
        amount.0,
        tx_id,
        leftpad_addr(receiver)
    )
}

// ---------------------------------------------------------------------------
// Redemption (burn)
// ---------------------------------------------------------------------------

/// Encode `redeemRequest(mToken, amount, receiver)` for a Midas standard
/// redemption vault.
///
/// Burns `amount` of `mToken` from the caller and queues a redemption request.  The
/// underlying payment is sent to `receiver` once the request is processed
/// off-chain (typically within 1–7 business days).  The caller must have approved
/// the redemption vault to burn `mToken` on their behalf (or be the token holder
/// calling directly).
pub const fn make_fn_redeem_request(
    m_token: Address,
    amount: &U,
    receiver: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM_REQUEST,
        leftpad_addr(m_token),
        amount.0,
        leftpad_addr(receiver)
    )
}

/// Encode `redeemInstant(mToken, amount, minPaymentAmount)` for a Midas instant
/// redemption vault.
///
/// Burns `amount` of `mToken` from the caller and pays out the underlying
/// immediately from the vault's liquidity buffer, subject to a minimum output of
/// `minPaymentAmount`.  The payout is subject to daily limits and per-token
/// capacity; if the instant capacity is exhausted the call will revert and the
/// caller should fall back to [`make_fn_redeem_request`].
///
/// The caller must have approved the redemption vault to burn `mToken` on their
/// behalf (or be the token holder calling directly).
pub const fn make_fn_redeem_instant(
    m_token: Address,
    amount: &U,
    min_payment_amount: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM_INSTANT,
        leftpad_addr(m_token),
        amount.0,
        min_payment_amount.0
    )
}

//! Core end-user calldata builders for Toros leveraged-token and yield-token
//! mint and redeem flows.
//!
//! Toros Finance issues automated, tokenized strategy tokens — leveraged tokens,
//! 1x tokens, options strategies, and index baskets — as standard ERC-20s on
//! Arbitrum (and Ethereum / HyperEVM).  Every Toros token is a vault built on the
//! Chamber protocol (formerly dHEDGE).  At the contract level a Toros token *is* a
//! `PoolLogic` proxy: an ERC-4626-like vault whose shares are the strategy token.
//!
//! ## Mint / redeem flow
//!
//! The vault (`PoolLogic`) exposes three core end-user functions:
//!
//! - `deposit(address asset, uint256 amount)` — deposit `amount` of a supported
//!   deposit asset (e.g. USDC, WETH) and receive newly minted vault shares (the
//!   Toros token) at the current NAV.  This is the **mint** (buy) path.
//! - `withdraw(uint256 fundTokenAmount)` — burn `fundTokenAmount` of the vault
//!   token and receive a pro-rata slice of the vault's underlying basket.  This is
//!   the **redeem** (sell) path — the "underlying basket" withdrawal method.
//! - `withdrawTo(address recipient, uint256 fundTokenAmount)` — same as `withdraw`
//!   but sends the redeemed basket to `recipient` instead of `msg.sender`.
//!
//! Both `withdraw` and `withdrawTo` return the vault's *actual holdings* (USDC,
//! WETH, aUSDC, LP tokens, etc.) in their pro-rata proportions.  To receive a
//! single output token instead, the Chamber `EasySwapperV2` contract provides a
//! two-step single-asset withdrawal, but that path requires dynamic
//! `ComplexAsset[]` / swap-data arrays and therefore needs `alloc`; it is
//! intentionally **not** exposed here.
//!
//! ## Prerequisites
//!
//! The caller must approve the vault (`PoolLogic` proxy) to spend `amount` of the
//! deposit asset before calling `deposit`.  For `withdraw` / `withdrawTo` the
//! vault burns the caller's own shares, so no approval is needed — the caller
//! must simply hold `fundTokenAmount` of the vault token.
//!
//! Shares are locked for up to 24 hours after each deposit; `withdraw` and
//! `withdrawTo` revert if the caller's lockup has not elapsed.
//!
//! ## Permissioning
//!
//! `deposit`, `withdraw`, and `withdrawTo` are end-user functions callable by any
//! address holding the deposit asset (for mint) or vault shares (for redeem).
//! Pause/unpause, fee configuration, asset whitelisting, manager assignment,
//! `execTransaction`, `setPoolPrivate`, `mintManagerFee`, and
//! `depositForWithCustomCooldown` are manager / governance functions and are
//! intentionally **not** exposed here.
//!
//! ## ABI source
//!
//! Function signatures verified against the official Chamber (dHEDGE) V2 contracts
//! repository: `contracts/interfaces/IPoolLogic.sol` and `contracts/PoolLogic.sol`
//! at <https://github.com/dhedge/V2-Public>.  Toros vaults are `PoolLogic` proxies
//! and inherit these signatures unchanged.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(address,uint256)",
    SEL_WITHDRAW = b"withdraw(uint256)",
    SEL_WITHDRAW_TO = b"withdrawTo(address,uint256)",
}

// ---------------------------------------------------------------------------
// deposit — mint vault shares (buy a Toros token)
// ---------------------------------------------------------------------------

/// Encode `deposit(address asset, uint256 amount)` for a Toros / Chamber vault.
///
/// This is the canonical end-user mint flow: transfer `amount` of a supported
/// deposit asset (e.g. USDC, WETH) into the vault and receive newly minted vault
/// shares (the Toros strategy token) at the current NAV.
///
/// `asset` is the address of the deposit token.  It must be one of the vault's
/// enabled deposit assets; paying with a non-deposit asset requires a prior swap
/// (not handled here — the Toros frontend uses a DEX aggregator for that path).
/// `amount` is denominated in the deposit token's native decimals
/// (e.g. `100_000_000` for 100 USDC with 6 decimals).
///
/// The caller must have approved the vault contract to spend `amount` of `asset`
/// before sending this calldata.  The received shares are locked for up to 24
/// hours.
pub const fn make_fn_deposit(asset: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, leftpad_addr(asset), amount.0)
}

// ---------------------------------------------------------------------------
// withdraw — redeem vault shares into the underlying basket
// ---------------------------------------------------------------------------

/// Encode `withdraw(uint256 fundTokenAmount)` for a Toros / Chamber vault.
///
/// This is the canonical end-user redeem flow: burn `fundTokenAmount` of the
/// vault token and receive a pro-rata slice of the vault's current underlying
/// basket (the "underlying basket withdrawal" method).  If the vault holds
/// complex positions (LP tokens, lending positions, etc.) they are unwound
/// on-chain as part of the withdrawal.
///
/// `fundTokenAmount` is denominated in the vault token's 18 decimals.
///
/// No approval is needed — the vault burns the caller's own shares.  The shares
/// must be past the 24-hour post-deposit lockup, otherwise the call reverts.
///
/// To receive a single output token instead of the basket, use the Chamber
/// `EasySwapperV2` two-step withdrawal (requires `alloc`, not exposed here).
pub const fn make_fn_withdraw(fund_token_amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_WITHDRAW, fund_token_amount.0)
}

// ---------------------------------------------------------------------------
// withdrawTo — redeem vault shares into the basket, sent to a recipient
// ---------------------------------------------------------------------------

/// Encode `withdrawTo(address recipient, uint256 fundTokenAmount)` for a Toros /
/// Chamber vault.
///
/// Identical to [`make_fn_withdraw`] except the redeemed basket is sent to
/// `recipient` instead of `msg.sender`.  This is useful when the caller is a
/// contract (e.g. a Stylus program) that wants to forward the redeemed assets to
/// a user or another protocol.
///
/// `fund_token_amount` is denominated in the vault token's 18 decimals.  The
/// shares must be past the 24-hour lockup.
pub const fn make_fn_withdraw_to(recipient: Address, fund_token_amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW_TO, leftpad_addr(recipient), fund_token_amount.0)
}

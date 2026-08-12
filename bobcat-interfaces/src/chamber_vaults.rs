//! Calldata builders for Chamber Vaults (formerly dHEDGE) index-vault share
//! mint and redeem flows on Arbitrum.
//!
//! Chamber is a non-custodial vault protocol for creating and managing onchain
//! vaults.  Each vault is a `PoolLogic` proxy (ERC-20 share token + asset
//! custodian) paired with a `PoolManagerLogic` proxy (manager / permissions
//! surface).  Users interact with the `PoolLogic` proxy to deposit assets and
//! receive vault shares, and to burn shares and receive a pro-rata slice of
//! the vault's holdings.
//!
//! ## Mint (deposit)
//!
//! - `deposit(address asset, uint256 amount)` — transfer `amount` of a
//!   supported deposit asset (e.g. USDC, WETH) into the vault and receive newly
//!   minted vault shares at the current NAV.  This is the canonical mint path
//!   for all Chamber index vaults.
//!
//! ## Redeem (withdraw)
//!
//! - `withdraw(uint256 fundTokenAmount)` — burn `fundTokenAmount` vault shares
//!   and receive a pro-rata slice of every asset the vault currently holds
//!   (the "underlying basket" withdrawal method).  Complex positions (LP
//!   tokens, lending positions, etc.) are unwound on-chain as part of the
//!   withdrawal.
//! - `withdrawTo(address recipient, uint256 fundTokenAmount)` — same as
//!   `withdraw` but sends the redeemed basket to `recipient` instead of
//!   `msg.sender`.
//!
//! `withdrawSafe` and `withdrawToSafe` require a dynamic `ComplexAsset[]`
//! array and therefore need `alloc`; they are intentionally **not** exposed
//! here.  `depositForWithCustomCooldown` requires the caller to be on the
//! factory's `customCooldownWhitelist` and is a permissioned function; it is
//! also excluded.  All manager / governance functions (`setPoolPrivate`,
//! `mintManagerFee`, `execTransaction`, `setPoolManagerLogic`, etc.) are
//! likewise excluded.
//!
//! ## Prerequisites
//!
//! The caller must approve the vault (`PoolLogic` proxy) to spend `amount` of
//! the deposit asset before calling `deposit`.  For `withdraw` and
//! `withdrawTo` the vault burns the caller's own shares, so no approval is
//! needed — the caller must simply hold `fundTokenAmount` of the vault token.
//!
//! Shares are locked for up to 24 hours after each deposit; `withdraw` and
//! `withdrawTo` revert if the caller's lockup has not elapsed.
//!
//! ## ABI source
//!
//! Function signatures verified against the official Chamber (dHEDGE) V2
//! contracts repository: `contracts/interfaces/IPoolLogic.sol` and
//! `contracts/PoolLogic.sol` at <https://github.com/dhedge/V2-Public>.
//! The protocol documentation lives at <https://docs.chamberfi.com/>.

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
// deposit — mint vault shares
// ---------------------------------------------------------------------------

/// Encode `deposit(address asset, uint256 amount)` for a Chamber vault
/// (`PoolLogic` proxy).
///
/// This is the canonical end-user mint flow: transfer `amount` of a supported
/// deposit asset (e.g. USDC, WETH) into the vault and receive newly minted
/// vault shares at the current NAV.
///
/// `asset` is the address of the deposit token.  It must be one of the vault's
/// enabled deposit assets; the set is configured by the vault manager via
/// `PoolManagerLogic`.
/// `amount` is denominated in the deposit token's native decimals
/// (e.g. `100_000_000` for 100 USDC with 6 decimals).
///
/// The caller must have approved the vault contract to spend `amount` of
/// `asset` before sending this calldata.  The received shares are locked for
/// up to 24 hours.
pub const fn make_fn_deposit(asset: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, leftpad_addr(asset), amount.0)
}

// ---------------------------------------------------------------------------
// withdraw — redeem vault shares into the underlying basket
// ---------------------------------------------------------------------------

/// Encode `withdraw(uint256 fundTokenAmount)` for a Chamber vault.
///
/// This is the canonical end-user redeem flow: burn `fund_token_amount` of the
/// vault token and receive a pro-rata slice of the vault's current underlying
/// basket (the "underlying basket" withdrawal method).  If the vault holds
/// complex positions (LP tokens, lending positions, etc.) they are unwound
/// on-chain as part of the withdrawal.
///
/// `fund_token_amount` is denominated in the vault token's 18 decimals.
///
/// No approval is needed — the vault burns the caller's own shares.  The
/// shares must be past the 24-hour post-deposit lockup, otherwise the call
/// reverts.
///
/// To receive a single output token instead of the basket, use the Chamber
/// `EasySwapperV2` two-step withdrawal (requires `alloc`, not exposed here).
pub const fn make_fn_withdraw(fund_token_amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_WITHDRAW, fund_token_amount.0)
}

// ---------------------------------------------------------------------------
// withdrawTo — redeem vault shares, sent to a recipient
// ---------------------------------------------------------------------------

/// Encode `withdrawTo(address recipient, uint256 fundTokenAmount)` for a
/// Chamber vault.
///
/// Identical to [`make_fn_withdraw`] except the redeemed basket is sent to
/// `recipient` instead of `msg.sender`.  This is useful when the caller is a
/// contract (e.g. a Stylus program) that wants to forward the redeemed assets
/// to a user or another protocol.
///
/// `fund_token_amount` is denominated in the vault token's 18 decimals.  The
/// shares must be past the 24-hour lockup.
pub const fn make_fn_withdraw_to(
    recipient: Address,
    fund_token_amount: &U,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW_TO, leftpad_addr(recipient), fund_token_amount.0)
}

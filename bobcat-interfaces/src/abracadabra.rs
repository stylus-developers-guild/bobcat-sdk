//! Calldata builders for Abracadabra cauldron core lending flows on Arbitrum.
//!
//! Abracadabra is a lending protocol that allows users to borrow MIM (Magic
//! Internet Money) against collateral that is deposited into a BentoBox vault.
//! Each cauldron is a clone of a master contract parameterised with a specific
//! collateral type and oracle.  Users interact with cauldron clones (not the
//! master contract) to manage their loan positions.
//!
//! The collateral is held inside the BentoBox as *shares*, not raw token
//! amounts.  Consequently every amount parameter in the cauldron interface is
//! denominated in BentoBox *share* units (for collateral) or in MIM's 18-decimal
//! amount units (for borrowing / repaying).  Callers are responsible for
//! converting between raw token amounts and BentoBox shares before building
//! the calldata; the BentoBox `toShare` / `toAmount` view functions serve this
//! purpose.
//!
//! ## End-user functions exposed
//!
//! - `addCollateral(address to, bool skim, uint256 share)` — deposits
//!   `share` BentoBox collateral shares into the cauldron, crediting them to
//!   `to`.
//! - `removeCollateral(address to, uint256 share)` — withdraws `share` collateral
//!   shares from the caller's position and transfers them to `to`.  The
//!   position must remain solvent after removal.
//! - `borrow(address to, uint256 amount)` — borrows `amount` MIM (18 decimals)
//!   and transfers it to `to`, increasing the caller's borrow part.  A flat
//!   opening fee is added to the borrow part.
//! - `repay(address to, bool skim, uint256 part)` — repays `part` of `to`'s
//!   borrow position using funds from `msg.sender` (or skimmed from the
//!   BentoBox if `skim` is true).
//!
//! ## Permission constraints
//!
//! Only the four core end-user functions are exposed.  Administration
//! (`setFeeTo`, `changeInterestRate`, `changeBorrowLimit`,
//! `setBlacklistedCallee`, `reduceSupply`), liquidation (`liquidate`), the
//! batch `cook` entrypoint, `withdrawFees`, `accrue`, `updateExchangeRate`, and
//! `init` are intentionally omitted.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official Abracadabra
//! contract source at
//! <https://github.com/Abracadabra-money/abracadabra-money-contracts>
//! (`src/interfaces/ICauldronV2.sol`, `ICauldronV3.sol`, `ICauldronV4.sol`,
//! `src/cauldrons/CauldronV4.sol`).  The interface is identical across
//! CauldronV2–V4 for the four core functions exposed here; V3 and V4 extend
//! V2 with admin and liquidation helpers only.
//!
//! All function arguments are static ABI types (address, bool, uint256), so the
//! entire module is `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_bool};
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_ADD_COLLATERAL = b"addCollateral(address,bool,uint256)",
    SEL_REMOVE_COLLATERAL = b"removeCollateral(address,uint256)",
    SEL_BORROW = b"borrow(address,uint256)",
    SEL_REPAY = b"repay(address,bool,uint256)",
}

// ---------------------------------------------------------------------------
// addCollateral — deposit collateral shares into the cauldron
// ---------------------------------------------------------------------------

/// Encode `addCollateral(to, skim, share)` for an Abracadabra cauldron.
///
/// Deposits `share` BentoBox collateral shares into the cauldron and credits
/// them to `to`.  When `skim` is `true` the shares are taken from the
/// cauldron's own BentoBox deposit balance (i.e. tokens previously transferred
/// directly to the cauldron); when `false` they are pulled from `msg.sender`'s
/// BentoBox balance.  In both cases the caller must ensure the shares are
/// available.
///
/// `to` is the address whose collateral position is credited.
/// `skim` controls whether the shares are skimmed from the cauldron or pulled
/// from the caller.
/// `share` is the amount of BentoBox collateral *shares* (not raw token amounts)
/// to deposit.  Convert raw amounts to shares via the BentoBox `toShare` view.
pub const fn make_fn_add_collateral(
    to: Address,
    skim: bool,
    share: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_ADD_COLLATERAL,
        leftpad_addr(to),
        leftpad_bool(skim),
        share.0
    )
}

// ---------------------------------------------------------------------------
// removeCollateral — withdraw collateral shares from the cauldron
// ---------------------------------------------------------------------------

/// Encode `removeCollateral(to, share)` for an Abracadabra cauldron.
///
/// Withdraws `share` BentoBox collateral shares from the caller's position and
/// transfers them to `to`.  The caller's position must remain solvent after
/// removal; the cauldron reverts otherwise.  Interest is accrued before the
/// solvency check, so the caller should ensure the position has sufficient
/// collateral at the current exchange rate.
///
/// `to` is the address that receives the withdrawn collateral shares.
/// `share` is the amount of BentoBox collateral *shares* to remove.  Convert
/// raw amounts to shares via the BentoBox `toShare` view.
pub const fn make_fn_remove_collateral(to: Address, share: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_REMOVE_COLLATERAL, leftpad_addr(to), share.0)
}

// ---------------------------------------------------------------------------
// borrow — borrow MIM from the cauldron
// ---------------------------------------------------------------------------

/// Encode `borrow(to, amount)` for an Abracadabra cauldron.
///
/// Borrows `amount` MIM (18 decimals) and transfers it to `to`, increasing the
/// caller's borrow part.  A flat opening fee (configured per-cauldron via
/// `BORROW_OPENING_FEE`) is added to the borrow part.  The caller's position
/// must remain solvent after the borrow; the cauldron reverts otherwise.
/// Interest is accrued before the borrow.
///
/// `to` is the address that receives the borrowed MIM (in BentoBox shares).
/// `amount` is the raw MIM amount (18 decimals) to borrow, *not* a share
/// amount — the cauldron internally converts this to a BentoBox share amount
/// for the transfer.
pub const fn make_fn_borrow(to: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_BORROW, leftpad_addr(to), amount.0)
}

// ---------------------------------------------------------------------------
// repay — repay borrowed MIM to the cauldron
// ---------------------------------------------------------------------------

/// Encode `repay(to, skim, part)` for an Abracadabra cauldron.
///
/// Repays `part` of `to`'s borrow position.  When `skim` is `true` the MIM is
/// taken from the cauldron's own BentoBox deposit balance; when `false` it is
/// pulled from `msg.sender`'s BentoBox balance.  The repaid `part` is a debt
/// share unit (from `userBorrowPart`), not a raw MIM amount — the cauldron
/// calculates the corresponding MIM amount internally.  Interest is accrued
/// before the repayment.
///
/// `to` is the address whose borrow position is reduced.
/// `skim` controls whether the MIM is skimmed from the cauldron or pulled from
/// the caller.
/// `part` is the borrow-part amount to repay, as returned by `userBorrowPart`.
pub const fn make_fn_repay(
    to: Address,
    skim: bool,
    part: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REPAY,
        leftpad_addr(to),
        leftpad_bool(skim),
        part.0
    )
}

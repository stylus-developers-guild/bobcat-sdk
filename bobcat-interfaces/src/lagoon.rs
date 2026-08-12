//! Narrow Lagoon vault-flow calldata builders for Arbitrum.
//!
//! Lagoon is an ERC-7540 asynchronous vault protocol.  Vaults cycle through
//! epochs: users submit *requests* that are queued and later settled at a
//! single valuation point by the vault operator.  When the vault permits
//! synchronous operations, the standard ERC-4626 `deposit` / `redeem` entry
//! points are also available.
//!
//! This module exposes only the core end-user calls:
//!
//!  | Flow               | Async (ERC-7540)              | Sync (ERC-4626)                    |
//!  |--------------------|-------------------------------|------------------------------------|
//!  | Deposit into vault  | `requestDeposit`              | `deposit`                          |
//!  | Redeem from vault   | `requestRedeem`               | `redeem`                            |
//!  | Cancel pending      | `cancelRequestDeposit`         | —                                  |
//!  | Cancel pending      | `cancelRequestRedeem`          | —                                  |
//!
//! `requestDeposit` is overloaded: a 3-arg canonical form and a 4-arg form
//! that accepts a `referral` address.  `deposit` is also overloaded with a
//! 2-arg form (`receiver` = `msg.sender`) and a 3-arg form that takes an
//! explicit `controller`.  `cancelRequestDeposit` has a 0-arg form (cancels
//! for `msg.sender`) and a 1-arg form (cancels for a specific `controller`).
//!
//! Permissioned admin / operator functions (`settleDeposit`, `settleRedeem`,
//! `updateRates`, `updateSafe`, `pause`, whitelist/blacklist management,
//! `activateAsyncOnly`, `setSyncMode`, ownership transfers, etc.) are
//! intentionally excluded.
//!
//! ABI reference: official `@lagoon-protocol/v0-core` v0.6.0 ABI, published at
//! <https://docs.lagoon.finance/developer-hub/vault-abis>.  ERC-7540 base
//! interface cross-checked against
//! <https://github.com/hemilabs/viem-erc7540>.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    // ERC-7540 asynchronous deposit / redeem requests
    SEL_REQUEST_DEPOSIT = b"requestDeposit(uint256,address,address)",
    SEL_REQUEST_DEPOSIT_REFERRAL = b"requestDeposit(uint256,address,address,address)",
    SEL_REQUEST_REDEEM = b"requestRedeem(uint256,address,address)",

    // ERC-4626 synchronous deposit / redeem (inherited by ERC-7540)
    SEL_DEPOSIT = b"deposit(uint256,address)",
    SEL_DEPOSIT_CONTROLLER = b"deposit(uint256,address,address)",
    SEL_REDEEM = b"redeem(uint256,address,address)",

    // Cancel pending async requests
    SEL_CANCEL_REQUEST_DEPOSIT_SELF = b"cancelRequestDeposit()",
    SEL_CANCEL_REQUEST_DEPOSIT = b"cancelRequestDeposit(address)",
    SEL_CANCEL_REQUEST_REDEEM = b"cancelRequestRedeem(address)",
}

// ---------------------------------------------------------------------------
// Async deposit (ERC-7540)
// ---------------------------------------------------------------------------

/// Encode `requestDeposit(assets, controller, owner)` on a Lagoon vault.
///
/// Transfers `assets` of the underlying token from `owner` into the vault's
/// pending silo and queues an asynchronous deposit request.  The caller must
/// have approved the vault to spend `assets` of the underlying token.  When
/// the caller is not `owner`, the caller must be set as an operator for
/// `controller` via `setOperator`.  This function is `payable` — the vault
/// accepts native ETH for wrapped-native vaults.
pub const fn make_fn_request_deposit(
    assets: &U,
    controller: Address,
    owner: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REQUEST_DEPOSIT,
        assets.0,
        leftpad_addr(controller),
        leftpad_addr(owner)
    )
}

/// Encode `requestDeposit(assets, controller, owner, referral)` on a Lagoon
/// vault.
///
/// As [`make_fn_request_deposit`] but with an additional `referral` address
/// that gets credited for the deposit.  Useful for vaults that have referral
/// programs enabled.
pub const fn make_fn_request_deposit_with_referral(
    assets: &U,
    controller: Address,
    owner: Address,
    referral: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_REQUEST_DEPOSIT_REFERRAL,
        assets.0,
        leftpad_addr(controller),
        leftpad_addr(owner),
        leftpad_addr(referral)
    )
}

// ---------------------------------------------------------------------------
// Async redeem (ERC-7540)
// ---------------------------------------------------------------------------

/// Encode `requestRedeem(shares, controller, owner)` on a Lagoon vault.
///
/// Burns `shares` from `owner` and queues an asynchronous redemption request.
/// The shares are locked in the vault's pending silo until the operator
/// settles the epoch, at which point the corresponding underlying assets
/// become claimable.  When the caller is not `owner`, the caller must be set
/// as an operator for `controller` via `setOperator`.
pub const fn make_fn_request_redeem(
    shares: &U,
    controller: Address,
    owner: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REQUEST_REDEEM,
        shares.0,
        leftpad_addr(controller),
        leftpad_addr(owner)
    )
}

// ---------------------------------------------------------------------------
// Sync deposit (ERC-4626, available when vault is in sync mode)
// ---------------------------------------------------------------------------

/// Encode `deposit(assets, receiver)` on a Lagoon vault.
///
/// Synchronous deposit: mints vault shares to `receiver` in exchange for
/// `assets` of the underlying token.  The caller must have approved the
/// vault to spend `assets`.  Only available when the vault's `SyncMode`
/// permits synchronous operations.
pub const fn make_fn_deposit(assets: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, assets.0, leftpad_addr(receiver))
}

/// Encode `deposit(assets, receiver, controller)` on a Lagoon vault.
///
/// Synchronous deposit with an explicit `controller`.  The shares are minted
/// to `receiver` but the deposit is attributed to `controller`'s position.
/// When the caller is not `controller`, the caller must be an approved
/// operator.  Only available when the vault's `SyncMode` permits synchronous
/// operations.
pub const fn make_fn_deposit_with_controller(
    assets: &U,
    receiver: Address,
    controller: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_DEPOSIT_CONTROLLER,
        assets.0,
        leftpad_addr(receiver),
        leftpad_addr(controller)
    )
}

// ---------------------------------------------------------------------------
// Sync redeem (ERC-4626, available when vault is in sync mode)
// ---------------------------------------------------------------------------

/// Encode `redeem(shares, receiver, controller)` on a Lagoon vault.
///
/// Synchronous redeem: burns `shares` from `controller` and sends the
/// corresponding underlying assets to `receiver`.  When the caller is not
/// `controller`, the vault must have received an ERC-20 approval to burn the
/// shares on the caller's behalf.  Only available when the vault's
/// `SyncMode` permits synchronous operations.
pub const fn make_fn_redeem(
    shares: &U,
    receiver: Address,
    controller: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM,
        shares.0,
        leftpad_addr(receiver),
        leftpad_addr(controller)
    )
}

// ---------------------------------------------------------------------------
// Cancel pending requests
// ---------------------------------------------------------------------------

/// Encode `cancelRequestDeposit()` on a Lagoon vault.
///
/// Cancels the pending deposit request for `msg.sender` as controller.  The
/// request must still be in the pending (not yet settled) state.  The
/// previously transferred assets are returned to the owner.
pub const fn make_fn_cancel_request_deposit_self() -> [u8; 4] {
    SEL_CANCEL_REQUEST_DEPOSIT_SELF
}

/// Encode `cancelRequestDeposit(controller)` on a Lagoon vault.
///
/// Cancels the pending deposit request for the specified `controller`.  The
/// caller must be an approved operator for `controller`.  The previously
/// transferred assets are returned to the owner.
pub const fn make_fn_cancel_request_deposit(controller: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_CANCEL_REQUEST_DEPOSIT, leftpad_addr(controller))
}

/// Encode `cancelRequestRedeem(controller)` on a Lagoon vault.
///
/// Cancels the pending redeem request for the specified `controller`.  The
/// caller must be an approved operator for `controller`.  The previously
/// locked shares are returned to the owner.
pub const fn make_fn_cancel_request_redeem(controller: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_CANCEL_REQUEST_REDEEM, leftpad_addr(controller))
}

//! Core end-user calldata builders for D2 Finance strategy vaults.
//!
//! D2 strategy vaults expose the ERC-4626 `deposit` and `redeem` entrypoints.
//! The caller must approve the vault to spend the underlying asset before a
//! deposit. Vault-specific funding windows, caps, allowlists, and active
//! strategy periods can still cause either call to revert.
//!
//! This module intentionally excludes vault administration and strategy
//! operator functions.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(uint256,address)",
    SEL_REDEEM = b"redeem(uint256,address,address)",
}

/// Encode `deposit(assets, receiver)` for a D2 strategy vault.
///
/// `assets` is denominated in the vault's underlying token. The vault returns
/// the number of shares minted to `receiver`.
pub const fn make_fn_deposit(assets: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, assets.0, leftpad_addr(receiver))
}

/// Encode `redeem(shares, receiver, owner)` for a D2 strategy vault.
///
/// The vault burns `shares` from `owner` and sends the resulting underlying
/// assets to `receiver`. When the caller is not `owner`, the caller needs
/// sufficient share allowance.
pub const fn make_fn_redeem(shares: &U, receiver: Address, owner: Address) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM,
        shares.0,
        leftpad_addr(receiver),
        leftpad_addr(owner)
    )
}

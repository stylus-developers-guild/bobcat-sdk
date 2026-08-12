//! Core end-user calldata builders for Steakhouse Financial curated vaults on
//! Arbitrum.
//!
//! Steakhouse Financial is a risk curator that manages ERC-4626 vaults built on
//! Morpho (MetaMorpho V1 and Vault V2).  Both vault flavours implement the
//! standard ERC-4626 interface, so the deposit and redeem flows use the same
//! canonical ABI signatures regardless of the underlying vault generation.
//!
//! This module exposes only the two core end-user calls:
//!   - `deposit(uint256 assets, address receiver)`
//!   - `redeem(uint256 shares, address receiver, address owner)`
//!
//! Permissioned curator, allocator, guardian, timelock, and adapter-management
//! functions are intentionally omitted.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(uint256,address)",
    SEL_REDEEM = b"redeem(uint256,address,address)",
}

/// Encode `deposit(assets, receiver)` for a Steakhouse-curated ERC-4626 vault.
///
/// The caller must have approved the vault to spend `assets` of the underlying
/// token before sending this calldata.
pub const fn make_fn_deposit(assets: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, assets.0, leftpad_addr(receiver))
}

/// Encode `redeem(shares, receiver, owner)` for a Steakhouse-curated ERC-4626
/// vault.
///
/// Burns `shares` from `owner` and sends the corresponding underlying assets
/// to `receiver`.  When the caller is not the `owner`, the vault must have
/// received an ERC-20 approval to burn the shares on the caller's behalf.
pub const fn make_fn_redeem(shares: &U, receiver: Address, owner: Address) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM,
        shares.0,
        leftpad_addr(receiver),
        leftpad_addr(owner)
    )
}

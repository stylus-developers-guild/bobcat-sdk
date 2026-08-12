//! Narrow Beefy vault calldata builders for Arbitrum.
//!
//! Beefy Finance runs single-asset yield-compounding vaults on Arbitrum. The
//! canonical vault implementations (`BeefyVaultV6`, `BeefyVaultV7`) and the
//! unified `IVault` interface expose the same two end-user entrypoints:
//!
//! - `deposit(uint256 _amount)` — transfers `_amount` of the vault's `want()`
//!   token from `msg.sender` (requires prior ERC-20 approval) and mints vault
//!   shares to `msg.sender`.
//! - `withdraw(uint256 _shares)` — burns `_shares` of `msg.sender`'s vault
//!   tokens and returns the proportional amount of `want()`.
//!
//! `depositAll()` / `withdrawAll()` convenience wrappers, `earn()`,
//! `upgradeStrat()`, `inCaseTokensGetStuck()`, and all strategy/keeper/admin
//! functions are permissioned or operator-only and are intentionally excluded.
//!
//! ABI reference: official `IVault` interface at
//! <https://github.com/beefyfinance/beefy-contracts/blob/master/contracts/BIFI/interfaces/beefy/IVault.sol>
//! and the V7 implementation at
//! <https://github.com/beefyfinance/beefy-contracts/blob/master/contracts/BIFI/vaults/BeefyVaultV7.sol>.

use array_concat::concat_arrays;
use bobcat_maths::U;

use crate::selectors;

selectors! {
    SEL_DEPOSIT = b"deposit(uint256)",
    SEL_WITHDRAW = b"withdraw(uint256)",
}

/// Encode `deposit(_amount)` for a Beefy single-asset vault.
///
/// Send this calldata to the vault contract after approving it to spend
/// `_amount` of the vault's `want()` token. The vault mints shares to
/// `msg.sender` proportional to the deposited amount and the current
/// `getPricePerFullShare()`.
pub const fn make_fn_deposit(amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_DEPOSIT, amount.0)
}

/// Encode `withdraw(_shares)` for a Beefy single-asset vault.
///
/// Burns `_shares` of the caller's vault tokens and transfers back the
/// proportional share of the underlying `want()` token. `_shares` is denominated
/// in the vault's own ERC-20 balance (moo tokens), not in `want()`.
pub const fn make_fn_withdraw(shares: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_WITHDRAW, shares.0)
}

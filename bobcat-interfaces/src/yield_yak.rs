//! Yield Yak auto-compounding strategy calldata builders for Arbitrum.
//!
//! Yield Yak operates auto-compounding yield farming strategies across
//! Avalanche and Arbitrum. Each strategy is an ERC-20 receipt-token vault
//! (`YakStrategy`) that accepts a single `depositToken`, deploys the
//! deposited tokens into an underlying protocol, and auto-compounds rewards
//! via keeper-called `reinvest()`. Depositors receive Yak Receipt Tokens
//! (YRT) proportional to their share of the pool.
//!
//! The `YakStrategy` base contract (inherited by all concrete strategies)
//! exposes two core end-user entrypoints:
//!
//! - `deposit(uint256 amount)` — transfers `amount` of the strategy's
//!   `depositToken` from `msg.sender` (requires prior ERC-20 approval) and
//!   mints YRT shares to `msg.sender`.
//! - `withdraw(uint256 amount)` — burns `amount` YRT shares from
//!   `msg.sender` and returns the proportional share of `depositToken`.
//!
//! `depositFor(address,uint256)`, `depositWithPermit(uint256,uint256,uint8,bytes32,bytes32)`,
//! `reinvest()`, `rescueDeployedFunds()`, and all configuration/admin
//! functions (`disableDeposits`, `updateDevFee`, `recoverERC20`, etc.) are
//! either operator/keeper-only or convenience wrappers and are intentionally
//! excluded from this module.
//!
//! ABI source: official `YakStrategy` abstract contract at
//! <https://github.com/yieldyak/smart-contracts/blob/master/contracts/YakStrategy.sol>
//! and the `BaseStrategy` implementation at
//! <https://github.com/yieldyak/smart-contracts/blob/master/contracts/strategies/BaseStrategy.sol>.
//!
//! Integration reference:
//! <https://docs.yieldyak.com/for-developers/integrations>

use array_concat::concat_arrays;
use bobcat_maths::U;

use crate::selectors;

selectors! {
    SEL_DEPOSIT = b"deposit(uint256)",
    SEL_WITHDRAW = b"withdraw(uint256)",
}

/// Encode `deposit(amount)` for a Yield Yak auto-compounding strategy.
///
/// Send this calldata to the strategy contract after approving it to spend
/// `amount` of the strategy's `depositToken`. The strategy transfers the
/// tokens from `msg.sender` and mints Yak Receipt Tokens (YRT) proportional
/// to the deposited amount and the current share price
/// (`getDepositTokensForShares` / `getSharesForDepositTokens`).
pub const fn make_fn_deposit(amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_DEPOSIT, amount.0)
}

/// Encode `withdraw(amount)` for a Yield Yak auto-compounding strategy.
///
/// Burns `amount` YRT shares from `msg.sender` and transfers back the
/// proportional share of the underlying `depositToken`. `amount` is
/// denominated in the strategy's own ERC-20 receipt tokens (YRT), not in
/// deposit tokens; convert using `getDepositTokensForShares(amount)` if
/// you need to withdraw a specific deposit-token value.
pub const fn make_fn_withdraw(shares: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_WITHDRAW, shares.0)
}

//! OmniYield vault calldata builders for direct deposit, withdraw, and claim flows.
//!
//! OmniYield is an omnichain yield optimizer that operates on a hub-and-spoke model with
//! Arbitrum as the central hub. Users deposit a single asset into an OmniYield vault and the
//! protocol automatically routes funds to the highest-yield strategies across chains.
//!
//! The OmniVault contract on Arbitrum exposes three core end-user entrypoints for
//! same-chain (hub) interaction:
//!
//! - `depositDirect(address token, uint256 amount)` — transfers `amount` of `token` from
//!   `msg.sender` into the vault (requires prior ERC-20 approval) and credits the caller
//!   with vault shares at the current share price.
//! - `withdrawDirect(address token, uint256 shares)` — burns `shares` of the caller's
//!   vault position and queues a withdrawal. The withdrawn assets become claimable after
//!   the vault processes the withdrawal queue.
//! - `claimDirect(address token)` — claims any pending withdrawal that has been processed
//!   and transfers the underlying assets to the caller.
//!
//! Cross-chain deposit/withdraw/claim flows are routed through separate Gate contracts via
//! LayerZero messaging and require dynamic fee quoting; those flows are not covered here.
//!
//! ABI source: official OmniYield application frontend (app.omniyield.finance), verified
//! against the deployed OmniVault contract at
//! <https://arbiscan.io/address/0x167619E1A828242A49d14487DAD5D9baF371f368>.
//!
//! Flow references:
//! - <https://docs.omniyield.finance/omniyield/how-omniyield-works>
//! - <https://docs.omniyield.finance/omniyield/architecture>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT_DIRECT = b"depositDirect(address,uint256)",
    SEL_WITHDRAW_DIRECT = b"withdrawDirect(address,uint256)",
    SEL_CLAIM_DIRECT = b"claimDirect(address)",
}

/// Encode `depositDirect(token, amount)` for an OmniYield vault.
///
/// Send this calldata to the OmniVault contract on Arbitrum after approving it to spend
/// `amount` of `token`. The vault transfers the tokens from `msg.sender` and credits
/// shares at the current `sharePrice(token)`.
pub const fn make_fn_deposit_direct(token: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT_DIRECT, leftpad_addr(token), amount.0)
}

/// Encode `withdrawDirect(token, shares)` for an OmniYield vault.
///
/// Burns `shares` of the caller's vault position for `token` and queues a withdrawal.
/// After the vault processes the withdrawal queue, call [`make_fn_claim_direct`] to
/// receive the underlying assets. `shares` is denominated in vault share units, not
/// in the underlying token amount; convert using `sharePrice(token)` if needed.
pub const fn make_fn_withdraw_direct(token: Address, shares: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW_DIRECT, leftpad_addr(token), shares.0)
}

/// Encode `claimDirect(token)` for an OmniYield vault.
///
/// Claims any processed withdrawal that is ready for the caller. The vault transfers
/// the underlying `token` assets to `msg.sender`. Use `claimable(user, token)` to check
/// whether a claimable amount exists before calling.
pub const fn make_fn_claim_direct(token: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_CLAIM_DIRECT, leftpad_addr(token))
}

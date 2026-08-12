//! Calldata builders for Penpie staking flows on Arbitrum.
//!
//! Penpie is a SubDAO by Magpie that builds on Pendle Finance's veYield
//! system.  Users stake their Pendle LP positions (market tokens) into the
//! `PendleStaking` contract and receive receipt tokens representing their
//! staked position.  Receipt-token holders earn a share of Penpie's vePendle
//! yield (ETH rewards harvested from Pendle's fee distributor).
//!
//! To earn additional PNP token rewards, users stake their receipt tokens
//! into the `MasterPenpie` contract, which distributes PNP emissions
//! proportionally to staked amounts across registered pools.
//!
//! ## Contracts
//!
//! Two contracts are involved in the core end-user flow:
//!
//! - **PendleStaking** (TransparentUpgradeableProxy on Arbitrum) — accepts
//!   Pendle LP tokens via `depositMarket` and issues receipt tokens.  Rewards
//!   from vePendle are harvested automatically on deposit and withdrawal.
//! - **MasterPenpie** (TransparentUpgradeableProxy on Arbitrum) — accepts
//!   receipt tokens (or other registered staking tokens) via `deposit` and
//!   distributes PNP emissions.  Rewards are claimed via `multiclaim`.
//!
//! ## End-user functions exposed
//!
//! ### PendleStaking
//! - `depositMarket(address _market, address _for, address _from, uint256 _amount)` —
//!   transfers `_amount` of the Pendle LP token (`_market`) from `_from` and
//!   mints receipt tokens to `_for`.
//! - `withdrawMarket(address _market, address _for, uint256 _amount)` — burns
//!   `_amount` of receipt tokens from `_for` and returns the underlying Pendle
//!   LP to `_for`.
//!
//! ### MasterPenpie
//! - `deposit(address _stakingToken, uint256 _amount)` — stakes `_amount` of a
//!   registered staking token (typically a Penpie receipt token) and mints a
//!   MasterPenpie receipt token to `msg.sender`.
//! - `depositFor(address _stakingToken, address _for, uint256 _amount)` — same
//!   as `deposit` but mints the MasterPenpie receipt token to `_for`.
//! - `withdraw(address _stakingToken, uint256 _amount)` — burns `_amount` of
//!   the MasterPenpie receipt token and returns the staking token to
//!   `msg.sender`.
//! - `multiclaim(address[] _stakingTokens)` — claims all pending PNP and
//!   bonus-token rewards across the given staking tokens.  Only a
//!   single-token variant is exposed here (see [`make_fn_multiclaim_single`]).
//!
//! ## Permission constraints
//!
//! Only the end-user functions above are exposed.  Pool administration
//! (`add`, `set`, `removePool`), vote management, bribe operations, emergency
//! withdrawals, emission-rate configuration, pauser management, fee
//! configuration, and all other admin/operator functions are intentionally
//! omitted.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official Penpie
//! contract source at
//! <https://github.com/magpiexyz/penpie_contracts> (`PendleStakingBaseUpg.sol`,
//! `MasterPenpie.sol`, `IPendleStaking.sol`, `IMasterPenpie.sol`).
//!
//! All function arguments are static ABI types (address, uint256).  The
//! `multiclaim` builder encodes a single-element `address[]` in a fixed-size
//! buffer, so the entire module is `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    // PendleStaking
    SEL_DEPOSIT_MARKET = b"depositMarket(address,address,address,uint256)",
    SEL_WITHDRAW_MARKET = b"withdrawMarket(address,address,uint256)",

    // MasterPenpie
    SEL_DEPOSIT = b"deposit(address,uint256)",
    SEL_DEPOSIT_FOR = b"depositFor(address,address,uint256)",
    SEL_WITHDRAW = b"withdraw(address,uint256)",
    SEL_MULTICLAIM = b"multiclaim(address[])",
}

// ---------------------------------------------------------------------------
// PendleStaking — stake / unstake Pendle LP positions
// ---------------------------------------------------------------------------

/// Encode `depositMarket(market, for, from, amount)` for the Penpie
/// `PendleStaking` contract.
///
/// This is the primary staking entry point: it transfers `amount` of the
/// Pendle LP token (`market`) from `from` into the `PendleStaking` contract
/// and mints Penpie receipt tokens to `for_`.  The caller must have
/// pre-approved the `PendleStaking` contract to spend the Pendle LP token
/// (via ERC-20 `approve`).
///
/// `market` is the Pendle LP token address (the address of the Pendle market
/// pool).
/// `for_` is the address that receives the staked receipt tokens; it may
/// differ from `from` to allow staking on behalf of another user.
/// `from` is the address from which the Pendle LP tokens are pulled (typically
/// `msg.sender`).
/// `amount` is denominated in the Pendle LP token's 18 decimals.
pub const fn make_fn_deposit_market(
    market: Address,
    for_: Address,
    from: Address,
    amount: &U,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT_MARKET,
        leftpad_addr(market),
        leftpad_addr(for_),
        leftpad_addr(from),
        amount.0
    )
}

/// Encode `withdrawMarket(market, for, amount)` for the Penpie
/// `PendleStaking` contract.
///
/// Burns `amount` of the Penpie receipt token from `for_`'s position and
/// transfers the underlying Pendle LP token to `for_`.  The caller must be
/// `for_` or authorised to act on their behalf.
///
/// `market` is the Pendle LP token address.
/// `for_` is the address whose receipt tokens are burned and which receives
/// the unstaked Pendle LP tokens.
/// `amount` is denominated in the receipt token's 18 decimals.
pub const fn make_fn_withdraw_market(
    market: Address,
    for_: Address,
    amount: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_WITHDRAW_MARKET,
        leftpad_addr(market),
        leftpad_addr(for_),
        amount.0
    )
}

// ---------------------------------------------------------------------------
// MasterPenpie — stake / unstake receipt tokens for PNP rewards
// ---------------------------------------------------------------------------

/// Encode `deposit(stakingToken, amount)` for the `MasterPenpie` contract.
///
/// Stakes `amount` of a registered staking token (typically a Penpie receipt
/// token from `PendleStaking`) into `MasterPenpie` and mints a MasterPenpie
/// receipt token to `msg.sender`.  The caller earns PNP emissions
/// proportionally to their staked amount.  The caller must have approved
/// `MasterPenpie` to spend the staking token.
///
/// `staking_token` is the address of the registered staking token (e.g. a
/// Penpie receipt token).
/// `amount` is denominated in the staking token's decimals.
pub const fn make_fn_deposit(staking_token: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, leftpad_addr(staking_token), amount.0)
}

/// Encode `depositFor(stakingToken, for, amount)` for the `MasterPenpie`
/// contract.
///
/// Same as [`make_fn_deposit`] but mints the MasterPenpie receipt token to
/// `for_` instead of `msg.sender`.  The caller must have approved
/// `MasterPenpie` to spend the staking token.
///
/// `staking_token` is the address of the registered staking token.
/// `for_` is the address that receives the MasterPenpie receipt tokens.
/// `amount` is denominated in the staking token's decimals.
pub const fn make_fn_deposit_for(
    staking_token: Address,
    for_: Address,
    amount: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_DEPOSIT_FOR,
        leftpad_addr(staking_token),
        leftpad_addr(for_),
        amount.0
    )
}

/// Encode `withdraw(stakingToken, amount)` for the `MasterPenpie` contract.
///
/// Burns `amount` of the MasterPenpie receipt token from `msg.sender` and
/// returns the underlying staking token to `msg.sender`.
///
/// `staking_token` is the address of the registered staking token.
/// `amount` is denominated in the staking token's decimals.
pub const fn make_fn_withdraw(staking_token: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW, leftpad_addr(staking_token), amount.0)
}

// ---------------------------------------------------------------------------
// MasterPenpie — claim rewards
// ---------------------------------------------------------------------------

/// Calldata length for `multiclaim(address[])` with a single-element array.
///
/// Layout: selector(4) + offset(32) + array_length(32) + element_0(32)
/// = 4 + 32 * 3 = 100.
pub const MULTICLAIM_SINGLE_CALLDATA_LEN: usize = 4 + 32 * 3;

/// Encode `multiclaim(address[])` with a single staking token for the
/// `MasterPenpie` contract.
///
/// Claims all pending PNP and bonus-token rewards for `msg.sender` across the
/// given staking token.  Rewards are sent directly to `msg.sender`.
///
/// `staking_token` is the address of the registered staking token for which
/// rewards are claimed.
///
/// Only a single-token variant is provided because the Solidity function
/// accepts a dynamic `address[]` and the module is `no_std` without `alloc`.
/// To claim rewards across multiple staking tokens, call this builder once
/// per token.
pub fn make_fn_multiclaim_single(staking_token: Address) -> [u8; MULTICLAIM_SINGLE_CALLDATA_LEN] {
    let mut out = [0u8; MULTICLAIM_SINGLE_CALLDATA_LEN];
    out[..4].copy_from_slice(&SEL_MULTICLAIM);
    // offset to dynamic array data: 1 word after the selector = 32
    out[4 + 31] = 32;
    // array length: 1
    out[4 + 32 + 31] = 1;
    // element[0]: the staking token address (left-padded to 32 bytes)
    let addr_start = 4 + 32 * 2 + 12;
    out[addr_start..addr_start + 20].copy_from_slice(&staking_token);
    out
}

//! Calldata builders for Convex Finance core staking on Arbitrum.
//!
//! Convex Finance allows Curve liquidity providers to deposit their Curve LP
//! tokens and earn boosted CRV rewards (plus CVX and extra incentive tokens)
//! without locking CRV themselves.  The system consists of two contract layers
//! that an end user interacts with:
//!
//! - **Booster** — the main deposit contract.  Users call `deposit(pid, amount,
//!   stake)` to deposit Curve LP tokens into a Convex pool identified by `pid`.
//!   If `stake` is `true`, the minted Convex LP tokens (cvxLP) are automatically
//!   staked into the pool's `BaseRewardPool`.  If `stake` is `false`, the cvxLP
//!   tokens are sent directly to the caller, who can stake them manually later.
//!   Users call `withdraw(pid, amount)` to withdraw their Curve LP tokens back
//!   by burning cvxLP tokens.
//!
//! - **BaseRewardPool** — the per-pool staking reward contract.  Users who
//!   received cvxLP tokens (either auto-staked or manually staked) earn CRV,
//!   CVX, and extra incentive rewards.  `stake(amount)` stakes cvxLP tokens,
//!   `withdraw(amount, claim)` unstakes them, `withdrawAndUnwrap(amount, claim)`
//!   unstakes and unwraps back to Curve LP tokens in a single call, and
//!   `getReward()` claims all pending rewards.  `exit()` withdraws and claims
//!   everything in one transaction.
//!
//! ## Contracts
//!
//! - **Booster** on Arbitrum — the main deposit and withdrawal contract.
//! - **BaseRewardPool** — the per-pool reward staking contract.  The address for
//!   a given pool can be obtained on-chain via the Booster's `poolInfo(pid)`
//!   function (field `crvRewards`, the 4th field in the struct).
//!
//! ## End-user functions exposed
//!
//! ### Booster
//! - `deposit(uint256 pid, uint256 amount, bool stake)` — deposits `amount` of
//!   Curve LP tokens into pool `pid`; if `stake` is true, auto-stakes the minted
//!   cvxLP tokens into the pool's reward contract.
//! - `withdraw(uint256 pid, uint256 amount)` — withdraws `amount` of Curve LP
//!   tokens from pool `pid` by burning the corresponding cvxLP tokens.
//!
//! ### BaseRewardPool
//! - `stake(uint256 amount)` — stakes `amount` of cvxLP tokens to earn rewards.
//! - `stakeFor(address for_, uint256 amount)` — stakes on behalf of `for_`.
//! - `withdraw(uint256 amount, bool claim)` — unstakes `amount` of cvxLP tokens;
//!   if `claim` is true, also claims pending rewards.
//! - `withdrawAndUnwrap(uint256 amount, bool claim)` — unstakes `amount` of
//!   cvxLP tokens and unwraps them back to the underlying Curve LP tokens in a
//!   single call; if `claim` is true, also claims pending rewards.
//! - `getReward()` — claims all pending rewards (CRV, CVX, and extra tokens).
//! - `exit()` — withdraws all staked cvxLP tokens and claims all rewards.
//!
//! ## Permission constraints
//!
//! Only the end-user functions above are exposed.  Pool administration
//! (`addPool`, `setFees`, `setFactories`, etc.), earmark rewards (`earmarkRewards`),
//! vote delegation, shutdown management, and all other admin/operator functions
//! are intentionally omitted.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official Convex
//! contract source at
//! <https://github.com/convex-eth/platform> (`Booster.sol`,
//! `BaseRewardPool.sol`) and the community interface files at
//! <https://github.com/convex-community/union_contracts> (`IBooster.sol`,
//! `IBasicRewards.sol`, `IRewardStaking.sol`).
//!
//! All function arguments are static ABI types (uint256, address, bool), so the
//! entire module is `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    // Booster
    SEL_DEPOSIT = b"deposit(uint256,uint256,bool)",
    SEL_WITHDRAW = b"withdraw(uint256,uint256)",

    // BaseRewardPool
    SEL_STAKE = b"stake(uint256)",
    SEL_STAKE_FOR = b"stakeFor(address,uint256)",
    SEL_POOL_WITHDRAW = b"withdraw(uint256,bool)",
    SEL_WITHDRAW_AND_UNWRAP = b"withdrawAndUnwrap(uint256,bool)",
    SEL_GET_REWARD = b"getReward()",
    SEL_EXIT = b"exit()",
}

// ---------------------------------------------------------------------------
// Booster — deposit / withdraw Curve LP tokens
// ---------------------------------------------------------------------------

/// Encode `deposit(pid, amount, stake)` for the Convex `Booster` contract.
///
/// Deposits `amount` of Curve LP tokens into Convex pool `pid`.  The caller must
/// have pre-approved the `Booster` contract to spend the Curve LP token (via
/// ERC-20 `approve`).
///
/// If `stake` is `true`, the minted cvxLP tokens are automatically staked into
/// the pool's `BaseRewardPool` on the caller's behalf, so the caller begins
/// earning rewards immediately.  If `stake` is `false`, the cvxLP tokens are
/// sent directly to the caller, who can stake them manually via
/// [`make_fn_stake`].
///
/// `pid` is the Convex pool ID (the index into the Booster's `poolInfo` array).
/// `amount` is denominated in the Curve LP token's decimals.
pub const fn make_fn_deposit(pid: &U, amount: &U, stake: bool) -> [u8; 4 + 32 * 3] {
    let mut bool_word = [0u8; 32];
    bool_word[31] = stake as u8;
    concat_arrays!(SEL_DEPOSIT, pid.0, amount.0, bool_word)
}

/// Encode `withdraw(pid, amount)` for the Convex `Booster` contract.
///
/// Withdraws `amount` of Curve LP tokens from Convex pool `pid` by burning the
/// corresponding cvxLP tokens from the caller's balance.  If the caller's cvxLP
/// tokens are staked in the `BaseRewardPool`, they must first be unstaked (see
/// [`make_fn_pool_withdraw`] or [`make_fn_withdraw_and_unwrap`]).
///
/// `pid` is the Convex pool ID.
/// `amount` is denominated in the cvxLP token's decimals (which match the
/// Curve LP token's decimals 1:1).
pub const fn make_fn_withdraw(pid: &U, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW, pid.0, amount.0)
}

// ---------------------------------------------------------------------------
// BaseRewardPool — stake / unstake cvxLP tokens
// ---------------------------------------------------------------------------

/// Encode `stake(amount)` for the pool's `BaseRewardPool` contract.
///
/// Stakes `amount` of cvxLP tokens into the pool's reward contract.  The caller
/// must have pre-approved the `BaseRewardPool` to spend the cvxLP token.  Once
/// staked, the caller earns CRV, CVX, and any extra incentive tokens
/// proportionally to their staked amount.
///
/// `amount` is denominated in the cvxLP token's decimals.
pub const fn make_fn_stake(amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_STAKE, amount.0)
}

/// Encode `stakeFor(for_, amount)` for the pool's `BaseRewardPool` contract.
///
/// Same as [`make_fn_stake`] but stakes on behalf of `for_`, crediting the
/// staked balance to `for_` instead of `msg.sender`.  The caller must have
/// pre-approved the `BaseRewardPool` to spend the cvxLP token.
///
/// `for_` is the address that receives the staked position.
/// `amount` is denominated in the cvxLP token's decimals.
pub const fn make_fn_stake_for(for_: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_STAKE_FOR, leftpad_addr(for_), amount.0)
}

/// Encode `withdraw(amount, claim)` for the pool's `BaseRewardPool` contract.
///
/// Unstakes `amount` of cvxLP tokens and returns them to the caller.  If `claim`
/// is `true`, all pending rewards (CRV, CVX, and extra tokens) are also claimed
/// and sent to the caller.
///
/// `amount` is denominated in the cvxLP token's decimals.
/// `claim` controls whether rewards are also claimed in this call.
pub const fn make_fn_pool_withdraw(amount: &U, claim: bool) -> [u8; 4 + 32 * 2] {
    let mut bool_word = [0u8; 32];
    bool_word[31] = claim as u8;
    concat_arrays!(SEL_POOL_WITHDRAW, amount.0, bool_word)
}

/// Encode `withdrawAndUnwrap(amount, claim)` for the pool's `BaseRewardPool`
/// contract.
///
/// Unstakes `amount` of cvxLP tokens and unwraps them back to the underlying
/// Curve LP tokens in a single call — the Curve LP tokens are sent directly to
/// the caller.  If `claim` is `true`, all pending rewards are also claimed and
/// sent to the caller.
///
/// This is the most convenient way to exit a Convex staking position entirely,
/// as it combines unstaking and unwrapping into one transaction.
///
/// `amount` is denominated in the cvxLP token's decimals.
/// `claim` controls whether rewards are also claimed in this call.
pub const fn make_fn_withdraw_and_unwrap(amount: &U, claim: bool) -> [u8; 4 + 32 * 2] {
    let mut bool_word = [0u8; 32];
    bool_word[31] = claim as u8;
    concat_arrays!(SEL_WITHDRAW_AND_UNWRAP, amount.0, bool_word)
}

// ---------------------------------------------------------------------------
// BaseRewardPool — claim rewards
// ---------------------------------------------------------------------------

/// Encode `getReward()` for the pool's `BaseRewardPool` contract.
///
/// Claims all pending rewards (CRV, CVX, and any extra incentive tokens) for
/// `msg.sender`.  Rewards are sent directly to `msg.sender`.  This does not
/// affect the staked balance.
pub const fn make_fn_get_reward() -> [u8; 4] {
    SEL_GET_REWARD
}

/// Encode `exit()` for the pool's `BaseRewardPool` contract.
///
/// Withdraws all staked cvxLP tokens and claims all pending rewards in a
/// single transaction.  The cvxLP tokens are returned to `msg.sender` (they
/// are not unwrapped to Curve LP tokens; use [`make_fn_withdraw_and_unwrap`]
/// for that).  Rewards are sent to `msg.sender`.
pub const fn make_fn_exit() -> [u8; 4] {
    SEL_EXIT
}

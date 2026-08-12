//! Radpie staking calldata builders for deposit, withdraw, and reward claim flows.
//!
//! Radpie is a SubDAO by Magpie that leverages Radiant Capital's lending and
//! liquidity infrastructure on Arbitrum. Users stake supported assets (e.g.
//! WETH, USDC, ARB, RDNT) into the `RadiantStaking` contract and receive
//! receipt tokens representing their staked position. Stakers earn RDNT
//! rewards that vest over time and can be claimed once vested.
//!
//! The `RadiantStaking` contract (TransparentUpgradeableProxy) exposes three
//! core end-user entrypoints for same-chain interaction:
//!
//! - `depositAssetFor(address _asset, address _for, uint256 _assetAmount)` —
//!   transfers `_assetAmount` of `_asset` from `msg.sender` and stakes it on
//!   behalf of `_for`. The caller must have pre-approved the staking contract
//!   to spend the asset (ERC-20 `approve`). For native-asset pools the call
//!   is `payable` and `msg.value` is used instead of `transferFrom`. The
//!   staking contract mints receipt tokens to `_for`.
//! - `withdrawAssetFor(address _asset, address _for, uint256 _shares)` —
//!   burns `_shares` of the receipt token from `_for`'s position and returns
//!   the underlying `_asset` to `msg.sender`. The caller must be authorised
//!   by `_for` (via `setAuthorizedOperator`) or be `_for` themselves.
//! - `claimVestedRDNT()` — claims all vested RDNT rewards accrued to
//!   `msg.sender` from the MultiFeeDistributor. Rewards vest linearly over
//!   the lock period; unvested rewards remain locked until the next vesting
//!   epoch.
//!
//! Leveraged looping, DLP staking, batch harvesting, pool administration,
//! and protocol-operator functions are permissioned or require dynamic
//! arrays and are intentionally excluded.
//!
//! ABI source: verified `RadiantStaking` implementation on Sourcify
//! (exact match, verified 2024-12-15), deployed behind a Transparent
//! Upgradeable Proxy on Arbitrum One.
//!
//! Flow references:
//! - <https://www.radiant.magpiexyz.io/stake>
//! - <https://defillama.com/protocol/radpie>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT_ASSET_FOR = b"depositAssetFor(address,address,uint256)",
    SEL_WITHDRAW_ASSET_FOR = b"withdrawAssetFor(address,address,uint256)",
    SEL_CLAIM_VESTED_RDNT = b"claimVestedRDNT()",
}

/// Encode `depositAssetFor(asset, for, assetAmount)` for the Radpie staking contract.
///
/// Send this calldata to the `RadiantStaking` proxy on Arbitrum after approving
/// it to spend `assetAmount` of `asset` (for ERC-20 assets). For native-asset
/// pools, send `assetAmount` as `msg.value` instead. The contract transfers
/// the asset from `msg.sender` and mints receipt tokens to `for_`.
///
/// `for_` is the address that receives the staked receipt tokens; it may
/// differ from `msg.sender` to allow staking on behalf of another user.
pub const fn make_fn_deposit_asset_for(
    asset: Address,
    for_: Address,
    asset_amount: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_DEPOSIT_ASSET_FOR,
        leftpad_addr(asset),
        leftpad_addr(for_),
        asset_amount.0
    )
}

/// Encode `withdrawAssetFor(asset, for, shares)` for the Radpie staking contract.
///
/// Burns `shares` of the receipt token from `for_`'s staked position and
/// transfers the underlying `asset` to `msg.sender`. The caller must be
/// `for_` or an authorised operator set via `setAuthorizedOperator`.
/// `shares` is denominated in the receipt token's units, not the underlying
/// asset amount.
pub const fn make_fn_withdraw_asset_for(
    asset: Address,
    for_: Address,
    shares: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_WITHDRAW_ASSET_FOR,
        leftpad_addr(asset),
        leftpad_addr(for_),
        shares.0
    )
}

/// Encode `claimVestedRDNT()` for the Radpie staking contract.
///
/// Claims all RDNT rewards that have vested for `msg.sender` from the
/// MultiFeeDistributor. Vested RDNT is transferred to the caller. Rewards
/// that are still locked (not yet vested) remain in the distributor and
/// become claimable in subsequent epochs.
pub const fn make_fn_claim_vested_rdnt() -> [u8; 4] {
    SEL_CLAIM_VESTED_RDNT
}

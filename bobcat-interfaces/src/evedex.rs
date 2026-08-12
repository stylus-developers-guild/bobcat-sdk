//! Core end-user calldata builders for EVEDEX collateral deposits.
//!
//! EVEDEX is a decentralized perpetual futures exchange that uses a hybrid
//! architecture: an off-chain orderbook and matching engine for order
//! matching, with on-chain settlement on Eventum (an Arbitrum Orbit L3,
//! chainId 161803). Cross-chain deposits are routed through Arbitrum One
//! bridge contracts.
//!
//! ## On-chain vs off-chain boundary
//!
//! **Positions are submitted and cancelled off-chain.** Users sign EIP-712
//! typed data and submit orders via the EVEDEX REST API
//! (`POST /api/v2/order/limit`, `POST /api/v2/order/market`,
//! `POST /api/v2/order/stop-limit`). Cancellation is likewise off-chain
//! (`DELETE /api/order/{orderId}`, `POST /api/order/mass-cancel`). The
//! on-chain `fillOrders()` function on the EVEDEX contract is
//! `MATCHER_ROLE`-restricted — only the backend matcher calls it to settle
//! matched orders. End users never submit or cancel positions on-chain.
//!
//! **Collateral deposits are on-chain** and are the core end-user action
//! that can be encoded as calldata. Two deposit pathways exist:
//!
//! - **EHMarket** (`EHMarketV2`): deposits the protocol's primary collateral
//!   token (USDT) via `depositAsset` / `depositAssetTo`.
//! - **DepositDEX**: deposits any whitelisted collateral token via
//!   `depositCollateral` / `depositCollateralTo`.
//!
//! ## Withdrawal requests
//!
//! `DepositDEX.withdrawRequest` and `withdrawRequestCancel` are user-initiated
//! on-chain calls, but their `OrderWithdrawal` struct contains a dynamic
//! `bytes signature` field. Because this module is `no_std` without `alloc`,
//! dynamic-sized types cannot be encoded, and these functions are
//! intentionally omitted.
//!
//! ## Permissioning
//!
//! Only the four deposit functions above are exposed. All admin, operator,
//! and matcher functions — `fillOrders`, `withdrawAsset`, `withdrawComplete`,
//! role management, pause, etc. — are permissioned protocol-operator actions
//! and are intentionally not exposed here.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the deployed contracts
//! and source code in the `evedex-official` GitHub organisation:
//! - `exchange-ehmarket/contracts/EHMarketV2.sol`
//! - `exchange-contracts/contracts/DepositDEX.sol`
//!
//! All function arguments are static ABI types (address, uint256, uint112),
//! so the entire module is `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Big-endian `uint112`, used for collateral amounts in DepositDEX.
pub type Uint112 = [u8; 14];

selectors! {
    // EHMarket — primary collateral (USDT) deposits
    SEL_DEPOSIT_ASSET = b"depositAsset(uint256)",
    SEL_DEPOSIT_ASSET_TO = b"depositAssetTo(address,uint256)",

    // DepositDEX — whitelisted collateral deposits
    SEL_DEPOSIT_COLLATERAL = b"depositCollateral(address,uint112)",
    SEL_DEPOSIT_COLLATERAL_TO = b"depositCollateralTo(address,uint112,address)",
}

// ---------------------------------------------------------------------------
// EHMarket deposits
// ---------------------------------------------------------------------------

/// Encode `EHMarket.depositAsset(amount)`.
///
/// Transfers `amount` of the protocol's primary collateral token (USDT) from
/// `msg.sender` to the EHMarket contract and credits the balance to the
/// caller. This is the simplest on-chain deposit path for EVEDEX.
///
/// The caller must have approved the EHMarket contract to spend `amount` of
/// the collateral token before sending this calldata.
pub const fn make_fn_deposit_asset(amount: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_DEPOSIT_ASSET, amount.0)
}

/// Encode `EHMarket.depositAssetTo(to, amount)`.
///
/// Same as [`make_fn_deposit_asset`] but credits the deposited balance to
/// address `to` instead of `msg.sender`. The tokens are still pulled from
/// `msg.sender`, so the caller must have approved the EHMarket contract.
pub const fn make_fn_deposit_asset_to(to: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT_ASSET_TO, leftpad_addr(to), amount.0)
}

// ---------------------------------------------------------------------------
// DepositDEX deposits
// ---------------------------------------------------------------------------

/// Encode `DepositDEX.depositCollateral(collateral, amount)`.
///
/// Transfers `amount` of `collateral` from `msg.sender` to the protocol vault
/// and credits the balance to the caller. The `collateral` token must be in
/// the DepositDEX's allowed collateral list; the call reverts otherwise.
///
/// `amount` is a `uint112` (big-endian, 14 bytes) denominated in the
/// collateral token's own decimals. The caller must have approved the
/// DepositDEX contract to spend `amount` of `collateral` before sending this
/// calldata.
pub const fn make_fn_deposit_collateral(
    collateral: Address,
    amount: Uint112,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(
        SEL_DEPOSIT_COLLATERAL,
        leftpad_addr(collateral),
        leftpad_u112(amount)
    )
}

/// Encode `DepositDEX.depositCollateralTo(collateral, amount, to)`.
///
/// Same as [`make_fn_deposit_collateral`] but credits the deposited balance
/// to address `to` instead of `msg.sender`. Tokens are still pulled from
/// `msg.sender`, so the caller must have approved the DepositDEX contract.
pub const fn make_fn_deposit_collateral_to(
    collateral: Address,
    amount: Uint112,
    to: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_DEPOSIT_COLLATERAL_TO,
        leftpad_addr(collateral),
        leftpad_u112(amount),
        leftpad_addr(to)
    )
}

const fn leftpad_u112(value: Uint112) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 14], value)
}

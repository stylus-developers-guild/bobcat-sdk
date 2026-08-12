//! Narrow Enzyme Finance vault-share calldata builders for Arbitrum.
//!
//! Enzyme (formerly Melon) is an on-chain asset-management protocol. A fund
//! is a `VaultProxy` (the ERC-20 share token and asset custodian) paired with a
//! `ComptrollerProxy` (the investment / redemption logic).  Users interact with
//! the **ComptrollerProxy** to buy and redeem shares.
//!
//! ## Buy shares
//!
//! - `buyShares(uint256 _investmentAmount, uint256 _minSharesQuantity)` —
//!   deposit `_investmentAmount` of the fund's denomination asset (the vault
//!   must be approved to spend it) and mint shares to `msg.sender` at the
//!   current gross share value.  `_minSharesQuantity` is a slippage guard.
//! - `buySharesOnBehalf(address _buyer, uint256 _investmentAmount, uint256 _minSharesQuantity)` —
//!   same as `buyShares` but mints the shares to `_buyer` instead of
//!   `msg.sender`.
//!
//! ## Redeem shares
//!
//! - `redeemSharesInKind(address _recipient, uint256 _sharesQuantity, address[] _additionalAssets, address[] _assetsToSkip)` —
//!   burns `_sharesQuantity` shares and pays out a pro-rata slice of every
//!   tracked asset to `_recipient`.  `_additionalAssets` and `_assetsToSkip`
//!   are optional filters; when both are empty the redeemer receives every
//!   currently tracked asset in the vault proportionally.  This is the
//!   standard, most common redemption path.
//!
//! `redeemSharesForSpecificAssets` (which requires non-empty `address[]` and
//! `uint256[]` arrays and therefore needs `alloc` for arbitrary inputs) and
//! `buyBackProtocolFeeShares` (a permissioned buyback) are intentionally
//! **not** exposed here.  All admin / manager / migration functions
//! (`activate`, `destructActivated`, `callOnExtension`,
//! `permissionedVaultAction`, `setAutoProtocolFeeSharesBuyback`,
//! `deployGasRelayPaymaster`, etc.) are also excluded.
//!
//! ## ABI source
//!
//! Function signatures verified against the official Enzyme v4 contracts at
//! <https://github.com/enzymefinance/protocol>:
//! `contracts/release/core/fund/comptroller/IComptroller.sol`
//! (`ComptrollerProxy` / `ComptrollerLib`).

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_BUY_SHARES = b"buyShares(uint256,uint256)",
    SEL_BUY_SHARES_ON_BEHALF = b"buySharesOnBehalf(address,uint256,uint256)",
    SEL_REDEEM_SHARES_IN_KIND = b"redeemSharesInKind(address,uint256,address[],address[])",
}

// ---------------------------------------------------------------------------
// buyShares — mint vault shares (buy into the fund)
// ---------------------------------------------------------------------------

/// Encode `buyShares(uint256 _investmentAmount, uint256 _minSharesQuantity)`
/// for an Enzyme ComptrollerProxy.
///
/// The caller deposits `_investment_amount` of the fund's denomination asset
/// and receives newly minted vault shares at the current gross share value.
/// `_min_shares_quantity` is a slippage guard; pass `0` to disable it.
///
/// The caller must have approved the vault's `VaultProxy` to spend
/// `_investment_amount` of the denomination asset before sending this
/// calldata.  Shares are minted to `msg.sender`.
pub const fn make_fn_buy_shares(
    investment_amount: &U,
    min_shares_quantity: &U,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_BUY_SHARES, investment_amount.0, min_shares_quantity.0)
}

// ---------------------------------------------------------------------------
// buySharesOnBehalf — mint vault shares to a third party
// ---------------------------------------------------------------------------

/// Encode `buySharesOnBehalf(address _buyer, uint256 _investmentAmount, uint256 _minSharesQuantity)`
/// for an Enzyme ComptrollerProxy.
///
/// Identical to [`make_fn_buy_shares`] except the minted shares are credited
/// to `_buyer` instead of `msg.sender`.  The denomination asset is still
/// pulled from `msg.sender`, who must have approved the vault to spend
/// `_investment_amount`.
pub const fn make_fn_buy_shares_on_behalf(
    buyer: Address,
    investment_amount: &U,
    min_shares_quantity: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_BUY_SHARES_ON_BEHALF,
        leftpad_addr(buyer),
        investment_amount.0,
        min_shares_quantity.0
    )
}

// ---------------------------------------------------------------------------
// redeemSharesInKind — redeem shares for a pro-rata slice of all tracked assets
// ---------------------------------------------------------------------------

/// Calldata length for `redeemSharesInKind` with two empty dynamic arrays.
///
/// Layout: selector + 6 words (recipient, sharesQuantity, offset to
/// additionalAssets, offset to assetsToSkip, length of additionalAssets = 0,
/// length of assetsToSkip = 0).
pub const REDEEM_IN_KIND_CALLDATA_LEN: usize = 4 + 32 * 6;

// Offset word for the _additionalAssets array data (4 head words × 32 = 128 = 0x80).
const OFFSET_ADDITIONAL_ASSETS: [u8; 32] = concat_arrays!(
    [0u8; 31],
    [0x80]
);

// Offset word for the _assetsToSkip array data (5 head words × 32 = 160 = 0xa0).
const OFFSET_ASSETS_TO_SKIP: [u8; 32] = concat_arrays!(
    [0u8; 31],
    [0xa0]
);

// Zero-length word for empty dynamic arrays.
const ZERO_LEN: [u8; 32] = [0u8; 32];

/// Encode `redeemSharesInKind(address _recipient, uint256 _sharesQuantity, address[] _additionalAssets, address[] _assetsToSkip)`
/// with both `_additionalAssets` and `_assetsToSkip` as empty arrays.
///
/// This is the standard redemption path: burn `_shares_quantity` shares and
/// receive a pro-rata slice of every currently tracked asset in the vault,
/// sent to `_recipient`.  Passing empty arrays means "no additional assets
/// and no assets to skip" — the redeemer simply receives every asset the
/// vault currently holds, in proportion.
///
/// `_shares_quantity` is denominated in the vault share token's 18 decimals.
/// The shares are burned from `msg.sender`; the caller must hold
/// `_shares_quantity` shares.  No ERC-20 approval is required because the
/// ComptrollerProxy burns the caller's own shares directly.
///
/// Redemption variants that specify additional/skip assets or that request
/// specific payout assets require non-empty dynamic arrays and are not
/// exposed here (they need `alloc` to build at runtime).
pub const fn make_fn_redeem_shares_in_kind(
    recipient: Address,
    shares_quantity: &U,
) -> [u8; REDEEM_IN_KIND_CALLDATA_LEN] {
    concat_arrays!(
        SEL_REDEEM_SHARES_IN_KIND,
        leftpad_addr(recipient),
        shares_quantity.0,
        OFFSET_ADDITIONAL_ASSETS,
        OFFSET_ASSETS_TO_SKIP,
        ZERO_LEN,
        ZERO_LEN
    )
}

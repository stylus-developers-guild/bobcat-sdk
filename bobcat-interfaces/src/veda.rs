//! Calldata builders for Veda (BoringVault) vault flows on Arbitrum.
//!
//! Veda is a DeFi vault primitive built on the BoringVault architecture.
//! A BoringVault deployment consists of several contracts:
//!
//! - **BoringVault** — the ERC-20 share token and asset custodian.
//! - **TellerWithMultiAssetSupport** — handles user deposits and withdrawals,
//!   supporting multiple ERC-20 assets and native ETH.
//! - **BoringOnChainQueue** — handles asynchronous (queued) withdrawal
//!   requests, settled by off-chain solvers.
//! - **AccountantWithRateProviders** — provides share pricing via oracles.
//! - **ManagerWithMerkleVerification** — restricts vault strategies via a
//!   merkle tree of allowed actions.
//!
//! Users interact with the **Teller** to deposit assets and receive vault
//! shares, and with the **OnChainQueue** to request asynchronous withdrawals.
//! Direct withdrawals through the Teller are permissioned (role-gated).
//!
//! This module exposes only the core end-user calls:
//!
//!  | Flow                        | Contract           | Function                |
//!  |-----------------------------|--------------------|-------------------------|
//!  | Deposit                     | Teller             | `deposit`               |
//!  | Deposit for another        | Teller             | `deposit` (5-arg)       |
//!  | Deposit with permit         | Teller             | `depositWithPermit`     |
//!  | Withdraw (permissioned)    | Teller             | `withdraw`              |
//!  | Async withdraw request     | OnChainQueue       | `requestOnChainWithdraw`|
//!  | Cancel async withdraw      | OnChainQueue       | `cancelOnChainWithdraw` |
//!  | Replace async withdraw     | OnChainQueue       | `replaceOnChainWithdraw`|
//!
//! Permissioned admin / operator functions (`pause`, `unpause`,
//! `updateAssetData`, `setShareLockPeriod`, `denyAll`, `allowAll`,
//! `setDepositCap`, `setPermissionedTransfers`, `refundDeposit`,
//! `stopWithdrawsInAsset`, `setWithdrawCapacity`, `cancelUserWithdraws`,
//! `solveOnChainWithdraws`, `rescueTokens`, etc.) are intentionally
//! excluded.  `bulkDeposit` and `bulkWithdraw` (SOLVER_ROLE) are also
//! excluded as they are not end-user calls.
//!
//! ## ABI source
//!
//! Function signatures verified against the official Veda Labs (BoringVault)
//! contracts at <https://github.com/Veda-Labs/boring-vault>:
//! - `src/base/Roles/TellerWithMultiAssetSupport.sol`
//! - `src/base/Roles/BoringQueue/BoringOnChainQueue.sol`
//!
//! All function arguments are static ABI types (address, uint256, uint128,
//! uint96, uint40, uint24, uint16, uint8, bytes32), so the entire module is
//! `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_cd::{leftpad_u8, leftpad_u16, leftpad_u32};
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

// =========================================================================
// OnChainWithdraw struct
// =========================================================================

/// An `OnChainWithdraw` struct as used by the BoringOnChainQueue contract.
///
/// This struct is emitted in the `OnChainWithdrawRequested` event at request
/// time and must be retained off-chain (event indexer or tx receipt) to
/// cancel or replace. It is not stored as a mapping on-chain; the request ID
/// is `keccak256(abi.encode(OnChainWithdraw))`.
///
/// Fields:
/// - `nonce`: uint96 — read from state, prevents duplicate request IDs.
/// - `user`: address — the original msg.sender who created the request.
/// - `asset_out`: address — the asset to withdraw.
/// - `amount_of_shares`: uint128 — shares transferred into the queue.
/// - `amount_of_assets`: uint128 — derived from shares and price at request.
/// - `creation_time`: uint40 — block timestamp when the request was made.
/// - `seconds_to_maturity`: uint24 — time before the request matures.
/// - `seconds_to_deadline`: uint24 — time the request is valid for.
///
/// In ABI encoding, the struct is a tuple of 8 static types, so it is encoded
/// inline as 8 x 32-byte words (no offset pointer needed).
pub struct OnChainWithdraw {
    pub nonce: u128,
    pub user: Address,
    pub asset_out: Address,
    pub amount_of_shares: u128,
    pub amount_of_assets: u128,
    pub creation_time: u64,
    pub seconds_to_maturity: u32,
    pub seconds_to_deadline: u32,
}

/// Encode an `OnChainWithdraw` struct as 8 ABI words (256 bytes).
const fn encode_on_chain_withdraw(w: &OnChainWithdraw) -> [u8; 32 * 8] {
    concat_arrays!(
        // nonce (uint96) -- left-padded to 32 bytes
        U::from_u128(w.nonce).0,
        // user (address)
        leftpad_addr(w.user),
        // asset_out (address)
        leftpad_addr(w.asset_out),
        // amount_of_shares (uint128)
        U::from_u128(w.amount_of_shares).0,
        // amount_of_assets (uint128)
        U::from_u128(w.amount_of_assets).0,
        // creation_time (uint40)
        U::from_u64(w.creation_time).0,
        // seconds_to_maturity (uint24)
        leftpad_u32(w.seconds_to_maturity),
        // seconds_to_deadline (uint24)
        leftpad_u32(w.seconds_to_deadline)
    )
}

// =========================================================================
// TellerWithMultiAssetSupport selectors
// =========================================================================

selectors! {
    SEL_DEPOSIT = b"deposit(address,uint256,uint256,address)",
    SEL_DEPOSIT_FOR = b"deposit(address,uint256,uint256,address,address)",
    SEL_DEPOSIT_WITH_PERMIT = b"depositWithPermit(address,uint256,uint256,uint256,uint8,bytes32,bytes32,address)",
    SEL_WITHDRAW = b"withdraw(address,uint256,uint256,address)",
}

// -------------------------------------------------------------------------
// deposit -- deposit assets into the BoringVault
// -------------------------------------------------------------------------

/// Encode `deposit(address depositAsset, uint256 depositAmount, uint256 minimumMint, address referralAddress)`
/// for a Veda TellerWithMultiAssetSupport contract.
///
/// Transfers `deposit_amount` of `deposit_asset` from the caller into the
/// BoringVault and mints vault shares to `msg.sender`.  The caller must have
/// pre-approved the Teller to spend `deposit_asset` (via ERC-20 `approve`).
/// For native ETH deposits, send ETH as `msg.value` and pass
/// `0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE` as `deposit_asset`.
///
/// `minimum_mint` is a slippage guard; pass `0` to disable it.  Shares are
/// locked to the caller for the vault's `shareLockPeriod` after deposit.
/// `referral_address` is an optional referral; pass the zero address if unused.
pub const fn make_fn_deposit(
    deposit_asset: Address,
    deposit_amount: &U,
    minimum_mint: &U,
    referral_address: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT,
        leftpad_addr(deposit_asset),
        deposit_amount.0,
        minimum_mint.0,
        leftpad_addr(referral_address)
    )
}

// -------------------------------------------------------------------------
// deposit (5-arg) -- deposit for another address
// -------------------------------------------------------------------------

/// Encode `deposit(address depositAsset, uint256 depositAmount, uint256 minimumMint, address to, address referralAddress)`
/// for a Veda TellerWithMultiAssetSupport contract.
///
/// Same as [`make_fn_deposit`] but mints vault shares to `to` instead of
/// `msg.sender`.  This variant is role-gated and intended for router-like
/// integrations.  The deposit asset is still pulled from `msg.sender`.
pub const fn make_fn_deposit_for(
    deposit_asset: Address,
    deposit_amount: &U,
    minimum_mint: &U,
    to: Address,
    referral_address: Address,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_DEPOSIT_FOR,
        leftpad_addr(deposit_asset),
        deposit_amount.0,
        minimum_mint.0,
        leftpad_addr(to),
        leftpad_addr(referral_address)
    )
}

// -------------------------------------------------------------------------
// depositWithPermit -- deposit using ERC-2612 permit
// -------------------------------------------------------------------------

/// Encode `depositWithPermit(address depositAsset, uint256 depositAmount, uint256 minimumMint, uint256 deadline, uint8 v, bytes32 r, bytes32 s, address referralAddress)`
/// for a Veda TellerWithMultiAssetSupport contract.
///
/// Combines ERC-2612 `permit` with a deposit in a single transaction.  The
/// permit approves the BoringVault to spend `deposit_amount` of
/// `deposit_asset` on behalf of the caller.  Does not support native ETH.
///
/// `deadline` is the permit deadline (Unix timestamp).  `v`, `r`, `s` are the
/// secp256k1 signature components.  `minimum_mint` is a slippage guard.
/// `referral_address` is an optional referral; pass the zero address if unused.
pub const fn make_fn_deposit_with_permit(
    deposit_asset: Address,
    deposit_amount: &U,
    minimum_mint: &U,
    deadline: &U,
    v: u8,
    r: [u8; 32],
    s: [u8; 32],
    referral_address: Address,
) -> [u8; 4 + 32 * 8] {
    concat_arrays!(
        SEL_DEPOSIT_WITH_PERMIT,
        leftpad_addr(deposit_asset),
        deposit_amount.0,
        minimum_mint.0,
        deadline.0,
        leftpad_u8(v),
        r,
        s,
        leftpad_addr(referral_address)
    )
}

// -------------------------------------------------------------------------
// withdraw -- withdraw from the BoringVault (permissioned)
// -------------------------------------------------------------------------

/// Encode `withdraw(address withdrawAsset, uint256 shareAmount, uint256 minimumAssets, address to)`
/// for a Veda TellerWithMultiAssetSupport contract.
///
/// Burns `share_amount` vault shares from `msg.sender` and transfers the
/// corresponding amount of `withdraw_asset` to `to`.  This function is
/// permissioned (role-gated); whether it is publicly callable depends on the
/// vault's configuration.  The caller must hold `share_amount` shares.
///
/// `minimum_assets` is a slippage guard; pass `0` to disable it.
pub const fn make_fn_withdraw(
    withdraw_asset: Address,
    share_amount: &U,
    minimum_assets: &U,
    to: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_WITHDRAW,
        leftpad_addr(withdraw_asset),
        share_amount.0,
        minimum_assets.0,
        leftpad_addr(to)
    )
}

// =========================================================================
// BoringOnChainQueue selectors
// =========================================================================

selectors! {
    SEL_REQUEST_ON_CHAIN_WITHDRAW = b"requestOnChainWithdraw(address,uint128,uint16,uint24)",
    SEL_CANCEL_ON_CHAIN_WITHDRAW = b"cancelOnChainWithdraw((uint96,address,address,uint128,uint128,uint40,uint24,uint24))",
    SEL_REPLACE_ON_CHAIN_WITHDRAW = b"replaceOnChainWithdraw((uint96,address,address,uint128,uint128,uint40,uint24,uint24),uint16,uint24)",
}

// -------------------------------------------------------------------------
// requestOnChainWithdraw -- submit an async withdrawal request
// -------------------------------------------------------------------------

/// Encode `requestOnChainWithdraw(address assetOut, uint128 amountOfShares, uint16 discount, uint24 secondsToDeadline)`
/// for a Veda BoringOnChainQueue contract.
///
/// Transfers `amount_of_shares` vault shares from the caller into the queue
/// contract and creates an asynchronous withdrawal request.  The request is
/// settled later by an off-chain solver who fills the withdrawal.  The
/// caller must have approved the BoringOnChainQueue to transfer their vault
/// shares (via ERC-20 `approve` on the BoringVault token).
///
/// `asset_out` is the ERC-20 token to receive on settlement.
/// `discount` is the discount in basis points applied to the withdrawal
/// (e.g. 100 = 1% discount to the solver); must be within the asset's
/// configured `[minDiscount, maxDiscount]` range.
/// `seconds_to_deadline` is how long the request is valid for; the deadline
/// is `creationTime + secondsToMaturity + secondsToDeadline`.
///
/// The full `OnChainWithdraw` struct is emitted in the
/// `OnChainWithdrawRequested` event and must be retained off-chain to
/// cancel or replace the request later.
pub const fn make_fn_request_on_chain_withdraw(
    asset_out: Address,
    amount_of_shares: u128,
    discount: u16,
    seconds_to_deadline: u32,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_REQUEST_ON_CHAIN_WITHDRAW,
        leftpad_addr(asset_out),
        U::from_u128(amount_of_shares).0,
        leftpad_u16(discount),
        leftpad_u32(seconds_to_deadline)
    )
}

// -------------------------------------------------------------------------
// cancelOnChainWithdraw -- cancel an async withdrawal request
// -------------------------------------------------------------------------

/// Calldata length for `cancelOnChainWithdraw`: selector + 8 words for the
/// OnChainWithdraw struct.
pub const CANCEL_WITHDRAW_CALLDATA_LEN: usize = 4 + 32 * 8;

/// Encode `cancelOnChainWithdraw(OnChainWithdraw request)` for a Veda
/// BoringOnChainQueue contract.
///
/// Cancels a previously submitted asynchronous withdrawal request.  The
/// vault shares held in the queue for this request are returned to the
/// original `user`.  Only the original `user` can cancel their own request.
///
/// `request` must match the exact `OnChainWithdraw` struct that was emitted
/// in the `OnChainWithdrawRequested` event when the request was created.
/// The struct is retained off-chain (event indexer or tx receipt) and is not
/// stored as a mapping on-chain.
pub const fn make_fn_cancel_on_chain_withdraw(
    request: &OnChainWithdraw,
) -> [u8; CANCEL_WITHDRAW_CALLDATA_LEN] {
    concat_arrays!(SEL_CANCEL_ON_CHAIN_WITHDRAW, encode_on_chain_withdraw(request))
}

// -------------------------------------------------------------------------
// replaceOnChainWithdraw -- replace an async withdrawal request
// -------------------------------------------------------------------------

/// Calldata length for `replaceOnChainWithdraw`: selector + 8 words for the
/// OnChainWithdraw struct + 1 word for discount + 1 word for secondsToDeadline.
pub const REPLACE_WITHDRAW_CALLDATA_LEN: usize = 4 + 32 * 10;

/// Encode `replaceOnChainWithdraw(OnChainWithdraw oldRequest, uint16 discount, uint24 secondsToDeadline)`
/// for a Veda BoringOnChainQueue contract.
///
/// Replaces an existing asynchronous withdrawal request with a new one that
/// has different discount and deadline parameters.  The shares remain in the
/// queue; only the discount and deadline are updated.  Only the original
/// `user` can replace their own request.
///
/// `old_request` must match the exact `OnChainWithdraw` struct that was
/// emitted in the `OnChainWithdrawRequested` event when the request was
/// created.  `discount` is the new discount in basis points.  `seconds_to_deadline`
/// is the new validity duration.
pub const fn make_fn_replace_on_chain_withdraw(
    old_request: &OnChainWithdraw,
    discount: u16,
    seconds_to_deadline: u32,
) -> [u8; REPLACE_WITHDRAW_CALLDATA_LEN] {
    concat_arrays!(
        SEL_REPLACE_ON_CHAIN_WITHDRAW,
        encode_on_chain_withdraw(old_request),
        leftpad_u16(discount),
        leftpad_u32(seconds_to_deadline)
    )
}

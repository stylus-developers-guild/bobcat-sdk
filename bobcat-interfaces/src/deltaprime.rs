//! Calldata builders for DeltaPrime Prime Account core flows on Arbitrum.
//!
//! DeltaPrime is an under-collateralised lending protocol.  Borrowers interact
//! with their own **Prime Account** — a per-user diamond-proxy smart contract
//! (a "SmartLoan") that custodies collateral, manages debt, and enforces
//! solvency invariants before every state-changing action.  All user-facing
//! functions live on facets of this diamond and are invoked through the
//! diamond address.
//!
//! Assets are identified on-chain by a `bytes32` symbol (e.g. `AVAX`, `USDC`,
//! `ETH`, `PRIME`).  The `TokenManager` resolves these symbols to ERC-20
//! addresses at execution time, so the calldata builders below take a raw
//! `[u8; 32]` asset symbol.
//!
//! ## End-user functions exposed
//!
//! ### Deposit
//! - `fund(bytes32 _fundedAsset, uint256 _amount)` — transfers `_amount` of the
//!   asset identified by `_fundedAsset` from `msg.sender` into the Prime Account.
//!   The caller must have ERC-20-approved the diamond for `_amount` beforehand.
//!
//! ### Borrow
//! - `borrow(bytes32 _asset, uint256 _amount)` — borrows `_amount` of `_asset`
//!   from the protocol's lending pool.  The Prime Account must remain solvent
//!   after the borrow.
//!
//! ### Repay
//! - `repay(bytes32 _asset, uint256 _amount)` — repays `_amount` of `_asset`
//!   debt from the Prime Account's internal balance.  The function is `payable`
//!   (for native-asset repayment paths); the calldata encoding is identical.
//!
//! ### Withdraw
//! DeltaPrime uses a two-phase withdrawal-intent mechanism:
//! - `createWithdrawalIntent(bytes32 _asset, uint256 _amount)` — locks
//!   `_amount` of `_asset` behind a 24-hour timelock.
//! - `executeWithdrawalIntent(bytes32 _asset, uint256[] _intentIndices)` —
//!   after the 24-hour waiting period, executes the withdrawal by supplying
//!   the indices of the matured intents.  Multiple intents can be batched in
//!   one call.
//!
//! ## Permission constraints
//!
//! Only the five end-user functions above are exposed.  Administration,
//! liquidation (`liquidate`, `snapshotInsolvency`, `unfreezeAccount`), asset
//! management (`addOwnedAsset`, `removeUnsupportedOwnedAsset`,
//! `removeUnsupportedStakedPosition`, `withdrawUnsupportedToken`), debt-swap
//! (`swapDebtParaSwap`), leverage (`fundGLP`, `PrimeLeverageFacet`), GM/GLV
//! position management, and all view facets are intentionally omitted.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official DeltaPrime
//! contract source at
//! <https://github.com/DeltaPrimeLabs/deltaprime-contracts>
//! (`contracts/interfaces/facets/IAssetsOperationsFacet.sol` for fund/borrow/repay,
//!  `contracts/interfaces/facets/IWithdrawalIntentFacet.sol` for withdrawal intents,
//!  `contracts/facets/AssetsOperationsFacet.sol`,
//!  `contracts/facets/WithdrawalIntentFacet.sol` for implementations).
//!
//! All builders are `no_std` and allocation-free.  `fund`, `borrow`, `repay`,
//! and `createWithdrawalIntent` have fully static ABI and return fixed-size
//! arrays.  `executeWithdrawalIntent` contains a dynamic `uint256[]` parameter
//! and writes into a caller-supplied buffer.

use array_concat::concat_arrays;
use bobcat_maths::U;

use crate::selectors;

/// An EVM asset symbol as used by DeltaPrime's `TokenManager` (a `bytes32`).
pub type AssetSymbol = [u8; 32];

/// Maximum number of withdrawal intents that can be batched in one
/// `executeWithdrawalIntent` call.
pub const MAX_INTENT_INDICES: usize = 32;

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Intent-index count is zero or exceeds [`MAX_INTENT_INDICES`].
    InvalidIndexCount,
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall {
        /// Minimum buffer length needed.
        required: usize,
    },
}

selectors! {
    SEL_FUND = b"fund(bytes32,uint256)",
    SEL_BORROW = b"borrow(bytes32,uint256)",
    SEL_REPAY = b"repay(bytes32,uint256)",
    SEL_CREATE_WITHDRAWAL_INTENT = b"createWithdrawalIntent(bytes32,uint256)",
    SEL_EXECUTE_WITHDRAWAL_INTENT = b"executeWithdrawalIntent(bytes32,uint256[])",
}

// ---------------------------------------------------------------------------
// Mutable-slice helpers
// ---------------------------------------------------------------------------

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

fn put_word(output: &mut [u8], offset: usize, value: &[u8; 32]) {
    output[offset..offset + 32].copy_from_slice(value);
}

// ---------------------------------------------------------------------------
// fund — deposit collateral into the Prime Account (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a DeltaPrime `fund` call: deposit `amount` of `asset` into the
/// caller's Prime Account.
///
/// The caller must have ERC-20-approved the Prime Account (diamond) to spend
/// `amount` of the underlying token before sending this calldata.  The function
/// transfers the token from `msg.sender` into the Prime Account and emits a
/// `Funded` event.
///
/// `asset` is the DeltaPrime `bytes32` asset symbol (e.g. `USDC`, `ETH`).
/// `amount` is the raw token amount (matching the token's decimals) to deposit.
pub const fn make_fn_fund(asset: AssetSymbol, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_FUND, asset, amount.0)
}

// ---------------------------------------------------------------------------
// borrow — borrow from the lending pool (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a DeltaPrime `borrow` call: borrow `amount` of `asset` from the
/// protocol's lending pool into the Prime Account.
///
/// The Prime Account must remain solvent after the borrow; the call reverts
/// otherwise.  Interest is accrued and the debt snapshot is updated before the
/// borrow is executed.  A per-asset borrowing cooldown (one borrow per block)
/// is enforced.
///
/// `asset` is the DeltaPrime `bytes32` asset symbol to borrow.
/// `amount` is the raw token amount to borrow.
pub const fn make_fn_borrow(asset: AssetSymbol, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_BORROW, asset, amount.0)
}

// ---------------------------------------------------------------------------
// repay — repay borrowed funds (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a DeltaPrime `repay` call: repay `amount` of `asset` debt from the
/// Prime Account's internal balance.
///
/// The repayment is drawn from the Prime Account's available balance of the
/// asset (not from `msg.sender`).  The actual amount repaid is capped at both
/// the available balance and the total borrowed amount.  The function is
/// `payable` to support native-asset repayment paths, but the calldata encoding
/// is identical regardless.
///
/// `asset` is the DeltaPrime `bytes32` asset symbol to repay.
/// `amount` is the raw token amount to repay.
pub const fn make_fn_repay(asset: AssetSymbol, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_REPAY, asset, amount.0)
}

// ---------------------------------------------------------------------------
// createWithdrawalIntent — lock funds for withdrawal (fully static ABI)
// ---------------------------------------------------------------------------

/// Encode a DeltaPrime `createWithdrawalIntent` call: lock `amount` of `asset`
/// behind a 24-hour withdrawal timelock.
///
/// After creation, the intent becomes actionable after 24 hours and expires
/// 48 hours after that (72 hours from creation).  During the pending window the
/// locked amount is deducted from the Prime Account's available balance.  Once
/// the 24-hour waiting period elapses, the intent can be executed via
/// [`make_fn_execute_withdrawal_intent`].
///
/// `asset` is the DeltaPrime `bytes32` asset symbol to withdraw.
/// `amount` is the raw token amount to lock for withdrawal.
pub const fn make_fn_create_withdrawal_intent(
    asset: AssetSymbol,
    amount: &U,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_CREATE_WITHDRAWAL_INTENT, asset, amount.0)
}

// ---------------------------------------------------------------------------
// executeWithdrawalIntent — execute matured withdrawal intents
// (bytes32, uint256[])
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_execute_withdrawal_intent`].
///
/// `executeWithdrawalIntent(bytes32,uint256[])` encodes as:
/// selector(4) + asset(32) + offset(32) + length(32) + N words.
pub fn execute_withdrawal_intent_calldata_len(n: usize) -> Result<usize, EncodeError> {
    if n == 0 || n > MAX_INTENT_INDICES {
        return Err(EncodeError::InvalidIndexCount);
    }
    // selector(4) + asset(32) + offset(32) + length(32) + N*32
    4_usize
        .checked_add(32 * 3)
        .and_then(|s| s.checked_add(32 * n))
        .ok_or(EncodeError::InvalidIndexCount)
}

/// Encode a DeltaPrime `executeWithdrawalIntent` call into `output`.  Returns
/// the number of bytes written.
///
/// After the 24-hour timelock on each intent has elapsed (and before the 72-hour
/// expiry), the Prime Account owner can execute the withdrawal by supplying
/// the indices of the matured intents.  The tokens are transferred to
/// `msg.sender` and the intents are removed from storage.
///
/// `asset` is the DeltaPrime `bytes32` asset symbol being withdrawn.
/// `intent_indices` is the slice of intent indices to execute.  Indices must
/// be strictly increasing and each must reference a matured, non-expired intent.
/// The caller must supply a buffer of at least
/// [`execute_withdrawal_intent_calldata_len`]`(intent_indices.len())` bytes.
pub fn make_fn_execute_withdrawal_intent(
    output: &mut [u8],
    asset: AssetSymbol,
    intent_indices: &[U],
) -> Result<usize, EncodeError> {
    let n = intent_indices.len();
    let required = execute_withdrawal_intent_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_EXECUTE_WITHDRAWAL_INTENT);
    // head: asset, offset to uint256[]
    // offset = 2 head words * 32 = 64 = 0x40
    put_word(out, 4, &asset);
    put_usize(out, 36, 64);
    // dynamic array tail: length prefix + N index words
    let arr = 4 + 32 * 2; // 68
    put_usize(out, arr, n);
    for (i, idx) in intent_indices.iter().enumerate() {
        put_word(out, arr + 32 + i * 32, &idx.0);
    }
    Ok(required)
}

//! Narrow cSigma Finance vault calldata builders for lender flows.
//!
//! cSigma's credit pools are reached through its diamond's `VaultFacet`. A lender first
//! deposits an approved pool token into the vault, then invests the resulting vault balance
//! into a credit pool. Repaid or otherwise available vault balance can be withdrawn with
//! `withdrawRequest`; the call may register a pending request instead of transferring
//! immediately when the protocol's amount or cooling-time threshold is not met.
//!
//! All three calls require `msg.sender` to be the wallet bound to the supplied lender ID and
//! require that lender to be KYB-verified. `deposit` also requires an ERC-20 allowance for the
//! diamond. Pool-manager withdrawals and request-processing functions are intentionally omitted.
//!
//! ABI and permissioning reference:
//! <https://github.com/csigma-labs/csigma-protocol/blob/main/contracts/facets/VaultFacet.sol>

use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(string,address,uint256)",
    SEL_INVEST = b"invest(string,string,uint256)",
    SEL_WITHDRAW_REQUEST = b"withdrawRequest(string,address,uint256)",
}

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
    BufferTooSmall { required: usize },
}

const HEAD_LENGTH: usize = 32 * 3;
const STRING_LENGTH_WORD: usize = 32;

fn padded_len(length: usize) -> Option<usize> {
    length.checked_add(31).map(|n| n & !31)
}

fn single_string_calldata_len(string_len: usize) -> Result<usize, EncodeError> {
    let padded = padded_len(string_len).ok_or(EncodeError::LengthOverflow)?;
    4usize
        .checked_add(HEAD_LENGTH)
        .and_then(|n| n.checked_add(STRING_LENGTH_WORD))
        .and_then(|n| n.checked_add(padded))
        .ok_or(EncodeError::LengthOverflow)
}

/// Required output length for [`make_fn_deposit`] and [`make_fn_withdraw_request`].
pub fn vault_calldata_len(lender_id_len: usize) -> Result<usize, EncodeError> {
    single_string_calldata_len(lender_id_len)
}

/// Required output length for [`make_fn_invest`].
pub fn invest_calldata_len(lender_id_len: usize, pool_id_len: usize) -> Result<usize, EncodeError> {
    let lender_padded = padded_len(lender_id_len).ok_or(EncodeError::LengthOverflow)?;
    let pool_padded = padded_len(pool_id_len).ok_or(EncodeError::LengthOverflow)?;
    4usize
        .checked_add(HEAD_LENGTH)
        .and_then(|n| n.checked_add(STRING_LENGTH_WORD))
        .and_then(|n| n.checked_add(lender_padded))
        .and_then(|n| n.checked_add(STRING_LENGTH_WORD))
        .and_then(|n| n.checked_add(pool_padded))
        .ok_or(EncodeError::LengthOverflow)
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

fn put_address(output: &mut [u8], offset: usize, value: Address) {
    output[offset + 12..offset + 32].copy_from_slice(&value);
}

fn put_string(output: &mut [u8], offset: usize, value: &str) {
    put_usize(output, offset, value.len());
    output[offset + 32..offset + 32 + value.len()].copy_from_slice(value.as_bytes());
}

fn make_single_string_call(
    output: &mut [u8],
    selector: [u8; 4],
    lender_id: &str,
    token: Address,
    amount: &U,
) -> Result<usize, EncodeError> {
    let required = single_string_calldata_len(lender_id.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }

    let output = &mut output[..required];
    output.fill(0);
    output[..4].copy_from_slice(&selector);
    put_usize(output, 4, HEAD_LENGTH);
    put_address(output, 36, token);
    output[68..100].copy_from_slice(&amount.0);
    put_string(output, 100, lender_id);
    Ok(required)
}

/// Encode `deposit(lenderId, token, amount)`.
///
/// The caller must be the wallet bound to a KYB-verified `lender_id` and must approve the
/// cSigma diamond to transfer at least `amount` of the whitelisted `token` first.
pub fn make_fn_deposit(
    output: &mut [u8],
    lender_id: &str,
    token: Address,
    amount: &U,
) -> Result<usize, EncodeError> {
    make_single_string_call(output, SEL_DEPOSIT, lender_id, token, amount)
}

/// Encode `invest(lenderId, poolId, amount)` to move vault balance into a credit pool.
///
/// The caller must be the wallet bound to the KYB-verified `lender_id`; the pool must be active
/// and unexpired and must use the token already deposited into the lender's vault balance.
pub fn make_fn_invest(
    output: &mut [u8],
    lender_id: &str,
    pool_id: &str,
    amount: &U,
) -> Result<usize, EncodeError> {
    let required = invest_calldata_len(lender_id.len(), pool_id.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }

    let lender_padded = padded_len(lender_id.len()).ok_or(EncodeError::LengthOverflow)?;
    let pool_offset = HEAD_LENGTH
        .checked_add(STRING_LENGTH_WORD)
        .and_then(|n| n.checked_add(lender_padded))
        .ok_or(EncodeError::LengthOverflow)?;

    let output = &mut output[..required];
    output.fill(0);
    output[..4].copy_from_slice(&SEL_INVEST);
    put_usize(output, 4, HEAD_LENGTH);
    put_usize(output, 36, pool_offset);
    output[68..100].copy_from_slice(&amount.0);
    put_string(output, 100, lender_id);
    put_string(output, 4 + pool_offset, pool_id);
    Ok(required)
}

/// Encode `withdrawRequest(lenderId, token, amount)` for available vault balance.
///
/// This does not pull principal directly out of an active credit pool. Pool repayments and exits
/// first credit the lender's vault balance. Depending on cSigma's configured threshold and
/// cooling time, this call either transfers the token or creates a pending withdrawal request.
pub fn make_fn_withdraw_request(
    output: &mut [u8],
    lender_id: &str,
    token: Address,
    amount: &U,
) -> Result<usize, EncodeError> {
    make_single_string_call(output, SEL_WITHDRAW_REQUEST, lender_id, token, amount)
}

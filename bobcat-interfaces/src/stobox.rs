//! Narrow Stobox STV3 subscription calldata builders.
//!
//! An investor subscribes by approving the selected ERC-20 payment token and then
//! calling `purchase` on the relevant STV3 security-token contract. Offerings can
//! require a valid Stobox DID and enforce issuer-configured investment rules.
//!
//! Stobox's `redeem(uint256)` is intentionally not exposed: it is restricted to the
//! protocol's financial-operations role and burns assets held by the treasury rather
//! than providing an end-user redemption flow.

extern crate alloc;

use alloc::vec::Vec;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_PURCHASE = b"purchase(string,uint256,address,string)",
    SEL_PREVIEW_PURCHASE = b"previewPurchase(address,string,uint256,address)",
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
}

fn padded_len(length: usize) -> Result<usize, EncodeError> {
    length
        .checked_add(31)
        .map(|value| value & !31)
        .ok_or(EncodeError::LengthOverflow)
}

fn push_word(output: &mut Vec<u8>, word: &[u8; 32]) {
    output.extend_from_slice(word);
}

fn push_usize(output: &mut Vec<u8>, value: usize) {
    let mut word = [0_u8; 32];
    let bytes = value.to_be_bytes();
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    push_word(output, &word);
}

fn push_string(output: &mut Vec<u8>, value: &str) -> Result<(), EncodeError> {
    let bytes = value.as_bytes();
    let padded = padded_len(bytes.len())?;
    push_usize(output, bytes.len());
    output.extend_from_slice(bytes);
    output.resize(
        output
            .len()
            .checked_add(padded - bytes.len())
            .ok_or(EncodeError::LengthOverflow)?,
        0,
    );
    Ok(())
}

/// Required output length for [`make_fn_purchase`].
pub fn purchase_calldata_len(offering_id: &str, note: &str) -> Result<usize, EncodeError> {
    let offering_tail = 32_usize
        .checked_add(padded_len(offering_id.len())?)
        .ok_or(EncodeError::LengthOverflow)?;
    let note_tail = 32_usize
        .checked_add(padded_len(note.len())?)
        .ok_or(EncodeError::LengthOverflow)?;
    4_usize
        .checked_add(4 * 32)
        .and_then(|length| length.checked_add(offering_tail))
        .and_then(|length| length.checked_add(note_tail))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `PurchaseFacet.purchase` for an investor subscription.
///
/// Send this calldata to the STV3 security-token contract after approving
/// `payment_token` to spend the amount returned by `previewPurchase`.
pub fn make_fn_purchase(
    offering_id: &str,
    security_token_amount: &U,
    payment_token: Address,
    note: &str,
) -> Result<Vec<u8>, EncodeError> {
    let offering_tail = 32_usize
        .checked_add(padded_len(offering_id.len())?)
        .ok_or(EncodeError::LengthOverflow)?;
    let required = purchase_calldata_len(offering_id, note)?;
    let note_offset = (4_usize * 32)
        .checked_add(offering_tail)
        .ok_or(EncodeError::LengthOverflow)?;

    let mut output = Vec::with_capacity(required);
    output.extend_from_slice(&SEL_PURCHASE);
    push_usize(&mut output, 4 * 32);
    push_word(&mut output, &security_token_amount.0);
    push_word(&mut output, &leftpad_addr(payment_token));
    push_usize(&mut output, note_offset);
    push_string(&mut output, offering_id)?;
    push_string(&mut output, note)?;
    Ok(output)
}

/// Required output length for [`make_fn_preview_purchase`].
pub fn preview_purchase_calldata_len(offering_id: &str) -> Result<usize, EncodeError> {
    let offering_padded = padded_len(offering_id.len())?;
    4_usize
        .checked_add(4 * 32)
        .and_then(|length| length.checked_add(32))
        .and_then(|length| length.checked_add(offering_padded))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `PurchaseFacet.previewPurchase` for an investor and offering.
///
/// Send this calldata to the same STV3 security-token contract that will receive
/// the subsequent `purchase` call.
pub fn make_fn_preview_purchase(
    investor: Address,
    offering_id: &str,
    security_token_amount: &U,
    payment_token: Address,
) -> Result<Vec<u8>, EncodeError> {
    let required = preview_purchase_calldata_len(offering_id)?;
    let mut output = Vec::with_capacity(required);
    output.extend_from_slice(&SEL_PREVIEW_PURCHASE);
    push_word(&mut output, &leftpad_addr(investor));
    push_usize(&mut output, 4 * 32);
    push_word(&mut output, &security_token_amount.0);
    push_word(&mut output, &leftpad_addr(payment_token));
    push_string(&mut output, offering_id)?;
    Ok(output)
}

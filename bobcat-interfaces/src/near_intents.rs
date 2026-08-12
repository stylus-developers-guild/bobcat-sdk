//! Core end-user calldata for the NEAR Intents Omni Bridge on Arbitrum.
//!
//! NEAR Intents is a cross-chain intent settlement protocol that uses the Omni
//! Bridge to move assets between NEAR and external chains. On Arbitrum, the
//! bridge contract (`OmniBridgeWormhole`) lets users deposit ERC-20 tokens
//! (or native ETH) into the bridge for transfer to a NEAR account, and claim
//! tokens that have arrived from NEAR.
//!
//! This module exposes only the two core end-user entrypoints:
//!
//! - `initTransfer` — lock tokens on Arbitrum to bridge them to a NEAR
//!   recipient. The caller pays `msg.value` to cover the Wormhole message fee.
//! - `finTransfer` — receive tokens on Arbitrum that were sent from NEAR. The
//!   caller supplies a signature from the NEAR bridge's derived address and
//!   the transfer payload.
//!
//! All admin, pausable, token-deployment, metadata, and relayer functions are
//! permissioned and intentionally excluded.
//!
//! # ABI
//!
//! `initTransfer(address,uint128,uint128,uint128,string,string)` — dynamic
//! (two `string` args require offset-based encoding).
//!
//! `finTransfer(bytes,(uint64,uint8,uint64,address,uint128,address,string,bytes))`
//! — dynamic (the `bytes` signature, `string` feeRecipient, and `bytes`
//! message all require offset-based encoding).
//!
//! ABI verified against the deployed `OmniBridgeWormhole` contract on Arbitrum
//! One at `0xd025b38762B4A4E36F0Cde483b86CB13ea00D989` (source verified via
//! Sourcify, solc 0.8.24, verified 2024-11-20).
//!
//! Sources:
//! - <https://github.com/Near-One/omni-bridge>
//! - <https://docs.near-intents.org/integration/bridging/overview>
//! - <https://docs.near.org/chain-abstraction/omnibridge/overview>

use bobcat_cd::leftpad_addr;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
}

selectors! {
    SEL_INIT_TRANSFER = b"initTransfer(address,uint128,uint128,uint128,string,string)",
    SEL_FIN_TRANSFER = b"finTransfer(bytes,(uint64,uint8,uint64,address,uint128,address,string,bytes))",
}

// ---------------------------------------------------------------------------
// Buffer writing helpers
// ---------------------------------------------------------------------------

fn put_word(output: &mut [u8], offset: usize, value: &[u8; 32]) {
    output[offset..offset + 32].copy_from_slice(value);
}

fn put_addr(output: &mut [u8], offset: usize, value: Address) {
    put_word(output, offset, &leftpad_addr(value));
}

fn put_u8(output: &mut [u8], offset: usize, value: u8) {
    let mut word = [0u8; 32];
    word[31] = value;
    put_word(output, offset, &word);
}

fn put_u64(output: &mut [u8], offset: usize, value: u64) {
    let mut word = [0u8; 32];
    word[24..].copy_from_slice(&value.to_be_bytes());
    put_word(output, offset, &word);
}

fn put_u128(output: &mut [u8], offset: usize, value: u128) {
    let mut word = [0u8; 32];
    word[16..].copy_from_slice(&value.to_be_bytes());
    put_word(output, offset, &word);
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let mut word = [0u8; 32];
    let bytes = value.to_be_bytes();
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    put_word(output, offset, &word);
}

/// Write a dynamic-length `bytes` or `string` tail at `offset`.
///
/// Writes: length word + data (right-padded to a multiple of 32).
/// Returns the number of bytes consumed.
fn put_bytes_tail(output: &mut [u8], offset: usize, data: &[u8]) -> usize {
    put_usize(output, offset, data.len());
    let data_start = offset + 32;
    let padded_len = data.len().div_ceil(32) * 32;
    output[data_start..data_start + data.len()].copy_from_slice(data);
    32 + padded_len
}

// ---------------------------------------------------------------------------
// initTransfer
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_init_transfer`].
///
/// `initTransfer(address,uint128,uint128,uint128,string,string)` ABI encoding:
/// ```text
/// selector(4)
/// + tokenAddress(32) + amount(32) + fee(32) + nativeFee(32)   // 4 static head words
/// + offset_to_recipient(32) + offset_to_message(32)          // 2 dynamic head words
/// + recipient_tail: length(32) + ceil(len/32)*32
/// + message_tail:   length(32) + ceil(len/32)*32
/// ```
pub fn init_transfer_calldata_len(
    recipient_len: usize,
    message_len: usize,
) -> Result<usize, EncodeError> {
    let recipient_padded = recipient_len.div_ceil(32) * 32;
    let message_padded = message_len.div_ceil(32) * 32;
    4_usize
        .checked_add(32 * 6) // 4 static + 2 offset head words
        .and_then(|s| s.checked_add(32 + recipient_padded))
        .and_then(|s| s.checked_add(32 + message_padded))
        .ok_or(EncodeError::BufferTooSmall { required: 0 })
}

/// Encode `initTransfer(tokenAddress, amount, fee, nativeFee, recipient, message)`.
///
/// This is the core deposit flow: the user locks tokens on Arbitrum to bridge
/// them to a NEAR account. For native ETH, pass `token_address = [0u8; 20]`.
///
/// `amount` is the token amount in its smallest unit (wei for ERC-20, wei for
/// native ETH). `fee` is the bridge fee (must be < `amount`); `native_fee` is
/// the Wormhole message fee which the caller must supply as `msg.value` in
/// addition to `amount` when bridging native ETH.
///
/// `recipient` is the NEAR account ID that will receive the bridged tokens
/// (e.g. `b"alice.near"`). `message` is an optional opaque message payload
/// (pass empty `&[]` for a plain transfer).
///
/// The caller must supply a buffer of at least
/// [`init_transfer_calldata_len`]`(recipient.len(), message.len())` bytes.
/// Returns the number of bytes written.
pub fn make_fn_init_transfer(
    output: &mut [u8],
    token_address: Address,
    amount: u128,
    fee: u128,
    native_fee: u128,
    recipient: &[u8],
    message: &[u8],
) -> Result<usize, EncodeError> {
    let required = init_transfer_calldata_len(recipient.len(), message.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_INIT_TRANSFER);

    // --- Head: 4 static + 2 dynamic offsets ---
    let head_base = 4;
    put_addr(out, head_base, token_address); // word 0
    put_u128(out, head_base + 32, amount); // word 1
    put_u128(out, head_base + 64, fee); // word 2
    put_u128(out, head_base + 96, native_fee); // word 3

    // Offsets to dynamic data, relative to the start of the argument region
    // (which begins right after the selector, at head_base).
    let head_end = head_base + 32 * 6; // after all 6 head words
    let recipient_offset = head_end - head_base; // = 6*32 = 192
    let recipient_tail_len = 32 + (recipient.len().div_ceil(32) * 32);
    let message_offset = recipient_offset + recipient_tail_len;

    put_usize(out, head_base + 32 * 4, recipient_offset); // word 4: offset to recipient
    put_usize(out, head_base + 32 * 5, message_offset); // word 5: offset to message

    // --- Tails ---
    let mut o = head_end;
    o += put_bytes_tail(out, o, recipient);
    o += put_bytes_tail(out, o, message);

    debug_assert_eq!(o, required);
    Ok(required)
}

// ---------------------------------------------------------------------------
// finTransfer
// ---------------------------------------------------------------------------

/// The payload for `finTransfer` — the `TransferMessagePayload` struct.
///
/// This encodes the transfer message that was signed by the NEAR bridge's
/// derived address. The relayer submits this along with the signature to
/// claim tokens that arrived from NEAR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferMessagePayload<'a> {
    /// Nonce on the destination chain (Arbitrum), monotonically increasing.
    pub destination_nonce: u64,
    /// Origin chain ID (Omni Bridge chain ID of the source chain).
    pub origin_chain: u8,
    /// Nonce on the origin chain.
    pub origin_nonce: u64,
    /// Token address on Arbitrum (`address(0)` for native ETH).
    pub token_address: Address,
    /// Amount in the token's smallest unit.
    pub amount: u128,
    /// Recipient address on Arbitrum.
    pub recipient: Address,
    /// NEAR account ID of the fee recipient (can be empty).
    pub fee_recipient: &'a [u8],
    /// Opaque message bytes (can be empty).
    pub message: &'a [u8],
}

/// Required output length for [`make_fn_fin_transfer`].
///
/// `finTransfer(bytes,(uint64,uint8,uint64,address,uint128,address,string,bytes))`:
/// ```text
/// selector(4)
/// + offset_to_signature(32)                                      // head word 0
/// + offset_to_struct(32)                                         // head word 1
/// + signature_tail: length(32) + ceil(sig_len/32)*32
/// + struct_tail:
///     destinationNonce(32) + originChain(32) + originNonce(32)
///     + tokenAddress(32) + amount(32) + recipient(32)
///     + offset_to_feeRecipient(32) + offset_to_message(32)       // 8 head words
///     + feeRecipient_tail: length(32) + ceil(fr_len/32)*32
///     + message_tail: length(32) + ceil(msg_len/32)*32
/// ```
pub fn fin_transfer_calldata_len(
    signature_len: usize,
    fee_recipient_len: usize,
    message_len: usize,
) -> Result<usize, EncodeError> {
    let sig_padded = signature_len.div_ceil(32) * 32;
    let fr_padded = fee_recipient_len.div_ceil(32) * 32;
    let msg_padded = message_len.div_ceil(32) * 32;
    // selector + 2 head offsets + signature tail + struct(8 static words + 2 dynamic tails)
    4_usize
        .checked_add(32 * 2) // 2 head offset words
        .and_then(|s| s.checked_add(32 + sig_padded)) // signature tail
        .and_then(|s| s.checked_add(32 * 8)) // struct: 6 static + 2 dynamic offset words
        .and_then(|s| s.checked_add(32 + fr_padded)) // feeRecipient tail
        .and_then(|s| s.checked_add(32 + msg_padded)) // message tail
        .ok_or(EncodeError::BufferTooSmall { required: 0 })
}

/// Encode `finTransfer(signatureData, payload)`.
///
/// This is the core claim flow: a relayer submits a signed transfer message
/// from the NEAR bridge to mint/transfer tokens to an Arbitrum recipient.
/// The `signature_data` is the ECDSA signature over the Borsh-encoded
/// `TransferMessagePayload`, recoverable to the bridge's
/// `nearBridgeDerivedAddress`. The `payload` contains the transfer details.
///
/// The caller must supply a buffer of at least
/// [`fin_transfer_calldata_len`]`(signature.len(), fee_recipient.len(), message.len())`
/// bytes. Returns the number of bytes written.
pub fn make_fn_fin_transfer(
    output: &mut [u8],
    signature_data: &[u8],
    payload: &TransferMessagePayload,
) -> Result<usize, EncodeError> {
    let required = fin_transfer_calldata_len(
        signature_data.len(),
        payload.fee_recipient.len(),
        payload.message.len(),
    )?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_FIN_TRANSFER);

    // The encoding region starts right after the selector.
    // Head: 2 offset words (offset to bytes signatureData, offset to struct payload)
    let head_base = 4;
    let head_end = head_base + 32 * 2;

    // --- Signature tail (bytes) ---
    // Offset relative to the start of the encoding after selector.
    let sig_offset = head_end - head_base; // = 64
    put_usize(out, head_base, sig_offset); // head word 0: offset to signature
    let mut o = head_end;
    o += put_bytes_tail(out, o, signature_data);
    let sig_tail_end = o;

    // --- Struct tail ---
    // The struct is dynamic (contains string + bytes), so it's a tail with:
    // 6 static words + 2 dynamic offset words, then the dynamic tails.
    // Offset relative to the start of the encoding after selector.
    let struct_offset = sig_tail_end - head_base;
    put_usize(out, head_base + 32, struct_offset); // head word 1: offset to struct

    let struct_base = sig_tail_end;
    // Static fields (6 words):
    put_u64(out, struct_base, payload.destination_nonce); // word 0
    put_u8(out, struct_base + 32, payload.origin_chain); // word 1
    put_u64(out, struct_base + 64, payload.origin_nonce); // word 2
    put_addr(out, struct_base + 96, payload.token_address); // word 3
    put_u128(out, struct_base + 128, payload.amount); // word 4
    put_addr(out, struct_base + 160, payload.recipient); // word 5

    // Dynamic offset words (relative to the start of the struct):
    // word 6: offset to feeRecipient = 8 * 32 (after the 8 head words)
    // word 7: offset to message = 8 * 32 + feeRecipient_tail_len
    let fr_tail_len = 32 + (payload.fee_recipient.len().div_ceil(32) * 32);
    let fr_offset = 8 * 32;
    let msg_offset = fr_offset + fr_tail_len;
    put_usize(out, struct_base + 32 * 6, fr_offset); // word 6
    put_usize(out, struct_base + 32 * 7, msg_offset); // word 7

    // Dynamic tails:
    let mut tail = struct_base + 32 * 8;
    tail += put_bytes_tail(out, tail, payload.fee_recipient);
    tail += put_bytes_tail(out, tail, payload.message);

    debug_assert_eq!(tail, required);
    Ok(required)
}

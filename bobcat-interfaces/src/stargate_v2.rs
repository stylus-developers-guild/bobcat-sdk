//! Narrow Stargate V2 bridging calldata for core end-user flows.
//!
//! The builders target Stargate V2's `IStargate` ABI and cover the three
//! core end-user operations: quoting the OFT limit with `quoteOFT`, quoting
//! the LayerZero messaging fee with `quoteSend`, and bridging a token across
//! chains with `sendToken`. Callers remain responsible for ERC-20 approval of
//! the Stargate token contract and for supplying the native messaging fee as
//! `msg.value` on `sendToken`.
//!
//! `SendParam` contains three dynamic `bytes` fields. For the common taxi-mode
//! flow where all three are empty, use [`make_fn_quote_oft_taxi`],
//! [`make_fn_quote_send_taxi`], and [`make_fn_send_token_taxi`] which return
//! fixed-size arrays. For the general case with non-empty bytes fields, use
//! the [`quote_oft_calldata_len`], [`quote_send_calldata_len`], and
//! [`send_token_calldata_len`] helpers to size a buffer and then call
//! [`make_fn_quote_oft`], [`make_fn_quote_send`], or [`make_fn_send_token`].
//!
//! Sources:
//! - Stargate V2 `IStargate` interface, verified against the official
//!   Stargate V2 contracts on Arbitrum.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_bool, leftpad_u32, leftpad_usize};
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_SEND_TOKEN = b"sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)",
    SEL_QUOTE_SEND = b"quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)",
    SEL_QUOTE_OFT = b"quoteOFT((uint32,bytes32,uint256,uint256,bytes,bytes,bytes))",
}

/// Stargate V2 `SendParam` struct.
///
/// `to` is a `bytes32` that encodes the recipient address, left-padded to
/// 32 bytes. Use [`address_to_bytes32`] to convert an address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendParam<'a> {
    /// Destination endpoint ID.
    pub dst_eid: u32,
    /// Recipient address, left-padded to 32 bytes.
    pub to: [u8; 32],
    /// Amount to send, in local decimals.
    pub amount_ld: U,
    /// Minimum amount to send, in local decimals.
    pub min_amount_ld: U,
    /// Additional LayerZero options.
    pub extra_options: &'a [u8],
    /// Composed message for the send operation.
    pub compose_msg: &'a [u8],
    /// OFT command (empty for taxi mode).
    pub oft_cmd: &'a [u8],
}

/// Stargate V2 `MessagingFee` struct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessagingFee {
    pub native_fee: U,
    pub lz_token_fee: U,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
    BufferTooSmall { required: usize },
}

/// Convert an address to the `bytes32` format Stargate V2 expects.
pub const fn address_to_bytes32(addr: Address) -> [u8; 32] {
    leftpad_addr(addr)
}

fn padded_len(length: usize) -> Option<usize> {
    length.checked_add(31).map(|n| n & !31)
}

/// Size of the `SendParam` tuple head (7 words: 4 static fields + 3 offsets).
const TUPLE_HEAD: usize = 224;

// ---------------------------------------------------------------------------
// Mutable-slice helpers
// ---------------------------------------------------------------------------

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

fn put_u32(output: &mut [u8], offset: usize, value: u32) {
    output[offset + 28..offset + 32].copy_from_slice(&value.to_be_bytes());
}

// ---------------------------------------------------------------------------
// Tuple encoder (shared by all three functions)
// ---------------------------------------------------------------------------

/// Encode the `SendParam` tuple into `output` starting at `tuple_offset`.
///
/// Returns the number of bytes written.
fn encode_send_param(
    output: &mut [u8],
    tuple_offset: usize,
    send: &SendParam<'_>,
) -> Result<usize, EncodeError> {
    let opts_padded = padded_len(send.extra_options.len()).ok_or(EncodeError::LengthOverflow)?;
    let msg_padded = padded_len(send.compose_msg.len()).ok_or(EncodeError::LengthOverflow)?;
    let cmd_padded = padded_len(send.oft_cmd.len()).ok_or(EncodeError::LengthOverflow)?;

    let tail = 96usize
        .checked_add(opts_padded)
        .and_then(|n| n.checked_add(msg_padded))
        .and_then(|n| n.checked_add(cmd_padded))
        .ok_or(EncodeError::LengthOverflow)?;

    let tuple_size = TUPLE_HEAD.checked_add(tail).ok_or(EncodeError::LengthOverflow)?;
    let end = tuple_offset.checked_add(tuple_size).ok_or(EncodeError::LengthOverflow)?;

    if output.len() < end {
        return Err(EncodeError::BufferTooSmall { required: end });
    }

    let tuple = &mut output[tuple_offset..end];
    tuple.fill(0);

    // Head: 4 static fields + 3 dynamic-field offsets
    put_u32(tuple, 0, send.dst_eid);
    tuple[32..64].copy_from_slice(&send.to);
    tuple[64..96].copy_from_slice(&send.amount_ld.0);
    tuple[96..128].copy_from_slice(&send.min_amount_ld.0);

    let offset_opts = TUPLE_HEAD;
    let offset_msg = offset_opts
        .checked_add(32)
        .and_then(|n| n.checked_add(opts_padded))
        .ok_or(EncodeError::LengthOverflow)?;
    let offset_cmd = offset_msg
        .checked_add(32)
        .and_then(|n| n.checked_add(msg_padded))
        .ok_or(EncodeError::LengthOverflow)?;

    put_usize(tuple, 128, offset_opts);
    put_usize(tuple, 160, offset_msg);
    put_usize(tuple, 192, offset_cmd);

    // Tail: extra_options
    put_usize(tuple, offset_opts, send.extra_options.len());
    tuple[offset_opts + 32..offset_opts + 32 + send.extra_options.len()]
        .copy_from_slice(send.extra_options);

    // Tail: compose_msg
    put_usize(tuple, offset_msg, send.compose_msg.len());
    tuple[offset_msg + 32..offset_msg + 32 + send.compose_msg.len()]
        .copy_from_slice(send.compose_msg);

    // Tail: oft_cmd
    put_usize(tuple, offset_cmd, send.oft_cmd.len());
    tuple[offset_cmd + 32..offset_cmd + 32 + send.oft_cmd.len()]
        .copy_from_slice(send.oft_cmd);

    Ok(tuple_size)
}

// ---------------------------------------------------------------------------
// quoteOFT
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_quote_oft`].
pub fn quote_oft_calldata_len(send: &SendParam<'_>) -> Result<usize, EncodeError> {
    let opts = padded_len(send.extra_options.len()).ok_or(EncodeError::LengthOverflow)?;
    let msg = padded_len(send.compose_msg.len()).ok_or(EncodeError::LengthOverflow)?;
    let cmd = padded_len(send.oft_cmd.len()).ok_or(EncodeError::LengthOverflow)?;
    // selector(4) + offset(32) + tuple_head(224) + 3 length words(96) + padded data
    356usize
        .checked_add(opts)
        .and_then(|n| n.checked_add(msg))
        .and_then(|n| n.checked_add(cmd))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `IStargate.quoteOFT(SendParam)`.
///
/// The returned `OFTLimit`, `OFTFeeDetail[]`, and `OFTReceipt` are decoded
/// off-chain from the view call's return data; this builder only produces the
/// calldata to invoke the quote.
pub fn make_fn_quote_oft(
    output: &mut [u8],
    send: &SendParam<'_>,
) -> Result<usize, EncodeError> {
    let required = quote_oft_calldata_len(send)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    output[..4].copy_from_slice(&SEL_QUOTE_OFT);
    put_usize(output, 4, 32); // offset to tuple — only argument
    encode_send_param(output, 36, send)?;
    Ok(required)
}

// ---------------------------------------------------------------------------
// quoteSend
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_quote_send`].
pub fn quote_send_calldata_len(send: &SendParam<'_>) -> Result<usize, EncodeError> {
    let opts = padded_len(send.extra_options.len()).ok_or(EncodeError::LengthOverflow)?;
    let msg = padded_len(send.compose_msg.len()).ok_or(EncodeError::LengthOverflow)?;
    let cmd = padded_len(send.oft_cmd.len()).ok_or(EncodeError::LengthOverflow)?;
    // selector(4) + offset(32) + bool(32) + tuple_head(224) + 3 length words(96) + padded data
    388usize
        .checked_add(opts)
        .and_then(|n| n.checked_add(msg))
        .and_then(|n| n.checked_add(cmd))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `IStargate.quoteSend(SendParam, bool)`.
///
/// The returned `MessagingFee` (nativeFee, lzTokenFee) is decoded off-chain
/// from the view call's return data; this builder only produces the calldata
/// to invoke the quote.
pub fn make_fn_quote_send(
    output: &mut [u8],
    send: &SendParam<'_>,
    pay_in_lz_token: bool,
) -> Result<usize, EncodeError> {
    let required = quote_send_calldata_len(send)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    output[..4].copy_from_slice(&SEL_QUOTE_SEND);
    put_usize(output, 4, 64); // offset to SendParam = 2 words (offset + bool)
    output[67] = u8::from(pay_in_lz_token); // bool at word 1
    encode_send_param(output, 68, send)?;
    Ok(required)
}

// ---------------------------------------------------------------------------
// sendToken
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_send_token`].
pub fn send_token_calldata_len(send: &SendParam<'_>) -> Result<usize, EncodeError> {
    let opts = padded_len(send.extra_options.len()).ok_or(EncodeError::LengthOverflow)?;
    let msg = padded_len(send.compose_msg.len()).ok_or(EncodeError::LengthOverflow)?;
    let cmd = padded_len(send.oft_cmd.len()).ok_or(EncodeError::LengthOverflow)?;
    // selector(4) + offset(32) + fee(64) + address(32) + tuple_head(224) + 3 length words(96) + padded data
    452usize
        .checked_add(opts)
        .and_then(|n| n.checked_add(msg))
        .and_then(|n| n.checked_add(cmd))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `IStargate.sendToken(SendParam, MessagingFee, address)`.
///
/// `sendToken` is payable; the caller must supply `fee.native_fee` as
/// `msg.value`. The caller must also approve the Stargate token contract to
/// spend `send.amount_ld` of the underlying ERC-20.
pub fn make_fn_send_token(
    output: &mut [u8],
    send: &SendParam<'_>,
    fee: &MessagingFee,
    refund_address: Address,
) -> Result<usize, EncodeError> {
    let required = send_token_calldata_len(send)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    output[..4].copy_from_slice(&SEL_SEND_TOKEN);
    put_usize(output, 4, 128); // offset to SendParam = 4 words
    output[36..68].copy_from_slice(&fee.native_fee.0);
    output[68..100].copy_from_slice(&fee.lz_token_fee.0);
    output[112..132].copy_from_slice(&refund_address); // address at word 3
    encode_send_param(output, 132, send)?;
    Ok(required)
}

// ---------------------------------------------------------------------------
// Taxi-mode const builders (all bytes fields empty)
// ---------------------------------------------------------------------------

/// Encode `IStargate.quoteOFT(SendParam)` for the common taxi-mode flow where
/// `extra_options`, `compose_msg`, and `oft_cmd` are all empty.
pub const fn make_fn_quote_oft_taxi(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
) -> [u8; 356] {
    concat_arrays!(
        SEL_QUOTE_OFT,          // 4:  selector
        leftpad_usize(32),      // 32: offset to tuple
        leftpad_u32(dst_eid),   // 32: dstEid
        to,                     // 32: to
        amount_ld.0,            // 32: amountLD
        min_amount_ld.0,        // 32: minAmountLD
        leftpad_usize(224),     // 32: offset to extra_options
        leftpad_usize(256),     // 32: offset to compose_msg
        leftpad_usize(288),     // 32: offset to oft_cmd
        [0u8; 32],              // 32: length of extra_options = 0
        [0u8; 32],              // 32: length of compose_msg = 0
        [0u8; 32]               // 32: length of oft_cmd = 0
    )
}

/// Encode `IStargate.quoteSend(SendParam, bool)` for the common taxi-mode flow
/// where `extra_options`, `compose_msg`, and `oft_cmd` are all empty.
pub const fn make_fn_quote_send_taxi(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    pay_in_lz_token: bool,
) -> [u8; 388] {
    concat_arrays!(
        SEL_QUOTE_SEND,              // 4:  selector
        leftpad_usize(64),           // 32: offset to SendParam
        leftpad_bool(pay_in_lz_token), // 32: _payInLzToken
        leftpad_u32(dst_eid),        // 32: dstEid
        to,                          // 32: to
        amount_ld.0,                 // 32: amountLD
        min_amount_ld.0,             // 32: minAmountLD
        leftpad_usize(224),          // 32: offset to extra_options
        leftpad_usize(256),          // 32: offset to compose_msg
        leftpad_usize(288),          // 32: offset to oft_cmd
        [0u8; 32],                   // 32: length of extra_options = 0
        [0u8; 32],                   // 32: length of compose_msg = 0
        [0u8; 32]                    // 32: length of oft_cmd = 0
    )
}

/// Encode `IStargate.sendToken(SendParam, MessagingFee, address)` for the
/// common taxi-mode flow where `extra_options`, `compose_msg`, and `oft_cmd`
/// are all empty.
///
/// `sendToken` is payable; the caller must supply `fee.native_fee` as
/// `msg.value`. The caller must also approve the Stargate token contract to
/// spend `amount_ld` of the underlying ERC-20.
pub const fn make_fn_send_token_taxi(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    fee: &MessagingFee,
    refund_address: Address,
) -> [u8; 452] {
    concat_arrays!(
        SEL_SEND_TOKEN,               // 4:  selector
        leftpad_usize(128),            // 32: offset to SendParam
        fee.native_fee.0,             // 32: nativeFee
        fee.lz_token_fee.0,           // 32: lzTokenFee
        leftpad_addr(refund_address),  // 32: refundAddress
        leftpad_u32(dst_eid),         // 32: dstEid
        to,                            // 32: to
        amount_ld.0,                   // 32: amountLD
        min_amount_ld.0,               // 32: minAmountLD
        leftpad_usize(224),            // 32: offset to extra_options
        leftpad_usize(256),            // 32: offset to compose_msg
        leftpad_usize(288),            // 32: offset to oft_cmd
        [0u8; 32],                     // 32: length of extra_options = 0
        [0u8; 32],                     // 32: length of compose_msg = 0
        [0u8; 32]                      // 32: length of oft_cmd = 0
    )
}

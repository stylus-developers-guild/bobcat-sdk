//! Core end-user calldata builders for Theo Network's thBILL OFT.
//!
//! On Arbitrum, thBILL is a native LayerZero OFT. Sending burns thBILL on the
//! source chain and mints it through the authenticated OFT peer on the
//! destination chain. Direct issuance and redemption are permissioned Theo
//! flows and are intentionally not exposed here.

extern crate alloc;

use alloc::vec::Vec;
use bobcat_cd::{leftpad_addr, leftpad_bool};
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_QUOTE_SEND = b"quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)",
    SEL_SEND = b"send((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)",
}

/// LayerZero OFT send parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendParams<'a> {
    pub destination_endpoint_id: u32,
    pub recipient: [u8; 32],
    pub amount: U,
    pub minimum_amount: U,
    pub extra_options: &'a [u8],
    pub compose_message: &'a [u8],
    pub oft_command: &'a [u8],
}

/// LayerZero messaging fee returned by `quoteSend`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessagingFee {
    pub native_fee: U,
    pub lz_token_fee: U,
}

/// Encode `quoteSend(params, pay_in_lz_token)`.
pub fn make_fn_quote_send(params: &SendParams<'_>, pay_in_lz_token: bool) -> Vec<u8> {
    let encoded_params_len = encoded_params_len(params);
    let mut out = Vec::with_capacity(4 + 32 * 2 + encoded_params_len);
    out.extend_from_slice(&SEL_QUOTE_SEND);
    push_usize(&mut out, 32 * 2);
    push_word(&mut out, &leftpad_bool(pay_in_lz_token));
    push_send_params(&mut out, params);
    out
}

/// Encode the payable `send(params, fee, refund_address)` OFT call.
///
/// The call value must cover `fee.native_fee`.
pub fn make_fn_send(
    params: &SendParams<'_>,
    fee: &MessagingFee,
    refund_address: Address,
) -> Vec<u8> {
    let encoded_params_len = encoded_params_len(params);
    let mut out = Vec::with_capacity(4 + 32 * 4 + encoded_params_len);
    out.extend_from_slice(&SEL_SEND);
    push_usize(&mut out, 32 * 4);
    push_word(&mut out, &fee.native_fee.0);
    push_word(&mut out, &fee.lz_token_fee.0);
    push_word(&mut out, &leftpad_addr(refund_address));
    push_send_params(&mut out, params);
    out
}

fn push_send_params(out: &mut Vec<u8>, params: &SendParams<'_>) {
    let extra_options_offset = 32 * 7;
    let compose_message_offset = extra_options_offset + encoded_bytes_len(params.extra_options);
    let oft_command_offset = compose_message_offset + encoded_bytes_len(params.compose_message);

    push_u32(out, params.destination_endpoint_id);
    push_word(out, &params.recipient);
    push_word(out, &params.amount.0);
    push_word(out, &params.minimum_amount.0);
    push_usize(out, extra_options_offset);
    push_usize(out, compose_message_offset);
    push_usize(out, oft_command_offset);
    push_bytes(out, params.extra_options);
    push_bytes(out, params.compose_message);
    push_bytes(out, params.oft_command);
}

fn encoded_params_len(params: &SendParams<'_>) -> usize {
    32 * 7
        + encoded_bytes_len(params.extra_options)
        + encoded_bytes_len(params.compose_message)
        + encoded_bytes_len(params.oft_command)
}

fn encoded_bytes_len(value: &[u8]) -> usize {
    32 + value.len().div_ceil(32) * 32
}

fn push_bytes(out: &mut Vec<u8>, value: &[u8]) {
    push_usize(out, value.len());
    out.extend_from_slice(value);
    let padding = value.len().next_multiple_of(32) - value.len();
    out.resize(out.len() + padding, 0);
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    let mut word = [0; 32];
    word[28..].copy_from_slice(&value.to_be_bytes());
    push_word(out, &word);
}

fn push_usize(out: &mut Vec<u8>, value: usize) {
    let mut word = [0; 32];
    let bytes = value.to_be_bytes();
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    push_word(out, &word);
}

fn push_word(out: &mut Vec<u8>, word: &[u8; 32]) {
    out.extend_from_slice(word);
}

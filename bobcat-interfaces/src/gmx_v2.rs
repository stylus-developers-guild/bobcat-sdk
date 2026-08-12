//! Minimal GMX V2 core order calldata builders.
//!
//! Collateral and execution fees must be transferred to GMX's OrderVault before
//! creating an order, normally in the same ExchangeRouter multicall.

extern crate alloc;

use alloc::vec::Vec;
use bobcat_cd::{leftpad_addr, leftpad_bool};
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_CREATE_ORDER = b"createOrder(((address,address,address,address,address,address,address[]),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256),uint8,uint8,bool,bool,bool,bytes32,bytes32[]))",
    SEL_CANCEL_ORDER = b"cancelOrder(bytes32)",
    SEL_CLAIM_FUNDING_FEES = b"claimFundingFees(address[],address[],address)",
    SEL_CLAIM_COLLATERAL = b"claimCollateral(address[],address[],uint256[],address)"
}

/// GMX order types that increase a position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum IncreaseOrderType {
    Market = 2,
    Limit = 3,
    Stop = 8,
}

/// GMX order types that decrease a position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DecreaseOrderType {
    Market = 4,
    Limit = 5,
    StopLoss = 6,
}

/// Handling for the PnL token during a decrease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DecreasePositionSwapType {
    NoSwap = 0,
    SwapPnlTokenToCollateralToken = 1,
    SwapCollateralTokenToPnlToken = 2,
}

/// Address fields from GMX's `CreateOrderParamsAddresses`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderAddresses<'a> {
    pub receiver: Address,
    pub cancellation_receiver: Address,
    pub callback_contract: Address,
    pub ui_fee_receiver: Address,
    pub market: Address,
    pub initial_collateral_token: Address,
    pub swap_path: &'a [Address],
}

/// Numeric fields from GMX's `CreateOrderParamsNumbers`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderNumbers {
    pub size_delta_usd: U,
    pub initial_collateral_delta_amount: U,
    pub trigger_price: U,
    pub acceptable_price: U,
    pub execution_fee: U,
    pub callback_gas_limit: U,
    pub min_output_amount: U,
    pub valid_from_time: U,
}

/// Shared inputs for GMX increase and decrease orders.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderParams<'a> {
    pub addresses: OrderAddresses<'a>,
    pub numbers: OrderNumbers,
    pub decrease_position_swap_type: DecreasePositionSwapType,
    pub is_long: bool,
    pub should_unwrap_native_token: bool,
    pub auto_cancel: bool,
}

/// Encode a GMX V2 increase order.
pub fn make_fn_create_increase_order(
    params: &OrderParams<'_>,
    order_type: IncreaseOrderType,
) -> Vec<u8> {
    encode_create_order(params, order_type as u8)
}

/// Encode a GMX V2 decrease order.
pub fn make_fn_create_decrease_order(
    params: &OrderParams<'_>,
    order_type: DecreaseOrderType,
) -> Vec<u8> {
    encode_create_order(params, order_type as u8)
}

fn encode_create_order(params: &OrderParams<'_>, order_type: u8) -> Vec<u8> {
    let swap_len = params.addresses.swap_path.len();
    let mut out = Vec::with_capacity(4 + 32 * (26 + swap_len));
    out.extend_from_slice(&SEL_CREATE_ORDER);

    // createOrder has one dynamic tuple argument.
    push_usize(&mut out, 32);

    // CreateOrderParams head. The addresses tuple is dynamic.
    push_usize(&mut out, 16 * 32);
    push_word(&mut out, &params.numbers.size_delta_usd.0);
    push_word(&mut out, &params.numbers.initial_collateral_delta_amount.0);
    push_word(&mut out, &params.numbers.trigger_price.0);
    push_word(&mut out, &params.numbers.acceptable_price.0);
    push_word(&mut out, &params.numbers.execution_fee.0);
    push_word(&mut out, &params.numbers.callback_gas_limit.0);
    push_word(&mut out, &params.numbers.min_output_amount.0);
    push_word(&mut out, &params.numbers.valid_from_time.0);
    push_u8(&mut out, order_type);
    push_u8(&mut out, params.decrease_position_swap_type as u8);
    push_bool(&mut out, params.is_long);
    push_bool(&mut out, params.should_unwrap_native_token);
    push_bool(&mut out, params.auto_cancel);
    // Core orders do not expose referral or extension fields.
    push_word(&mut out, &[0; 32]);
    push_usize(&mut out, (24 + swap_len) * 32);

    // CreateOrderParamsAddresses body.
    push_address(&mut out, params.addresses.receiver);
    push_address(&mut out, params.addresses.cancellation_receiver);
    push_address(&mut out, params.addresses.callback_contract);
    push_address(&mut out, params.addresses.ui_fee_receiver);
    push_address(&mut out, params.addresses.market);
    push_address(&mut out, params.addresses.initial_collateral_token);
    push_usize(&mut out, 7 * 32);
    push_usize(&mut out, swap_len);
    for address in params.addresses.swap_path {
        push_address(&mut out, *address);
    }

    // Empty bytes32[] dataList.
    push_usize(&mut out, 0);
    out
}

/// Encode `ExchangeRouter.cancelOrder` for either an increase or decrease key.
pub fn make_fn_cancel_order(key: [u8; 32]) -> [u8; 36] {
    let mut out = [0; 36];
    out[..4].copy_from_slice(&SEL_CANCEL_ORDER);
    out[4..].copy_from_slice(&key);
    out
}

/// Encode a claim for positive funding fees.
pub fn make_fn_claim_funding_fees(
    markets: &[Address],
    tokens: &[Address],
    receiver: Address,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 32 * (5 + markets.len() + tokens.len()));
    out.extend_from_slice(&SEL_CLAIM_FUNDING_FEES);
    push_usize(&mut out, 3 * 32);
    push_usize(&mut out, (4 + markets.len()) * 32);
    push_address(&mut out, receiver);
    push_addresses(&mut out, markets);
    push_addresses(&mut out, tokens);
    out
}

/// Encode a claim for collateral retained after capped negative price impact.
pub fn make_fn_claim_collateral(
    markets: &[Address],
    tokens: &[Address],
    time_keys: &[U],
    receiver: Address,
) -> Vec<u8> {
    let token_offset = (5 + markets.len()) * 32;
    let time_key_offset = token_offset + (1 + tokens.len()) * 32;
    let mut out = Vec::with_capacity(4 + 32 * (7 + markets.len() + tokens.len() + time_keys.len()));
    out.extend_from_slice(&SEL_CLAIM_COLLATERAL);
    push_usize(&mut out, 4 * 32);
    push_usize(&mut out, token_offset);
    push_usize(&mut out, time_key_offset);
    push_address(&mut out, receiver);
    push_addresses(&mut out, markets);
    push_addresses(&mut out, tokens);
    push_usize(&mut out, time_keys.len());
    for time_key in time_keys {
        push_word(&mut out, &time_key.0);
    }
    out
}

fn push_addresses(out: &mut Vec<u8>, values: &[Address]) {
    push_usize(out, values.len());
    for value in values {
        push_address(out, *value);
    }
}

fn push_address(out: &mut Vec<u8>, value: Address) {
    push_word(out, &leftpad_addr(value));
}

fn push_bool(out: &mut Vec<u8>, value: bool) {
    push_word(out, &leftpad_bool(value));
}

fn push_u8(out: &mut Vec<u8>, value: u8) {
    let mut word = [0; 32];
    word[31] = value;
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

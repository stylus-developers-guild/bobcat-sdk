//! Narrow Ostium trading calldata builders for core end-user flows.
//!
//! These builders target Ostium's `OstiumTrading` ABI and cover opening market,
//! limit, and stop orders; updating or cancelling pending limit orders; updating
//! take-profit and stop-loss prices; and partially or fully closing open trades.
//! Callers remain responsible for Ostium's fixed-point scaling and USDC approval.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_bool, leftpad_u8, leftpad_u16, leftpad_u32};
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// A big-endian `uint192`, used by Ostium for prices.
pub type Uint192 = [u8; 24];

selectors! {
    SEL_OPEN_TRADE = b"openTrade((uint256,uint192,uint192,uint192,address,uint32,uint16,uint8,bool,bool),(address,uint32),uint8,uint256)",
    SEL_CLOSE_TRADE_MARKET = b"closeTradeMarket(uint16,uint8,uint16,uint192,uint32)",
    SEL_UPDATE_OPEN_LIMIT_ORDER = b"updateOpenLimitOrder(uint16,uint8,uint192,uint192,uint192)",
    SEL_CANCEL_OPEN_LIMIT_ORDER = b"cancelOpenLimitOrder(uint16,uint8)",
    SEL_UPDATE_TP = b"updateTp(uint16,uint8,uint192)",
    SEL_UPDATE_SL = b"updateSl(uint16,uint8,uint192)",
}

/// Ostium's user-supplied trade fields.
///
/// `collateral` uses 6 decimals, prices use 18 decimals, and `leverage` uses 2
/// decimals. Ostium replaces `trader` and `index` where appropriate, but they
/// remain ABI fields and should match the initiating user and expected order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Trade {
    pub collateral: U,
    pub open_price: Uint192,
    pub take_profit: Uint192,
    pub stop_loss: Uint192,
    pub trader: Address,
    pub leverage: u32,
    pub pair_index: u16,
    pub index: u8,
    pub buy: bool,
    pub is_day_trade: bool,
}

/// Optional builder attribution and fee.
///
/// `builder_fee` uses 6 decimals. Use [`BuilderFee::NONE`] when no builder fee
/// should be charged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuilderFee {
    pub builder: Address,
    pub builder_fee: u32,
}

impl BuilderFee {
    pub const NONE: Self = Self {
        builder: [0; 20],
        builder_fee: 0,
    };
}

/// The type of order to open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OpenOrderType {
    Market = 0,
    Limit = 1,
    Stop = 2,
}

/// Encode `OstiumTrading.openTrade`.
///
/// `slippage_percent` uses 2 decimals. Ostium requires a non-zero value below
/// 100% for market orders and zero for limit or stop orders.
pub const fn make_fn_open_trade(
    trade: &Trade,
    builder_fee: &BuilderFee,
    order_type: OpenOrderType,
    slippage_percent: &U,
) -> [u8; 4 + 32 * 14] {
    concat_arrays!(
        SEL_OPEN_TRADE,
        trade.collateral.0,
        leftpad_u192(trade.open_price),
        leftpad_u192(trade.take_profit),
        leftpad_u192(trade.stop_loss),
        leftpad_addr(trade.trader),
        leftpad_u32(trade.leverage),
        leftpad_u16(trade.pair_index),
        leftpad_u8(trade.index),
        leftpad_bool(trade.buy),
        leftpad_bool(trade.is_day_trade),
        leftpad_addr(builder_fee.builder),
        leftpad_u32(builder_fee.builder_fee),
        leftpad_u8(order_type as u8),
        slippage_percent.0
    )
}

/// Encode a partial or full market close.
///
/// `close_percentage` and `slippage_percent` use 2 decimals. Ostium treats a
/// zero close percentage as 100%.
pub const fn make_fn_close_trade_market(
    pair_index: u16,
    index: u8,
    close_percentage: u16,
    market_price: Uint192,
    slippage_percent: u32,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_CLOSE_TRADE_MARKET,
        leftpad_u16(pair_index),
        leftpad_u8(index),
        leftpad_u16(close_percentage),
        leftpad_u192(market_price),
        leftpad_u32(slippage_percent)
    )
}

/// Encode an update to a pending limit or stop order's price, TP, and SL.
pub const fn make_fn_update_open_limit_order(
    pair_index: u16,
    index: u8,
    price: Uint192,
    take_profit: Uint192,
    stop_loss: Uint192,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_UPDATE_OPEN_LIMIT_ORDER,
        leftpad_u16(pair_index),
        leftpad_u8(index),
        leftpad_u192(price),
        leftpad_u192(take_profit),
        leftpad_u192(stop_loss)
    )
}

/// Encode cancellation of a pending limit or stop order.
pub const fn make_fn_cancel_open_limit_order(pair_index: u16, index: u8) -> [u8; 4 + 32 * 2] {
    concat_arrays!(
        SEL_CANCEL_OPEN_LIMIT_ORDER,
        leftpad_u16(pair_index),
        leftpad_u8(index)
    )
}

/// Encode an update to an open trade's take-profit price.
pub const fn make_fn_update_take_profit(
    pair_index: u16,
    index: u8,
    new_take_profit: Uint192,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_UPDATE_TP,
        leftpad_u16(pair_index),
        leftpad_u8(index),
        leftpad_u192(new_take_profit)
    )
}

/// Encode an update to an open trade's stop-loss price. Zero removes the SL.
pub const fn make_fn_update_stop_loss(
    pair_index: u16,
    index: u8,
    new_stop_loss: Uint192,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_UPDATE_SL,
        leftpad_u16(pair_index),
        leftpad_u8(index),
        leftpad_u192(new_stop_loss)
    )
}

const fn leftpad_u192(value: Uint192) -> [u8; 32] {
    concat_arrays!([0u8; 8], value)
}

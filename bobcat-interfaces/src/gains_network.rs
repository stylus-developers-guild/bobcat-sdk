//! Minimal Gains Network core trading calldata builders for the gTrade diamond
//! on Arbitrum.
//!
//! These builders target the v10 `GNSMultiCollatDiamond` ABI and cover the core
//! end-user leveraged trading lifecycle: opening a trade, market-closing,
//! updating take-profit / stop-loss / leverage, updating a pending limit or
//! trigger order, and cancelling a pending order.
//!
//! All calldata is fixed-size: the `Trade` tuple contains only static types, so
//! the entire encoding is static ABI and requires no heap allocation. Callers
//! remain responsible for collateral approval, fixed-point scaling, and
//! slippage selection.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_bool, leftpad_u16, leftpad_u24, leftpad_u32, leftpad_u8};

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Big-endian `uint120`, used for collateral amounts.
pub type Uint120 = [u8; 15];

/// Big-endian `uint160`, used for position size in the position token.
pub type Uint160 = [u8; 20];

/// Big-endian `uint24`, used for leverage and placeholder fields.
pub type Uint24 = [u8; 3];

/// Gains Network's `ITradingStorage.TradeType` enum.
///
/// `Market` submits a market order executed at the next oracle price. `Limit`
/// and `Trigger` submit conditional orders that execute when the oracle price
/// reaches the trigger price.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TradeType {
    Market = 0,
    Limit = 1,
    Trigger = 2,
}

/// Gains Network's `ITradingStorage.Trade` struct (v10).
///
/// `user` and `index` identify the trade slot; the diamond fills them for
/// new opens, but they must be provided for updates on existing trades.
/// `collateral_amount` uses the collateral token's own decimals. `open_price`,
/// `tp`, and `sl` use 1e10 precision. `leverage` uses 2 decimals (e.g. 1000 =
/// 10x). `position_size_token` uses the position token's own decimals. Set
/// `is_open` to true for a new position. Set `is_counter_trade` to false for
/// end-user trades. `placeholder` must be zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Trade {
    /// Trader EOA. Must match `msg.sender` for open; the stored user for updates.
    pub user: Address,
    /// Position index within the user's trade list.
    pub index: u32,
    /// Trading pair index.
    pub pair_index: u16,
    /// Desired leverage, 2 decimals (1000 = 10x). Big-endian `uint24`.
    pub leverage: Uint24,
    /// Long or short.
    pub long: bool,
    /// Whether this is a new open position.
    pub is_open: bool,
    /// Collateral token index (0 = DAI, 1 = WETH, etc.).
    pub collateral_index: u8,
    /// Order type for this trade.
    pub trade_type: TradeType,
    /// Collateral amount in the collateral token's decimals. Big-endian `uint120`.
    pub collateral_amount: Uint120,
    /// Desired open price, 1e10 precision. Zero for market orders.
    pub open_price: u64,
    /// Take-profit price, 1e10 precision. Zero to disable.
    pub tp: u64,
    /// Stop-loss price, 1e10 precision. Zero to disable.
    pub sl: u64,
    /// Whether this is a protocol counter-trade. End users set false.
    pub is_counter_trade: bool,
    /// Position size denominated in the position token. Big-endian `uint160`.
    pub position_size_token: Uint160,
    /// Reserved padding field; must be zero. Big-endian `uint24`.
    pub placeholder: Uint24,
}

selectors! {
    SEL_OPEN_TRADE = b"openTrade((address,uint32,uint16,uint24,bool,bool,uint8,uint8,uint120,uint64,uint64,uint64,bool,uint160,uint24),uint16,address)",
    SEL_CLOSE_TRADE_MARKET = b"closeTradeMarket(uint32,uint64)",
    SEL_UPDATE_TP = b"updateTp(uint32,uint64)",
    SEL_UPDATE_SL = b"updateSl(uint32,uint64)",
    SEL_UPDATE_LEVERAGE = b"updateLeverage(uint32,uint24)",
    SEL_UPDATE_OPEN_ORDER = b"updateOpenOrder(uint32,uint64,uint64,uint64,uint16)",
    SEL_CANCEL_OPEN_ORDER = b"cancelOpenOrder(uint32)",
}

/// Encode `openTrade(Trade, maxSlippageP, referrer)`.
///
/// Submits a new trade to the gTrade diamond. For market orders, set
/// `trade.open_price` to zero and `max_slippage_p` to a non-zero value (2
/// decimals, e.g. 100 = 1%). For limit/trigger orders, set `trade.open_price`
/// to the trigger price and `max_slippage_p` to zero. `referrer` is the
/// optional referrer address; use the zero address for none.
pub const fn make_fn_open_trade(
    trade: &Trade,
    max_slippage_p: u16,
    referrer: Address,
) -> [u8; 4 + 32 * 17] {
    concat_arrays!(
        SEL_OPEN_TRADE,
        leftpad_addr(trade.user),
        leftpad_u32(trade.index),
        leftpad_u16(trade.pair_index),
        leftpad_u24(trade.leverage),
        leftpad_bool(trade.long),
        leftpad_bool(trade.is_open),
        leftpad_u8(trade.collateral_index),
        leftpad_u8(trade.trade_type as u8),
        leftpad_u120(trade.collateral_amount),
        leftpad_u64(trade.open_price),
        leftpad_u64(trade.tp),
        leftpad_u64(trade.sl),
        leftpad_bool(trade.is_counter_trade),
        leftpad_addr(trade.position_size_token),
        leftpad_u24(trade.placeholder),
        leftpad_u16(max_slippage_p),
        leftpad_addr(referrer)
    )
}

/// Encode `closeTradeMarket(index, expectedPrice)`.
///
/// Initiates a market close of the open position at `index`. `expected_price`
/// is the caller's reference price in 1e10 precision; the diamond reverts if
/// the executed price deviates beyond the trade's `max_slippage_p`.
pub const fn make_fn_close_trade_market(
    index: u32,
    expected_price: u64,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(
        SEL_CLOSE_TRADE_MARKET,
        leftpad_u32(index),
        leftpad_u64(expected_price)
    )
}

/// Encode `updateTp(index, newTp)`.
///
/// Updates the take-profit price of an open position. `new_tp` uses 1e10
/// precision; zero disables the take-profit.
pub const fn make_fn_update_tp(index: u32, new_tp: u64) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_UPDATE_TP, leftpad_u32(index), leftpad_u64(new_tp))
}

/// Encode `updateSl(index, newSl)`.
///
/// Updates the stop-loss price of an open position. `new_sl` uses 1e10
/// precision; zero disables the stop-loss.
pub const fn make_fn_update_sl(index: u32, new_sl: u64) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_UPDATE_SL, leftpad_u32(index), leftpad_u64(new_sl))
}

/// Encode `updateLeverage(index, newLeverage)`.
///
/// Adjusts the leverage of an open position. `new_leverage` uses 2 decimals
/// (1000 = 10x). Big-endian `uint24`. The change is subject to the pair's max
/// leverage and may trigger a liquidation check.
pub const fn make_fn_update_leverage(
    index: u32,
    new_leverage: Uint24,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(
        SEL_UPDATE_LEVERAGE,
        leftpad_u32(index),
        leftpad_u24(new_leverage)
    )
}

/// Encode `updateOpenOrder(index, triggerPrice, tp, sl, maxSlippageP)`.
///
/// Updates the parameters of a pending limit or trigger open order. All prices
/// use 1e10 precision. `max_slippage_p` uses 2 decimals and should be non-zero
/// for market-type orders.
pub const fn make_fn_update_open_order(
    index: u32,
    trigger_price: u64,
    tp: u64,
    sl: u64,
    max_slippage_p: u16,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_UPDATE_OPEN_ORDER,
        leftpad_u32(index),
        leftpad_u64(trigger_price),
        leftpad_u64(tp),
        leftpad_u64(sl),
        leftpad_u16(max_slippage_p)
    )
}

/// Encode `cancelOpenOrder(index)`.
///
/// Cancels a pending limit or trigger order that has not yet been executed.
pub const fn make_fn_cancel_open_order(index: u32) -> [u8; 4 + 32] {
    concat_arrays!(SEL_CANCEL_OPEN_ORDER, leftpad_u32(index))
}

const fn leftpad_u64(value: u64) -> [u8; 32] {
    concat_arrays!([0u8; 24], value.to_be_bytes())
}

const fn leftpad_u120(value: Uint120) -> [u8; 32] {
    concat_arrays!([0u8; 17], value)
}

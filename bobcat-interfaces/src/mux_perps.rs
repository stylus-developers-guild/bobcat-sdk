//! Minimal MUX Perps core position calldata builders for the `OrderBook`
//! contract on Arbitrum.
//!
//! MUX models each trader as a sub-account identified by a packed `bytes32`
//! sub-account id (see [`make_sub_account_id`]). Position lifecycle is
//! request/fill: a trader calls `placePositionOrder3` to submit an open or
//! close request, a permissioned broker later fills it at an oracle price.
//! End users only submit and cancel orders; they never fill orders themselves.
//!
//! These builders target the `OrderBook.placePositionOrder3`,
//! `OrderBook.cancelOrder`, `OrderBook.depositCollateral`, and
//! `OrderBook.withdrawAllCollateral` entry points. The `PositionOrderExtra`
//! struct (tp/sl strategy parameters) is included in `placePositionOrder3`
//! as a trailing static ABI tuple. Callers remain responsible for collateral
//! approval, fixed-point scaling, flag construction, and deadline selection.
//!
//! ABI references:
//! - <https://github.com/mux-world/mux-protocol/blob/main/contracts/orderbook/OrderBook.sol>
//! - <https://github.com/mux-world/mux-protocol/blob/main/contracts/interfaces/IOrderBook.sol>
//! - <https://github.com/mux-world/mux-protocol/blob/main/contracts/libraries/LibSubAccount.sol>
//! - <https://github.com/mux-world/mux-protocol/blob/main/contracts/libraries/LibOrder.sol>
//!
//! All builders are `const fn`, `no_std`, and allocation-free.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_u8, leftpad_u32};

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Big-endian `uint96`, used for collateral, size, and price fields.
pub type Uint96 = [u8; 12];

selectors! {
    SEL_PLACE_POSITION_ORDER3 = b"placePositionOrder3(bytes32,uint96,uint96,uint96,uint8,uint8,uint32,bytes32,(uint96,uint96,uint8,uint32))",
    SEL_CANCEL_ORDER = b"cancelOrder(uint64)",
    SEL_DEPOSIT_COLLATERAL = b"depositCollateral(bytes32,uint256)",
    SEL_WITHDRAW_ALL_COLLATERAL = b"withdrawAllCollateral(bytes32)",
}

/// Position order flag bits from `LibOrder`.
///
/// Combine with bitwise OR and pass as the `flags` argument to
/// [`make_fn_place_position_order3`]. See `LibOrder.sol` for the exact
/// semantics; a brief summary follows.
pub mod flags {
    /// Open a position. Without this flag the order closes an existing position.
    pub const POSITION_OPEN: u8 = 0x80;
    /// Market order: ignore `price` and `deadline` (both must be zero).
    pub const POSITION_MARKET_ORDER: u8 = 0x40;
    /// Auto-withdraw all collateral once the position size reaches zero.
    pub const POSITION_WITHDRAW_ALL_IF_EMPTY: u8 = 0x20;
    /// Trigger order (e.g. stop-loss). Without this flag a non-market order is
    /// a limit order (e.g. take-profit).
    pub const POSITION_TRIGGER_ORDER: u8 = 0x10;
    /// TP/SL strategy: for opens, auto-place tp/sl orders on fill; for closes,
    /// use `extra.tp_price`/`extra.sl_price`/`extra.tpsl_profit_token_id` and
    /// ignore `price`/`profit_token_id`.
    pub const POSITION_TPSL_STRATEGY: u8 = 0x08;
    /// Enforce the asset's minimum-profit time/ratio when closing.
    pub const POSITION_SHOULD_REACH_MIN_PROFIT: u8 = 0x04;
}

/// Build a MUX sub-account id from its packed components.
///
/// Layout (see `LibSubAccount.sol`):
/// ```text
///   bits 96..256  account      (160 bits)
///   bits 88..96   collateralId (8 bits)
///   bits 80..88   assetId       (8 bits)
///   bits 72..80   isLong        (8 bits, 0 or 1)
///   bits  0..72   unused        (must be zero)
/// ```
pub const fn make_sub_account_id(
    account: Address,
    collateral_id: u8,
    asset_id: u8,
    is_long: bool,
) -> [u8; 32] {
    concat_arrays!(
        account,
        [collateral_id],
        [asset_id],
        [if is_long { 1 } else { 0 }],
        [0u8; 32 - 23]
    )
}

/// Extra tp/sl strategy parameters appended to a `placePositionOrder3` call.
///
/// Only consulted when `flags & POSITION_TPSL_STRATEGY != 0`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositionOrderExtra {
    /// Take-profit price (1e18). Zero disables tp.
    pub tp_price: Uint96,
    /// Stop-loss price (1e18). Zero disables sl.
    pub sl_price: Uint96,
    /// Profit token id used when the tp/sl leg closes profitably.
    pub tpsl_profit_token_id: u8,
    /// Unix deadline after which the tp/sl legs must not be filled.
    pub tpsl_deadline: u32,
}

/// Encode `OrderBook.placePositionOrder3(subAccountId, collateralAmount, size,
/// price, profitTokenId, flags, deadline, referralCode, extra)`.
///
/// This is the single entry point for both opening and closing leveraged
/// positions. Set `flags & POSITION_OPEN` to open; clear it to close. For
/// market orders set `price = 0`, `deadline = 0`, and `POSITION_MARKET_ORDER`.
/// For limit/trigger orders set `price` to the trigger price and `deadline`
/// to a future unix timestamp.
///
/// `collateral_amount` uses the collateral token's own decimals and is
/// deposited on open (or withdrawn on close). `size` uses 1e18 decimals.
/// `referral_code` may be all-zero to opt out. `extra` is only consulted when
/// `flags & POSITION_TPSL_STRATEGY != 0`; pass a zeroed value otherwise.
pub const fn make_fn_place_position_order3(
    sub_account_id: [u8; 32],
    collateral_amount: Uint96,
    size: Uint96,
    price: Uint96,
    profit_token_id: u8,
    flags: u8,
    deadline: u32,
    referral_code: [u8; 32],
    extra: &PositionOrderExtra,
) -> [u8; 4 + 32 * 12] {
    concat_arrays!(
        SEL_PLACE_POSITION_ORDER3,
        sub_account_id,
        leftpad_u96(collateral_amount),
        leftpad_u96(size),
        leftpad_u96(price),
        leftpad_u8(profit_token_id),
        leftpad_u8(flags),
        leftpad_u32(deadline),
        referral_code,
        // PositionOrderExtra static tuple (uint96,uint96,uint8,uint32) inlined.
        leftpad_u96(extra.tp_price),
        leftpad_u96(extra.sl_price),
        leftpad_u8(extra.tpsl_profit_token_id),
        leftpad_u32(extra.tpsl_deadline)
    )
}

/// Encode `OrderBook.cancelOrder(orderId)`.
///
/// Cancels a pending position, liquidity, or withdrawal order. Only the
/// original account owner (or an approved aggregator) may cancel before
/// expiry; brokers may cancel after expiry.
pub const fn make_fn_cancel_order(order_id: u64) -> [u8; 4 + 32] {
    concat_arrays!(SEL_CANCEL_ORDER, leftpad_u64(order_id))
}

/// Encode `OrderBook.depositCollateral(subAccountId, collateralAmount)`.
///
/// Deposits collateral into a sub-account. The caller must have approved the
/// OrderBook to spend `collateral_amount` of the sub-account's collateral
/// token. `collateral_amount` uses the collateral token's own decimals and
/// is a `uint256` on-chain.
pub const fn make_fn_deposit_collateral(
    sub_account_id: [u8; 32],
    collateral_amount: [u8; 32],
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT_COLLATERAL, sub_account_id, collateral_amount)
}

/// Encode `OrderBook.withdrawAllCollateral(subAccountId)`.
///
/// Withdraws all collateral from a sub-account whose position size is zero.
/// Only the sub-account owner may call this.
pub const fn make_fn_withdraw_all_collateral(sub_account_id: [u8; 32]) -> [u8; 4 + 32] {
    concat_arrays!(SEL_WITHDRAW_ALL_COLLATERAL, sub_account_id)
}

const fn leftpad_u96(value: Uint96) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 12], value)
}

const fn leftpad_u64(value: u64) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 8], value.to_be_bytes())
}


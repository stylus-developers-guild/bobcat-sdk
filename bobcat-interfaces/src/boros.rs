//! Narrow Boros interest-rate trading calldata builders for Arbitrum.
//!
//! Boros is Pendle's on-chain interest-rate swap (IRS) platform, deployed on
//! Arbitrum. Users take leveraged long or short positions on variable funding
//! rates through a hybrid CLOB + AMM architecture.
//!
//! All interaction goes through a single Router contract. The three builders
//! below cover the minimum end-user lifecycle:
//!
//! - `vaultDeposit` — deposit cash collateral into a market account (required
//!   before trading).
//! - `placeSingleOrder` — open or close a position via the order book. The
//!   compound `placeSingleOrder` function can auto-enter the market
//!   (`enterMarket = true`), cancel a prior order (`idToStrictCancel`), and
//!   auto-exit the market after the fill (`exitMarket = true`), so it can
//!   serve as a one-call open or close.
//! - `swapWithAmm` — open or close a position directly against the AMM without
//!   touching the order book.
//!
//! To close an existing position, call `placeSingleOrder` (or `swapWithAmm`)
//! with the opposite `Side` and a taker time-in-force (`IOC` or `FOK`). The
//! matched trade reduces the net position size.
//!
//! AMM liquidity provision, conditional orders, OTC trades, bulk orders,
//! bulk cancels, cash transfers, subaccount operations, vault withdrawal,
//! simulation, and all admin/operator functions are intentionally excluded.
//!
//! ABI reference: official interfaces at
//! <https://github.com/pendle-finance/boros-core-public/tree/main/contracts/interfaces>
//! (`ITradeModule.sol`, `IAMMModule.sol`, `IRouterEventsAndTypes.sol`) and
//! developer documentation at <https://docs.pendle.finance/boros-dev>.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_u16, leftpad_u24, leftpad_u8};
use bobcat_maths::{I, U};

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Big-endian `uint24`, used by Boros for `MarketId`.
pub type MarketId = [u8; 3];

/// Big-endian `uint24`, used by Boros for `AMMId`.
pub type AmmId = [u8; 3];

/// Big-endian `int128`, used by Boros for slippage rates.
pub type I128 = [u8; 16];

/// The special cross-margin `MarketId` (`type(uint24).max`).
pub const MARKET_ID_CROSS: MarketId = [0xFF, 0xFF, 0xFF];

/// The zero `AMMId` — order-book only, no AMM interaction.
pub const AMM_ID_ZERO: AmmId = [0, 0, 0];

selectors! {
    SEL_VAULT_DEPOSIT = b"vaultDeposit(uint8,uint16,uint24,uint256)",
    SEL_PLACE_SINGLE_ORDER = b"placeSingleOrder(((bool,uint24,uint24,uint8,uint8,uint256,int16),bool,uint64,bool,uint256,bool,int128))",
    SEL_SWAP_WITH_AMM = b"swapWithAmm((bool,uint24,int256,int128))",
}

/// Order direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Side {
    /// Buy interest-rate swap (pay fixed, receive floating).
    Long = 0,
    /// Sell interest-rate swap (receive fixed, pay floating).
    Short = 1,
}

/// Order lifetime / matching behaviour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TimeInForce {
    /// Good Till Cancel — rests on the book as a maker order.
    Gtc = 0,
    /// Immediate or Cancel — taker, unfilled portion is cancelled.
    Ioc = 1,
    /// Fill or Kill — taker, reverts if not fully filled.
    Fok = 2,
    /// Add Liquidity Only (Post-Only) — reverts if it would match.
    Alo = 3,
    /// Soft ALO — silently skips the matching portion.
    SoftAlo = 4,
}

/// Fields for the inner `OrderReq` struct.
///
/// `market_id` should be `MARKET_ID_CROSS` for cross-margin mode, or a specific
/// market ID for isolated-margin.  `amm_id` should be `AMM_ID_ZERO` for
/// order-book-only orders, or a specific AMM ID to allow AMM matching.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderReq {
    /// Cross-margin (`true`) or isolated-margin (`false`).
    pub cross: bool,
    /// Market identifier. Use `MARKET_ID_CROSS` for cross-margin.
    pub market_id: MarketId,
    /// AMM identifier. Use `AMM_ID_ZERO` for order-book only.
    pub amm_id: AmmId,
    /// Order direction.
    pub side: Side,
    /// Order lifetime / matching behaviour.
    pub tif: TimeInForce,
    /// Position size in 18-decimal precision.
    pub size: U,
    /// Price tick.
    pub tick: i16,
}

/// Encode `Router.vaultDeposit(accountId, tokenId, marketId, amount)`.
///
/// Deposits `amount` of the collateral token identified by `token_id` into the
/// market account.  For cross-margin, pass `MARKET_ID_CROSS` as `market_id`.
/// The caller must have previously approved the Router to spend the
/// underlying ERC-20 token.
pub const fn make_fn_vault_deposit(
    account_id: u8,
    token_id: u16,
    market_id: MarketId,
    amount: &U,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_VAULT_DEPOSIT,
        leftpad_u8(account_id),
        leftpad_u16(token_id),
        leftpad_u24(market_id),
        amount.0
    )
}

/// Encode `Router.placeSingleOrder(SingleOrderReq)`.
///
/// This is the primary entry point for opening or closing an interest-rate
/// position.  Set `enter_market` to `true` to auto-enter the market if not
/// already entered.  Set `exit_market` to `true` to auto-exit the market after
/// the fill (useful when the order fully closes a position).
///
/// To **open** a position: choose `Side::Long` or `Side::Short`, set `size` to
/// the desired exposure, and typically use `TimeInForce::Gtc` for a maker
/// order or `TimeInForce::Ioc` for a taker order.
///
/// To **close** a position: use the opposite `Side` from the open, set `size`
/// to the amount to reduce, and use `TimeInForce::Ioc` or `TimeInForce::Fok`
/// to ensure immediate execution without leaving a residual maker order.
///
/// `id_to_strict_cancel` may be set to `0` to skip the cancel step, or to a
/// valid `OrderId` to atomically cancel that order before placing the new one.
///
/// `isolated_cash_in` and `isolated_cash_transfer_all` are only relevant for
/// isolated-margin mode; pass `&U::ZERO` and `false` for cross-margin.
///
/// `desired_match_rate` is a slippage parameter (signed `int128`).  Pass zero
/// (`[0; 16]`) to accept any match rate.
pub const fn make_fn_place_single_order(
    order: &OrderReq,
    enter_market: bool,
    id_to_strict_cancel: u64,
    exit_market: bool,
    isolated_cash_in: &U,
    isolated_cash_transfer_all: bool,
    desired_match_rate: I128,
) -> [u8; 4 + 32 * 13] {
    concat_arrays!(
        SEL_PLACE_SINGLE_ORDER,
        // OrderReq (7 words)
        leftpad_bool(order.cross),
        leftpad_u24(order.market_id),
        leftpad_u24(order.amm_id),
        leftpad_u8(order.side as u8),
        leftpad_u8(order.tif as u8),
        order.size.0,
        leftpad_i16(order.tick),
        // SingleOrderReq remaining fields (6 words)
        leftpad_bool(enter_market),
        leftpad_u64(id_to_strict_cancel),
        leftpad_bool(exit_market),
        isolated_cash_in.0,
        leftpad_bool(isolated_cash_transfer_all),
        leftpad_i128(desired_match_rate)
    )
}

/// Encode `Router.swapWithAmm(SwapWithAmmReq)`.
///
/// Swaps directly against the AMM, bypassing the order book.  A positive
/// `signed_size` opens a long position; a negative `signed_size` opens a short
/// position (or closes an existing long).  To close, pass the opposite sign of
/// the open.
///
/// `desired_swap_rate` is a slippage parameter (signed `int128`).  Pass zero
/// (`[0; 16]`) to accept any swap rate.
pub const fn make_fn_swap_with_amm(
    cross: bool,
    amm_id: AmmId,
    signed_size: &I,
    desired_swap_rate: I128,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_SWAP_WITH_AMM,
        leftpad_bool(cross),
        leftpad_u24(amm_id),
        signed_size.0,
        leftpad_i128(desired_swap_rate)
    )
}

// ---------------------------------------------------------------------------
// Local encoding helpers
// ---------------------------------------------------------------------------

const fn leftpad_bool(value: bool) -> [u8; 32] {
    leftpad_u8(value as u8)
}

const fn leftpad_u64(value: u64) -> [u8; 32] {
    concat_arrays!([0u8; 24], value.to_be_bytes())
}

/// Sign-extend a signed 16-bit integer to a 32-byte ABI word.
const fn leftpad_i16(value: i16) -> [u8; 32] {
    let bytes = value.to_be_bytes();
    let mut out = [0u8; 32];
    out[30] = bytes[0];
    out[31] = bytes[1];
    if value < 0 {
        let mut i = 0;
        while i < 30 {
            out[i] = 0xFF;
            i += 1;
        }
    }
    out
}

/// Sign-extend a signed 128-bit big-endian value to a 32-byte ABI word.
const fn leftpad_i128(value: I128) -> [u8; 32] {
    let out: [u8; 32] = concat_arrays!([0u8; 16], value);
    if value[0] & 0x80 != 0 {
        let mut out = out;
        let mut i = 0;
        while i < 16 {
            out[i] = 0xFF;
            i += 1;
        }
        out
    } else {
        out
    }
}

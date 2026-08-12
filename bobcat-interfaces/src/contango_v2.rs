//! Core end-user calldata builders for Contango V2 leveraged trading.
//!
//! Contango V2 is a looping/leveraged-position protocol that builds positions
//! by automating spot-and-money-market strategies. On Arbitrum it is deployed
//! behind the `ContangoProxy` and exposes a single entrypoint — `trade` — that
//! handles opening, modifying, and closing leveraged positions.
//!
//! This module exposes only the two core end-user trading entrypoints:
//!
//! - `trade` — open, modify, or close a position. When `quantity > 0` a new
//!   position is opened (or an existing one increased); when `quantity < 0`
//!   an existing position is closed (partially or fully); when `quantity == 0`
//!   the position is modified (cashflow only, no size change).
//! - `tradeOnBehalfOf` — same as `trade` but executed on behalf of a
//!   specified address (requires prior approval from the position owner).
//!
//! All admin, pause/unpause, instrument creation, callback, reward-claiming,
//! and position-donation functions are permissioned or operator-only and are
//! intentionally **not** exposed here.
//!
//! # Position encoding
//!
//! A Contango `PositionId` is a packed `bytes32`:
//!
//! ```text
//!  16B          1B           4B        1B     6B
//!  symbol - moneyMarketId - expiry - flags - number
//! ```
//!
//! - **symbol** (`bytes16`): the trading instrument, e.g. `WETHUSDC`.
//! - **moneyMarketId** (`uint8`): the lending market (Aave, Spark, Morpho,
//!   Compound, Euler, Fluid, etc.).
//! - **expiry** (`uint32`): `type(uint32).max` for perps, or a future timestamp
//!   for expiring instruments. Must be non-zero.
//! - **flags** (`bytes1`): money-market-specific flags (e.g. Aave E-mode /
//!   isolation mode). Usually `0` for non-Aave markets.
//! - **number** (`uint48`): `0` to open a new position; a non-zero existing
//!   position number to modify/close.
//!
//! See the Contango V2 `PositionIdExt.sol` for the canonical encoding.
//!
//! # Cashflow
//!
//! `cashflow` is signed: positive means the user is putting collateral *into*
//! the position; negative means the user is pulling collateral *out*.
//! `cashflow_ccy` specifies whether the cashflow is denominated in `Base`
//! (the collateral token, e.g. WETH) or `Quote` (the borrowed token, e.g.
//! USDC). Use `Currency::None` (0) only when no cashflow is involved.
//!
//! # Execution params
//!
//! `ExecutionParams` describes how the spot swap is executed:
//!
//! - `spender` — address approved to pull the swap input token.
//! - `router` — the DEX router to call.
//! - `swap_amount` — the amount of the swap input token to sell.
//! - `swap_bytes` — opaque router-specific calldata.
//! - `flash_loan_provider` — the flash-loan provider address, or zero to
//!   bypass the flash loan (only valid when the user's own cashflow covers
//!   the entire swap).
//!
//! When no swap is needed (e.g. pure cashflow modification), pass
//! `swap_amount = 0`, an empty `swap_bytes`, and `flash_loan_provider = [0; 20]`.
//!
//! # ABI
//!
//! `trade((bytes32,int256,uint256,uint8,int256),(address,address,uint256,bytes,address))`
//! — `TradeParams` is a static tuple (5 head words); `ExecutionParams` is a
//! dynamic tuple (because of the inner `bytes swapBytes`) and is encoded as an
//! offset + tail.
//!
//! `tradeOnBehalfOf((bytes32,int256,uint256,uint8,int256),(address,address,uint256,bytes,address),address)`
//! — same as `trade` with an additional static `address onBehalfOf` in the
//! head.
//!
//! ABI verified against the official Contango V2 source:
//! - `src/interfaces/IContango.sol` (`IContango.trade`, `IContango.tradeOnBehalfOf`)
//! - `src/libraries/DataTypes.sol` (`PositionId`, `Symbol`, `Currency`)
//! - `src/libraries/extensions/PositionIdExt.sol` (position-id encoding)
//! - `test/Encoder.sol` (`encode` helper)
//!
//! Source: <https://github.com/contango-xyz/core-v2>

use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// A Contango position identifier — a packed `bytes32`.
pub type PositionId = [u8; 32];

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
}

/// Currency denomination for cashflow.
///
/// Matches the `Currency` enum in Contango's `DataTypes.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Currency {
    /// No cashflow.
    None = 0,
    /// Cashflow denominated in the base (collateral) token.
    Base = 1,
    /// Cashflow denominated in the quote (borrowed) token.
    Quote = 2,
}

impl Currency {
    /// Convert to the `uint8` ABI value.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

selectors! {
    SEL_TRADE = b"trade((bytes32,int256,uint256,uint8,int256),(address,address,uint256,bytes,address))",
    SEL_TRADE_ON_BEHALF_OF = b"tradeOnBehalfOf((bytes32,int256,uint256,uint8,int256),(address,address,uint256,bytes,address),address)",
}

// --------------------------------------------------------------------------- //
// Position-id encoding
// --------------------------------------------------------------------------- //

/// Encode a Contango V2 `PositionId`.
///
/// Layout (32 bytes, big-endian):
///
/// | bytes 0-15  | byte 16    | bytes 17-20 | byte 21    | bytes 22-27     |
/// |-------------|------------|-------------|------------|-----------------|
/// | symbol      | mm id      | expiry      | flags      | number          |
/// | `bytes16`   | `uint8`    | `uint32`    | `bytes1`   | `uint48`        |
///
/// Pass `number = 0` to open a new position; pass the existing position's
/// number to modify or close it.
///
/// `expiry` must be non-zero — use `0xFFFFFFFF` (`u32::MAX`) for perps.
#[rustfmt::skip]
pub const fn encode_position_id(
    symbol: [u8; 16],
    money_market_id: u8,
    expiry: u32,
    flags: u8,
    number: u64,
) -> PositionId {
    let mut id = [0u8; 32];
    // bytes 0-15: symbol
    id[0] = symbol[0];   id[1] = symbol[1];   id[2] = symbol[2];   id[3] = symbol[3];
    id[4] = symbol[4];   id[5] = symbol[5];   id[6] = symbol[6];   id[7] = symbol[7];
    id[8] = symbol[8];   id[9] = symbol[9];   id[10] = symbol[10]; id[11] = symbol[11];
    id[12] = symbol[12]; id[13] = symbol[13]; id[14] = symbol[14]; id[15] = symbol[15];
    // byte 16: money market id
    id[16] = money_market_id;
    // bytes 17-20: expiry (big-endian u32)
    id[17] = (expiry >> 24) as u8;
    id[18] = (expiry >> 16) as u8;
    id[19] = (expiry >> 8) as u8;
    id[20] = expiry as u8;
    // byte 21: flags
    id[21] = flags;
    // bytes 22-27: number (big-endian u48, 6 bytes)
    id[22] = (number >> 40) as u8;
    id[23] = (number >> 32) as u8;
    id[24] = (number >> 24) as u8;
    id[25] = (number >> 16) as u8;
    id[26] = (number >> 8) as u8;
    id[27] = number as u8;
    // bytes 28-31: unused (zero)
    id
}

// --------------------------------------------------------------------------- //
// Buffer-writing helpers
// --------------------------------------------------------------------------- //

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

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let mut word = [0u8; 32];
    let bytes = value.to_be_bytes();
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    put_word(output, offset, &word);
}

/// Write a `bytes` tail at `offset`: length word + data right-padded to a
/// multiple of 32. Returns the number of bytes consumed.
fn put_bytes_tail(output: &mut [u8], offset: usize, data: &[u8]) -> usize {
    put_usize(output, offset, data.len());
    let data_start = offset + 32;
    let padded_len = data.len().div_ceil(32) * 32;
    output[data_start..data_start + data.len()].copy_from_slice(data);
    32 + padded_len
}

// --------------------------------------------------------------------------- //
// Calldata length helpers
// --------------------------------------------------------------------------- //

/// Required output length for [`make_fn_trade`].
///
/// Layout:
/// ```text
/// selector(4)
/// + TradeParams head: 5 words (positionId, quantity, limitPrice, cashflowCcy, cashflow)
/// + offset to ExecutionParams: 1 word
/// + ExecutionParams tail:
///     5 head words (spender, router, swapAmount, swapBytes-offset, flashLoanProvider)
///     + swapBytes tail: length(32) + ceil(swap_bytes_len/32)*32
/// ```
pub fn trade_calldata_len(swap_bytes_len: usize) -> Result<usize, EncodeError> {
    let swap_padded = swap_bytes_len.div_ceil(32) * 32;
    4_usize
        .checked_add(32 * 5) // TradeParams (static, inline)
        .and_then(|s| s.checked_add(32))     // offset to ExecutionParams
        .and_then(|s| s.checked_add(32 * 5)) // ExecutionParams head
        .and_then(|s| s.checked_add(32 + swap_padded)) // swapBytes tail
        .ok_or(EncodeError::BufferTooSmall { required: 0 })
}

/// Required output length for [`make_fn_trade_on_behalf_of`].
///
/// Same as [`trade_calldata_len`] plus one extra head word for `onBehalfOf`.
pub fn trade_on_behalf_of_calldata_len(swap_bytes_len: usize) -> Result<usize, EncodeError> {
    let swap_padded = swap_bytes_len.div_ceil(32) * 32;
    4_usize
        .checked_add(32 * 5) // TradeParams (static, inline)
        .and_then(|s| s.checked_add(32))     // offset to ExecutionParams
        .and_then(|s| s.checked_add(32))     // onBehalfOf (static, inline)
        .and_then(|s| s.checked_add(32 * 5)) // ExecutionParams head
        .and_then(|s| s.checked_add(32 + swap_padded)) // swapBytes tail
        .ok_or(EncodeError::BufferTooSmall { required: 0 })
}

// --------------------------------------------------------------------------- //
// TradeParams / ExecutionParams
// --------------------------------------------------------------------------- //

/// Parameters for a Contango V2 `trade` / `tradeOnBehalfOf` call.
///
/// All fields are static ABI types and are encoded inline in the calldata
/// head as the `TradeParams` tuple.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeParams<'a> {
    /// Existing position id, or a newly-encoded one with `number = 0` to open.
    pub position_id: PositionId,
    /// Signed quantity: positive to open/increase, negative to close, zero to
    /// modify cashflow only.
    pub quantity: &'a U,
    /// Limit price in the quote currency; pass `0` to skip the price check.
    pub limit_price: &'a U,
    /// Which currency the cashflow is denominated in.
    pub cashflow_ccy: Currency,
    /// Signed cashflow: positive = collateral in, negative = collateral out.
    pub cashflow: &'a U,
}

/// Execution parameters describing the spot swap.
///
/// Because `swap_bytes` is a dynamic `bytes` field, `ExecutionParams` is
/// encoded as a dynamic tuple (offset + tail).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionParams<'a> {
    /// Address approved to pull the swap input token from the Contango contract.
    pub spender: Address,
    /// The DEX router to call.
    pub router: Address,
    /// Amount of the swap input token to sell.
    pub swap_amount: &'a U,
    /// Opaque router-specific calldata (may be empty).
    pub swap_bytes: &'a [u8],
    /// Flash-loan provider address, or `[0; 20]` to bypass.
    pub flash_loan_provider: Address,
}

// --------------------------------------------------------------------------- //
// trade()
// --------------------------------------------------------------------------- //

/// Encode `trade(TradeParams, ExecutionParams)`.
///
/// This is the core Contango V2 entrypoint for opening, modifying, and
/// closing leveraged positions.
///
/// **Opening**: set `quantity > 0` and `position_id` with `number = 0`. The
/// protocol mints a new position NFT to `msg.sender`.
///
/// **Closing**: set `quantity < 0` (use the negative of the desired size) and
/// `position_id` with the existing position number. A full close (quantity
/// >= collateral balance) burns the position NFT.
///
/// **Modify**: set `quantity = 0` and adjust `cashflow`/`cashflow_ccy`.
///
/// The caller must supply a buffer of at least
/// [`trade_calldata_len`]`(exec.swap_bytes.len())` bytes. Returns the number
/// of bytes written.
pub fn make_fn_trade(
    output: &mut [u8],
    trade_params: &TradeParams,
    exec: &ExecutionParams,
) -> Result<usize, EncodeError> {
    let required = trade_calldata_len(exec.swap_bytes.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_TRADE);

    // --- Head: TradeParams (5 words, inline) + offset to ExecutionParams ---
    let head_base = 4;
    // Word 0: positionId (bytes32, written as-is)
    put_word(out, head_base, &trade_params.position_id);
    // Word 1: quantity (int256)
    put_word(out, head_base + 32, &trade_params.quantity.0);
    // Word 2: limitPrice (uint256)
    put_word(out, head_base + 64, &trade_params.limit_price.0);
    // Word 3: cashflowCcy (uint8)
    put_u8(out, head_base + 96, trade_params.cashflow_ccy.as_u8());
    // Word 4: cashflow (int256)
    put_word(out, head_base + 128, &trade_params.cashflow.0);

    // Word 5: offset to ExecutionParams (relative to start of args after selector)
    // The exec tail starts right after the 6 head words.
    let exec_offset = 32 * 6; // = 192
    put_usize(out, head_base + 160, exec_offset);

    // --- ExecutionParams tail ---
    let exec_base = head_base + exec_offset;
    // 5 head words of the exec tuple:
    put_addr(out, exec_base, exec.spender); // word 0
    put_addr(out, exec_base + 32, exec.router); // word 1
    put_word(out, exec_base + 64, &exec.swap_amount.0); // word 2: swapAmount
    // word 3: offset to swapBytes (relative to start of exec tuple) = 5*32 = 160
    put_usize(out, exec_base + 96, 32 * 5);
    put_addr(out, exec_base + 128, exec.flash_loan_provider); // word 4

    // swapBytes tail
    let sb_base = exec_base + 32 * 5;
    let written = put_bytes_tail(out, sb_base, exec.swap_bytes);

    debug_assert_eq!(sb_base + written, required);
    Ok(required)
}

// --------------------------------------------------------------------------- //
// tradeOnBehalfOf()
// --------------------------------------------------------------------------- //

/// Encode `tradeOnBehalfOf(TradeParams, ExecutionParams, address)`.
///
/// Same as [`make_fn_trade`] but executes on behalf of `on_behalf_of`. The
/// position owner must have granted permission to the caller (via
/// `PositionNFT.permit` or `setApprovalForAll`).
///
/// The caller must supply a buffer of at least
/// [`trade_on_behalf_of_calldata_len`]`(exec.swap_bytes.len())` bytes.
/// Returns the number of bytes written.
pub fn make_fn_trade_on_behalf_of(
    output: &mut [u8],
    trade_params: &TradeParams,
    exec: &ExecutionParams,
    on_behalf_of: Address,
) -> Result<usize, EncodeError> {
    let required = trade_on_behalf_of_calldata_len(exec.swap_bytes.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_TRADE_ON_BEHALF_OF);

    // --- Head ---
    // TradeParams (5 words, inline) + offset to exec + onBehalfOf (1 word)
    let head_base = 4;
    // Word 0: positionId
    put_word(out, head_base, &trade_params.position_id);
    // Word 1: quantity
    put_word(out, head_base + 32, &trade_params.quantity.0);
    // Word 2: limitPrice
    put_word(out, head_base + 64, &trade_params.limit_price.0);
    // Word 3: cashflowCcy
    put_u8(out, head_base + 96, trade_params.cashflow_ccy.as_u8());
    // Word 4: cashflow
    put_word(out, head_base + 128, &trade_params.cashflow.0);
    // Word 5: offset to ExecutionParams
    // Head: 5 (TradeParams) + 1 (exec offset) + 1 (onBehalfOf) = 7 words
    let exec_offset = 32 * 7; // = 224
    put_usize(out, head_base + 160, exec_offset);
    // Word 6: onBehalfOf
    put_addr(out, head_base + 192, on_behalf_of);

    // --- ExecutionParams tail ---
    let exec_base = head_base + exec_offset;
    put_addr(out, exec_base, exec.spender); // word 0
    put_addr(out, exec_base + 32, exec.router); // word 1
    put_word(out, exec_base + 64, &exec.swap_amount.0); // word 2: swapAmount
    put_usize(out, exec_base + 96, 32 * 5); // word 3: offset to swapBytes = 160
    put_addr(out, exec_base + 128, exec.flash_loan_provider); // word 4

    // swapBytes tail
    let sb_base = exec_base + 32 * 5;
    let written = put_bytes_tail(out, sb_base, exec.swap_bytes);

    debug_assert_eq!(sb_base + written, required);
    Ok(required)
}


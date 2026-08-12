//! Core end-user calldata builders for Ondo Finance yield-asset mint and
//! redeem flows.
//!
//! Ondo Finance issues two yield-bearing tokens — **USDY** (US Dollar Yield) and
//! **OUSG** (Ondo Short-Term US Government Bond) — that are backed by short-term
//! US Treasuries and related instruments.  Both are available to qualifying
//! non-US investors through the Ondo web app and, programmatically, through a
//! dedicated on-chain manager contract.
//!
//! ## Mint / redeem flow
//!
//! The `USDY_InstantManager` is the on-chain entry point used by the Ondo web
//! app and by third-party integrators to instantly mint and redeem USDY.  It
//! exposes two core end-user functions:
//!
//! - `subscribe(address depositToken, uint256 depositAmount, uint256
//!   minimumRwaReceived)` — deposit `depositAmount` of a supported stablecoin
//!   (USDC, PYUSD, or RLUSD) and receive newly minted USDY at the current oracle
//!   price.
//! - `redeem(uint256 rwaAmount, address receivingToken, uint256
//!   minimumTokenReceived)` — burn `rwaAmount` of USDY and receive the
//!   corresponding amount of a supported stablecoin.
//!
//! The OUSG flow uses the `OUSG_InstantManager` with the same two function
//! signatures.
//!
//! ## Prerequisites
//!
//! The calling address must be registered in the OndoIDRegistry before any
//! mint or redeem call; unregistered addresses will revert with
//! `UserNotRegistered`.  The manager pulls tokens via `transferFrom`, so the
//! caller must approve the manager to spend the deposit token (before
//! `subscribe`) or USDY (before `redeem`).
//!
//! ## Permissioning
//!
//! `subscribe` and `redeem` are end-user functions callable by any registered
//! address.  Pause/unpause, rate-limit configuration, oracle management,
//! blocklist administration, and role management are permissioned operator
//! functions and are intentionally **not** exposed here.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official Ondo
//! developer documentation
//! (<https://docs.ondo.finance/developer-guides/usdy-instant-manager-integration>)
//! and the deployed `USDY_InstantManager` contract on Ethereum mainnet.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_SUBSCRIBE = b"subscribe(address,uint256,uint256)",
    SEL_REDEEM = b"redeem(uint256,address,uint256)",
}

/// Encode `subscribe(depositToken, depositAmount, minimumRwaReceived)`.
///
/// This is the canonical end-user mint flow for Ondo yield assets: deposit a
/// supported stablecoin (USDC, PYUSD, or RLUSD) and receive newly minted USDY
/// or OUSG at the current oracle price.
///
/// `deposit_token` is the address of the stablecoin to deposit.
/// `deposit_amount` is denominated in the deposit token's native decimals
/// (e.g. `100_000_000` for 100 USDC with 6 decimals).
/// `minimum_rwa_received` is slippage protection, denominated in the yield
/// token's 18 decimals.  Because minting is direct with Ondo (no AMM), there
/// is no front-running risk and `0` is safe.
///
/// The caller must have approved the manager contract to spend
/// `deposit_amount` of `deposit_token` before sending this calldata, and the
/// caller's address must be registered in the OndoIDRegistry.
pub const fn make_fn_subscribe(
    deposit_token: Address,
    deposit_amount: &U,
    minimum_rwa_received: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_SUBSCRIBE,
        leftpad_addr(deposit_token),
        deposit_amount.0,
        minimum_rwa_received.0
    )
}

/// Encode `redeem(rwaAmount, receivingToken, minimumTokenReceived)`.
///
/// This is the canonical end-user redeem flow for Ondo yield assets: burn
/// `rwa_amount` of the yield token (USDY or OUSG) and receive the corresponding
/// amount of a supported stablecoin.
///
/// `rwa_amount` is denominated in the yield token's 18 decimals
/// (e.g. `100_000_000_000_000_000_000` for 100 USDY).
/// `receiving_token` is the address of the stablecoin to receive (USDC, PYUSD,
/// or RLUSD).
/// `minimum_token_received` is slippage protection, denominated in the
/// receiving token's native decimals.  Because redemption is direct with Ondo,
/// `0` is safe.
///
/// The caller must have approved the manager contract to spend `rwa_amount` of
/// the yield token before sending this calldata, and the caller's address must
/// be registered in the OndoIDRegistry.
pub const fn make_fn_redeem(
    rwa_amount: &U,
    receiving_token: Address,
    minimum_token_received: &U,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM,
        rwa_amount.0,
        leftpad_addr(receiving_token),
        minimum_token_received.0
    )
}

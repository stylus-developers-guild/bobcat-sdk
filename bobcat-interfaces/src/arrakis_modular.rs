//! Arrakis Modular public-vault calldata builders for Arbitrum.
//!
//! Arrakis Modular is a universal Meta Vault standard that allows LPs to deposit
//! into actively managed vaults. Public vaults are accessed through the
//! `ArrakisPublicVaultRouterV2` contract, which handles token transfers, share
//! minting, and slippage protection.
//!
//! This module exposes only the four core end-user entrypoints:
//!
//! - `addLiquidity(AddLiquidityData)` — deposits token0 and token1 into a public
//!   vault and mints LP shares to the receiver. Requires prior ERC-20 approval
//!   of both tokens to the router.
//! - `removeLiquidity(RemoveLiquidityData)` — burns LP shares and sends the
//!   underlying token0 and token1 to the receiver. Requires prior ERC-20
//!   approval of the LP shares to the router.
//! - `getMintAmounts(address, uint256, uint256)` — view helper that quotes the
//!   shares and token amounts for a prospective deposit.
//! - `getBurnAmounts(address, uint256)` — view helper that quotes the token0 and
//!   token1 amounts for a prospective share burn.
//!
//! The Permit2 variants, swap-and-add variants, and wrap-and-add variants are
//! not included — they require dynamic arrays for Permit2 details and swap
//! payload data that cannot be constructed without heap allocation. Admin
//! and operator functions (`pause`, `unpause`, `setResolvers`,
//! `updateSwapExecutor`, etc.) are intentionally excluded.
//!
//! ABI source: official Arrakis documentation and `ArrakisPublicVaultRouterV2`
//! interface at
//! <https://docs.arrakis.fi/arrakis-modular/technical_reference/routers/arrakis_public_vault_router_v2.html>.
//!
//! Struct definitions verified at:
//! - <https://docs.arrakis.fi/arrakis-modular/technical_reference/routers/structs/struct.AddLiquidityData.html>
//! - <https://docs.arrakis.fi/arrakis-modular/technical_reference/routers/structs/struct.RemoveLiquidityData.html>
//!
//! Flow references:
//! - <https://docs.arrakis.fi/arrakis-modular/quickstart/public_vaults.html>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_ADD_LIQUIDITY = b"addLiquidity((uint256,uint256,uint256,uint256,uint256,address,address))",
    SEL_REMOVE_LIQUIDITY = b"removeLiquidity((uint256,uint256,uint256,address,address))",
    SEL_GET_MINT_AMOUNTS = b"getMintAmounts(address,uint256,uint256)",
    SEL_GET_BURN_AMOUNTS = b"getBurnAmounts(address,uint256)",
}

// ---------------------------------------------------------------------------
// AddLiquidityData struct
// ---------------------------------------------------------------------------

/// Parameters for `addLiquidity` on the Arrakis Public Vault Router V2.
///
/// All fields are static (uint256 / address), so the struct encodes as a
/// fixed-size ABI tuple with no dynamic data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddLiquidityData {
    /// Maximum amount of token0 to transfer from `msg.sender` to the vault.
    pub amount0_max: U,
    /// Maximum amount of token1 to transfer from `msg.sender` to the vault.
    pub amount1_max: U,
    /// Minimum amount of token0 to transfer (slippage protection).
    pub amount0_min: U,
    /// Minimum amount of token1 to transfer (slippage protection).
    pub amount1_min: U,
    /// Minimum vault shares to receive (slippage protection).
    pub amount_shares_min: U,
    /// Address of the public vault to add liquidity to.
    pub vault: Address,
    /// Address that receives the minted vault shares.
    pub receiver: Address,
}

/// Encode `addLiquidity(AddLiquidityData)` calldata for the Arrakis Public
/// Vault Router V2.
///
/// The caller must have approved the router to spend `amount0_max` of token0
/// and `amount1_max` of token1. The router calls `getMintAmounts` internally to
/// determine the actual deposit amounts (capped by the max values), transfers
/// those amounts from `msg.sender`, and mints vault shares to `receiver`.
///
/// The vault composition may change between submission and execution; always
/// set `amount0_min`, `amount1_min`, and `amount_shares_min` to appropriate
/// slippage tolerances.
pub const fn make_fn_add_liquidity(data: &AddLiquidityData) -> [u8; 4 + 32 * 7] {
    concat_arrays!(
        SEL_ADD_LIQUIDITY,
        data.amount0_max.0,
        data.amount1_max.0,
        data.amount0_min.0,
        data.amount1_min.0,
        data.amount_shares_min.0,
        leftpad_addr(data.vault),
        leftpad_addr(data.receiver)
    )
}

// ---------------------------------------------------------------------------
// RemoveLiquidityData struct
// ---------------------------------------------------------------------------

/// Parameters for `removeLiquidity` on the Arrakis Public Vault Router V2.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoveLiquidityData {
    /// Amount of vault shares to burn.
    pub burn_amount: U,
    /// Minimum amount of token0 to receive (slippage protection).
    pub amount0_min: U,
    /// Minimum amount of token1 to receive (slippage protection).
    pub amount1_min: U,
    /// Address of the public vault to remove liquidity from.
    pub vault: Address,
    /// Address that receives the withdrawn token0 and token1.
    pub receiver: Address,
}

/// Encode `removeLiquidity(RemoveLiquidityData)` calldata for the Arrakis Public
/// Vault Router V2.
///
/// The caller must have approved the router to spend `burn_amount` of the
/// vault's LP shares. The router burns the shares, withdraws the underlying
/// token0 and token1 from the vault, and transfers them to `receiver`.
pub const fn make_fn_remove_liquidity(data: &RemoveLiquidityData) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_REMOVE_LIQUIDITY,
        data.burn_amount.0,
        data.amount0_min.0,
        data.amount1_min.0,
        leftpad_addr(data.vault),
        leftpad_addr(data.receiver)
    )
}

// ---------------------------------------------------------------------------
// getMintAmounts — view quote for deposit
// ---------------------------------------------------------------------------

/// Encode `getMintAmounts(address vault, uint256 maxAmount0, uint256 maxAmount1)`
/// calldata.
///
/// This is a view function: call it on the router to determine how many vault
/// shares you would receive and how much token0/token1 you need to deposit,
/// given the maximum amounts you are willing to provide. Use the returned
/// values to populate [`AddLiquidityData`] before calling
/// [`make_fn_add_liquidity`].
pub const fn make_fn_get_mint_amounts(
    vault: Address,
    max_amount0: &U,
    max_amount1: &U
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_GET_MINT_AMOUNTS,
        leftpad_addr(vault),
        max_amount0.0,
        max_amount1.0
    )
}

// ---------------------------------------------------------------------------
// getBurnAmounts — view quote for withdrawal
// ---------------------------------------------------------------------------

/// Encode `getBurnAmounts(address vault, uint256 shares)` calldata.
///
/// This is a view function: call it on the router to determine how much token0
/// and token1 you would receive for burning a given amount of vault shares.
/// Use the returned values to set slippage limits in [`RemoveLiquidityData`]
/// before calling [`make_fn_remove_liquidity`].
pub const fn make_fn_get_burn_amounts(vault: Address, shares: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_GET_BURN_AMOUNTS, leftpad_addr(vault), shares.0)
}

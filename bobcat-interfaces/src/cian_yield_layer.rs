//! Calldata builders for CIAN Yield Layer vault flows on Arbitrum.
//!
//! CIAN Yield Layer is a vault protocol that redistributes yield across
//! strategies to optimise APY.  Vaults are ERC-4626 compatible — they accept
//! deposits of an underlying asset and issue share tokens (LP tokens) to
//! depositors, and allow redemption of shares for the underlying asset.
//!
//! In addition to the standard ERC-4626 entry points, CIAN vaults expose
//! `optionalDeposit` and `optionalRedeem` which let the caller choose the
//! ERC-20 token to deposit / receive, and `requestRedeem` for asynchronous
//! (queued) redemptions.
//!
//! This module exposes only the core end-user calls:
//!
//!  | Flow                  | Function           | Notes                          |
//!  |-----------------------|--------------------|--------------------------------|
//!  | Deposit (ERC-4626)    | `deposit`          | `receiver` defaults to caller  |
//!  | Deposit (multi-token) | `optionalDeposit`  | Choose token + referral        |
//!  | Redeem (ERC-4626)     | `redeem`           | Burn shares → assets            |
//!  | Redeem (multi-token)  | `optionalRedeem`   | Choose output token             |
//!  | Withdraw (ERC-4626)   | `withdraw`         | Specify assets to withdraw     |
//!  | Mint (ERC-4626)       | `mint`             | Specify shares to mint         |
//!  | Async redeem request  | `requestRedeem`    | Queue shares for later settle  |
//!
//! Permissioned admin / operator functions (`updateExchangePrice`,
//! `createStrategy`, `removeStrategy`, `pause`, `unpause`, `sweep`,
//! `collectRevenue`, `collectManagementFee`, `transferToStrategy`,
//! `stakeTo`, `transferOwnership`, `renounceOwnership`, fee / capacity /
//! rate configuration, rebalancer / redeem-operator management, etc.) are
//! intentionally excluded.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official CIAN
//! Yield Layer vault ABI, extracted from the CIAN front-end application at
//! <https://yieldlayer.cian.app> (production JS bundle).  The ABI defines a
//! single vault contract (`Vault`) deployed on Arbitrum (chainId 42161) and
//! other chains.
//!
//! All function arguments are static ABI types (address, uint256), so the
//! entire module is `no_std` without `alloc`.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    // ERC-4626 standard
    SEL_DEPOSIT = b"deposit(uint256,address)",
    SEL_REDEEM = b"redeem(uint256,address,address)",
    SEL_WITHDRAW = b"withdraw(uint256,address,address)",
    SEL_MINT = b"mint(uint256,address)",

    // CIAN-specific vault extensions
    SEL_OPTIONAL_DEPOSIT = b"optionalDeposit(address,uint256,address,address)",
    SEL_OPTIONAL_REDEEM = b"optionalRedeem(address,uint256,uint256,address,address)",
    SEL_REQUEST_REDEEM = b"requestRedeem(uint256,address)",
}

// ---------------------------------------------------------------------------
// ERC-4626 deposit
// ---------------------------------------------------------------------------

/// Encode `deposit(assets, receiver)` on a CIAN Yield Layer vault.
///
/// Transfers `assets` of the underlying token from `msg.sender` into the
/// vault and mints vault share tokens to `receiver`.  The caller must have
/// pre-approved the vault to spend the underlying token (via ERC-20
/// `approve`).  Returns the number of shares minted.
///
/// `assets` is the amount of the underlying token to deposit, denominated in
/// the token's decimals.
/// `receiver` is the address that receives the minted vault share tokens;
/// it may differ from `msg.sender`.
pub const fn make_fn_deposit(assets: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, assets.0, leftpad_addr(receiver))
}

// ---------------------------------------------------------------------------
// ERC-4626 mint
// ---------------------------------------------------------------------------

/// Encode `mint(shares, receiver)` on a CIAN Yield Layer vault.
///
/// Deposits the exact amount of underlying token needed to mint `shares`
/// vault share tokens, and mints those shares to `receiver`.  The caller
/// must have pre-approved the vault to spend the underlying token.  Returns
/// the amount of assets deposited.
///
/// `shares` is the number of vault share tokens to mint, denominated in the
/// vault share token's decimals (typically 18).
/// `receiver` is the address that receives the minted vault share tokens.
pub const fn make_fn_mint(shares: &U, receiver: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_MINT, shares.0, leftpad_addr(receiver))
}

// ---------------------------------------------------------------------------
// ERC-4626 withdraw
// ---------------------------------------------------------------------------

/// Encode `withdraw(assets, receiver, owner)` on a CIAN Yield Layer vault.
///
/// Burns vault share tokens from `owner` and transfers `assets` of the
/// underlying token to `receiver`.  If `owner` is not `msg.sender`, the
/// caller must have an ERC-20 allowance from `owner` for the vault share
/// tokens.  Returns the number of shares burned.
///
/// `assets` is the amount of the underlying token to withdraw, denominated
/// in the token's decimals.
/// `receiver` is the address that receives the withdrawn underlying tokens.
/// `owner` is the address whose vault share tokens are burned.
pub const fn make_fn_withdraw(
    assets: &U,
    receiver: Address,
    owner: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(SEL_WITHDRAW, assets.0, leftpad_addr(receiver), leftpad_addr(owner))
}

// ---------------------------------------------------------------------------
// ERC-4626 redeem
// ---------------------------------------------------------------------------

/// Encode `redeem(shares, receiver, owner)` on a CIAN Yield Layer vault.
///
/// Burns `shares` vault share tokens from `owner` and transfers the
/// corresponding amount of underlying token to `receiver`.  If `owner` is
/// not `msg.sender`, the caller must have an ERC-20 allowance from `owner`.
/// Returns the amount of underlying assets returned.
///
/// `shares` is the number of vault share tokens to burn, denominated in the
/// vault share token's decimals.
/// `receiver` is the address that receives the underlying tokens.
/// `owner` is the address whose vault share tokens are burned.
pub const fn make_fn_redeem(
    shares: &U,
    receiver: Address,
    owner: Address,
) -> [u8; 4 + 32 * 3] {
    concat_arrays!(SEL_REDEEM, shares.0, leftpad_addr(receiver), leftpad_addr(owner))
}

// ---------------------------------------------------------------------------
// CIAN optionalDeposit — deposit with token choice + referral
// ---------------------------------------------------------------------------

/// Encode `optionalDeposit(token, assets, receiver, referral)` on a CIAN
/// Yield Layer vault.
///
/// This is CIAN's extended deposit function that allows the caller to specify
/// which ERC-20 token to deposit.  For native ETH deposits, pass the zero
/// address as `token` and send ETH as `msg.value`.  For ERC-20 deposits, the
/// caller must have pre-approved the vault to spend `assets` of `token`.
/// Returns the number of shares minted.
///
/// `token` is the address of the ERC-20 token to deposit.  Pass the zero
/// address (`[0u8; 20]`) to deposit native ETH.
/// `assets` is the amount to deposit, denominated in `token`'s decimals.
/// `receiver` is the address that receives the minted vault share tokens.
/// `referral` is an optional referral address; pass the zero address if not
/// used.
pub const fn make_fn_optional_deposit(
    token: Address,
    assets: &U,
    receiver: Address,
    referral: Address,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_OPTIONAL_DEPOSIT,
        leftpad_addr(token),
        assets.0,
        leftpad_addr(receiver),
        leftpad_addr(referral)
    )
}

// ---------------------------------------------------------------------------
// CIAN optionalRedeem — redeem with token choice
// ---------------------------------------------------------------------------

/// Encode `optionalRedeem(token, shares, cutPercentage, receiver, owner)` on
/// a CIAN Yield Layer vault.
///
/// This is CIAN's extended redeem function that allows the caller to specify
/// which ERC-20 token to receive when redeeming vault shares.  Burns
/// `shares` vault share tokens from `owner` and transfers the corresponding
/// value in `token` to `receiver`.  Returns the amount of assets received
/// after fees.
///
/// `token` is the address of the ERC-20 token to receive on redemption.
/// `shares` is the number of vault share tokens to burn, denominated in the
/// vault share token's decimals.
/// `cut_percentage` is the fee cut percentage (in basis points or as defined
/// by the vault's fee configuration); pass `0` for the standard exit fee.
/// `receiver` is the address that receives the output tokens.
/// `owner` is the address whose vault share tokens are burned; if not
/// `msg.sender`, the caller must have an allowance.
pub const fn make_fn_optional_redeem(
    token: Address,
    shares: &U,
    cut_percentage: &U,
    receiver: Address,
    owner: Address,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_OPTIONAL_REDEEM,
        leftpad_addr(token),
        shares.0,
        cut_percentage.0,
        leftpad_addr(receiver),
        leftpad_addr(owner)
    )
}

// ---------------------------------------------------------------------------
// CIAN requestRedeem — async (queued) redemption request
// ---------------------------------------------------------------------------

/// Encode `requestRedeem(shares, token)` on a CIAN Yield Layer vault.
///
/// Submits an asynchronous redemption request: burns `shares` vault share
/// tokens from `msg.sender` and queues the redemption to be settled later
/// by the vault operator.  The redeemed assets are sent to `msg.sender`
/// when the request is executed.  This is used for vaults that do not
/// support instant redemption.
///
/// `shares` is the number of vault share tokens to redeem, denominated in
/// the vault share token's decimals.
/// `token` is the address of the ERC-20 token to receive on settlement.
pub const fn make_fn_request_redeem(
    shares: &U,
    token: Address,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_REQUEST_REDEEM, shares.0, leftpad_addr(token))
}

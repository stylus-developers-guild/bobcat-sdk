//! Core end-user calldata for Revert Lend on Arbitrum.
//!
//! Revert Lend is a lending protocol that uses Uniswap V3 concentrated-liquidity
//! LP positions as collateral. The `V3Vault` contract implements ERC4626 for the
//! lending side (deposit/withdraw asset tokens) and manages collateral positions
//! via Uniswap V3 NFTs. This module exposes the four core end-user flows:
//!
//! 1. **Supply collateral** — transfer a Uniswap V3 LP NFT into the vault via
//!    `create` or `createWithPermit`.
//! 2. **Borrow** — borrow the vault's asset token against a collateralized
//!    position via `borrow`.
//! 3. **Repay** — repay borrowed assets (or debt shares) via `repay`.
//! 4. **Withdraw collateral** — withdraw a collateralized position from the
//!    vault via `remove`.
//!
//! All admin, liquidation, transformer, and reserve-management functions are
//! permissioned and intentionally excluded.
//!
//! ABI verified against the source at
//! <https://github.com/revert-finance/lend/blob/main/src/V3Vault.sol>
//! and the `IVault` interface at `src/interfaces/IVault.sol`.
//!
//! Sources:
//! - <https://github.com/revert-finance/lend>
//! - <https://revert.finance>

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_bool, leftpad_u8};
use bobcat_maths::U;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_CREATE = b"create(uint256,address)",
    SEL_CREATE_WITH_PERMIT = b"createWithPermit(uint256,address,uint256,uint8,bytes32,bytes32)",
    SEL_BORROW = b"borrow(uint256,uint256)",
    SEL_REPAY = b"repay(uint256,uint256,bool)",
    SEL_REMOVE = b"remove(uint256,address,bytes)",
}

// ---------------------------------------------------------------------------
// create — supply a Uniswap V3 LP position as collateral
// ---------------------------------------------------------------------------

/// Calldata for `create(uint256,address)` — transfers a Uniswap V3 LP NFT
/// into the vault as collateral. The caller must have approved the vault
/// for the NFT (via `safeTransferFrom`).
///
/// The `recipient` is the address that will own the collateralized loan
/// position inside the vault.
pub const fn make_fn_create(token_id: &U, recipient: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_CREATE, token_id.0, leftpad_addr(recipient))
}

// ---------------------------------------------------------------------------
// createWithPermit — supply collateral with NFT permit
// ---------------------------------------------------------------------------

/// Calldata for `createWithPermit(uint256,address,uint256,uint8,bytes32,bytes32)`
/// — transfers a Uniswap V3 LP NFT into the vault as collateral using an
/// EIP-712 permit signature so the vault can pull the NFT on the caller's
/// behalf without a separate approval transaction.
///
/// `deadline` is the unix timestamp until which the permit is valid.
/// `v`, `r`, `s` are the ECDSA signature components from the NFT owner
/// authorising the vault to transfer `token_id`.
pub const fn make_fn_create_with_permit(
    token_id: &U,
    recipient: Address,
    deadline: &U,
    v: u8,
    r: [u8; 32],
    s: [u8; 32],
) -> [u8; 4 + 32 * 6] {
    concat_arrays!(
        SEL_CREATE_WITH_PERMIT,
        token_id.0,
        leftpad_addr(recipient),
        deadline.0,
        leftpad_u8(v),
        r,
        s
    )
}

// ---------------------------------------------------------------------------
// borrow — borrow assets against a collateralized position
// ---------------------------------------------------------------------------

/// Calldata for `borrow(uint256,uint256)` — borrows `assets` amount of the
/// vault's underlying token against the collateralized position `token_id`.
///
/// The caller must be the owner of the loan position. The vault checks that
/// the position remains healthy after borrowing (collateral value >= debt
/// with a safety buffer).
pub const fn make_fn_borrow(token_id: &U, assets: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_BORROW, token_id.0, assets.0)
}

// ---------------------------------------------------------------------------
// repay — repay borrowed assets or debt shares
// ---------------------------------------------------------------------------

/// Calldata for `repay(uint256,uint256,bool)` — repays debt on a
/// collateralized position.
///
/// When `is_share` is `false`, `amount` is in asset tokens (e.g. USDC).
/// When `is_share` is `true`, `amount` is in debt shares. The caller must
/// have approved the vault to spend the underlying asset for the repay
/// amount.
pub const fn make_fn_repay(token_id: &U, amount: &U, is_share: bool) -> [u8; 4 + 32 * 3] {
    concat_arrays!(SEL_REPAY, token_id.0, amount.0, leftpad_bool(is_share))
}

// ---------------------------------------------------------------------------
// remove — withdraw a collateralized position
// ---------------------------------------------------------------------------

/// Calldata length for `remove` with empty `data` (the common case).
///
/// Layout: selector(4) + tokenId(32) + recipient(32) + offset(32) + length(32)
/// = 4 + 4*32 = 132.
pub const REMOVE_EMPTY_CALLDATA_LEN: usize = 4 + 32 * 4;

/// Calldata for `remove(uint256,address,bytes)` with empty `data` — withdraws
/// a collateralized Uniswap V3 LP NFT from the vault. The loan must be fully
/// repaid (debt shares == 0) before calling this.
///
/// `data` is passed through to `safeTransferFrom` on the Uniswap V3 NFT
/// manager; empty is the standard case for a plain withdrawal.
pub fn make_fn_remove(token_id: &U, recipient: Address) -> [u8; REMOVE_EMPTY_CALLDATA_LEN] {
    let mut out = [0u8; REMOVE_EMPTY_CALLDATA_LEN];
    out[..4].copy_from_slice(&SEL_REMOVE);
    // tokenId
    out[4..36].copy_from_slice(&token_id.0);
    // recipient
    out[36..56].copy_from_slice(&recipient);
    // offset to dynamic bytes data: 3 words from after selector = 96
    out[4 + 32 * 3 - 1] = 96;
    // length of bytes data: 0
    // (remaining bytes are already zero)
    out
}

/// Required output length for [`make_fn_remove_with_data`].
///
/// `remove(uint256,address,bytes)`:
/// selector(4) + tokenId(32) + recipient(32) + offset(32) + length(32) +
/// ceil(data_len / 32) * 32.
pub const fn remove_with_data_calldata_len(data_len: usize) -> usize {
    let padded = (data_len + 31) / 32 * 32;
    4 + 32 * 4 + padded
}

/// Encode `remove(uint256,address,bytes)` with arbitrary `data` into
/// `output`. Returns the number of bytes written, or 0 if the buffer is
/// too small.
///
/// `data` is forwarded to `safeTransferFrom` on the Uniswap V3 NFT manager.
/// The caller must supply a buffer of at least
/// [`remove_with_data_calldata_len`]`(data.len())` bytes.
pub fn make_fn_remove_with_data(
    output: &mut [u8],
    token_id: &U,
    recipient: Address,
    data: &[u8],
) -> usize {
    let required = remove_with_data_calldata_len(data.len());
    if output.len() < required {
        return 0;
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_REMOVE);
    // tokenId
    out[4..36].copy_from_slice(&token_id.0);
    // recipient
    out[36..56].copy_from_slice(&recipient);
    // offset to dynamic bytes: 3 words = 96
    out[4 + 32 * 3 - 1] = 96;
    // bytes length
    let len_word = (data.len() as u64).to_be_bytes();
    out[4 + 32 * 3 + 24..4 + 32 * 4].copy_from_slice(&len_word);
    // bytes data (zero-padded to 32-byte boundary by the fill above)
    out[4 + 32 * 4..4 + 32 * 4 + data.len()].copy_from_slice(data);
    required
}

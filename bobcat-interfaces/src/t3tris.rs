//! T3tris Finance vault calldata builders for depositor flows.
//!
//! T3tris vaults may settle synchronously or through request-based epochs. Synchronous deposits
//! use T3tris' allowance-based `deposit(address,uint256,bytes)` overload and redemptions use the
//! ERC-4626 `redeem(uint256,address,address)` call. Asynchronous flows request a deposit or
//! redemption first, then claim after the curator settles the epoch.
//!
//! These builders deliberately select T3tris' safe paths (`unsafe = false`) and omit Permit2 data,
//! so callers must approve the vault to spend the underlying asset before depositing. Curator,
//! settlement, whitelist, fee, lifecycle, and other protocol-operator functions are omitted.
//!
//! Flow references:
//! - <https://docs.t3tris.finance/depositors/deposit>
//! - <https://docs.t3tris.finance/depositors/redeem>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit(address,uint256,bytes)",
    SEL_REDEEM = b"redeem(uint256,address,address)",
    SEL_REQUEST_DEPOSIT = b"requestDeposit(address,bool,uint256,bytes)",
    SEL_CLAIM_DEPOSIT = b"claimDeposit(address)",
    SEL_REQUEST_REDEEM = b"requestRedeem(address,address,address,bool,uint256)",
    SEL_CLAIM_REDEEM = b"claimRedeem(address,address,bool)",
}

const FALSE: [u8; 32] = [0; 32];
const EMPTY_BYTES_LENGTH: [u8; 32] = [0; 32];
const DEPOSIT_BYTES_OFFSET: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96,
];
const REQUEST_DEPOSIT_BYTES_OFFSET: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128,
];

/// Encode a synchronous T3tris deposit using an existing ERC-20 allowance.
///
/// `receiver` receives the newly minted vault shares. Send this calldata to the strategy vault
/// after approving it to transfer at least `assets` of the vault's underlying token.
pub const fn make_fn_deposit(receiver: Address, assets: &U) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT,
        leftpad_addr(receiver),
        assets.0,
        DEPOSIT_BYTES_OFFSET,
        EMPTY_BYTES_LENGTH
    )
}

/// Encode synchronous ERC-4626 `redeem(shares, receiver, owner)`.
///
/// `owner` supplies the vault shares and `receiver` receives the underlying assets. If the caller
/// is not `owner`, the caller must have sufficient share allowance.
pub const fn make_fn_redeem(shares: &U, receiver: Address, owner: Address) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_REDEEM,
        shares.0,
        leftpad_addr(receiver),
        leftpad_addr(owner)
    )
}

/// Encode an asynchronous deposit request using an existing ERC-20 allowance.
///
/// This selects T3tris' safe transfer path and supplies no Permit2 payload. After the epoch is
/// settled, call [`make_fn_claim_deposit`] to mint the claimable shares to `receiver`.
pub const fn make_fn_request_deposit(receiver: Address, assets: &U) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_REQUEST_DEPOSIT,
        leftpad_addr(receiver),
        FALSE,
        assets.0,
        REQUEST_DEPOSIT_BYTES_OFFSET,
        EMPTY_BYTES_LENGTH
    )
}

/// Encode `claimDeposit(receiver)` after an asynchronous deposit has settled.
pub const fn make_fn_claim_deposit(receiver: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_CLAIM_DEPOSIT, leftpad_addr(receiver))
}

/// Encode an asynchronous redemption request.
///
/// `owner` supplies the vault shares, `receiver` receives assets after settlement, and
/// `previous_claim_receiver` identifies where any already-claimable redemption should be sent
/// before this new request replaces it. This builder selects T3tris' safe transfer path.
pub const fn make_fn_request_redeem(
    receiver: Address,
    owner: Address,
    previous_claim_receiver: Address,
    shares: &U,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_REQUEST_REDEEM,
        leftpad_addr(receiver),
        leftpad_addr(owner),
        leftpad_addr(previous_claim_receiver),
        FALSE,
        shares.0
    )
}

/// Encode `claimRedeem(owner, receiver, false)` after an asynchronous redemption has settled.
///
/// `receiver` receives the underlying assets. This builder selects T3tris' safe transfer path.
pub const fn make_fn_claim_redeem(owner: Address, receiver: Address) -> [u8; 4 + 32 * 3] {
    concat_arrays!(
        SEL_CLAIM_REDEEM,
        leftpad_addr(owner),
        leftpad_addr(receiver),
        FALSE
    )
}

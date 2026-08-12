//! Core Derive V2 collateral and trade-action builders.
//!
//! Collateral deposits are direct on-chain calls to a supported Derive asset
//! contract. The caller must first approve that asset contract to spend the
//! underlying ERC-20.
//!
//! Orders are not submitted on-chain by end users. Users sign the EIP-712
//! digest produced by [`trade_action_signing_hash`] and send the action to
//! Derive's `private/order` API. A permissioned trade executor later submits
//! matched signed actions on-chain. Order cancellation is likewise an
//! authenticated off-chain API operation (`private/cancel` or
//! `private/cancel_by_nonce`), so this module exposes no submit or cancel
//! calldata builder.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::{I, U};
use bobcat_storage::{const_keccak256, keccak256};

use crate::selectors;

pub type Address = [u8; 20];

pub const ACTION_TYPE_HASH: U = const_keccak256(
    b"Action(uint256 subaccountId,uint256 nonce,address module,bytes data,uint256 expiry,address owner,address signer)",
);
pub const EIP712_DOMAIN_TYPE_HASH: U = const_keccak256(
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
);
pub const EIP712_NAME_HASH: U = const_keccak256(b"Matching");
pub const EIP712_VERSION_HASH: U = const_keccak256(b"1.0");

selectors! {
    SEL_DEPOSIT = b"deposit(uint256,uint256)",
}

/// The fixed-width ABI data signed for a Derive TradeModule order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeData {
    pub asset: Address,
    pub sub_id: U,
    pub limit_price: I,
    pub desired_amount: I,
    pub worst_fee: U,
    pub recipient_id: U,
    pub is_bid: bool,
}

/// Fields shared by all Derive signed actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Action<'a> {
    pub subaccount_id: U,
    pub nonce: U,
    pub module: Address,
    pub data: &'a [u8],
    pub expiry: U,
    pub owner: Address,
    pub signer: Address,
}

/// Encode a direct collateral deposit into an existing Derive subaccount.
///
/// Send this calldata to the Derive asset contract that wraps the collateral,
/// after approving it to spend `asset_amount` of the underlying token.
pub const fn make_fn_deposit(recipient_account: &U, asset_amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT, recipient_account.0, asset_amount.0)
}

/// ABI-encode the TradeModule data included in a signed order action.
pub const fn make_trade_data(data: &TradeData) -> [u8; 32 * 7] {
    concat_arrays!(
        leftpad_addr(data.asset),
        data.sub_id.0,
        data.limit_price.0,
        data.desired_amount.0,
        data.worst_fee.0,
        data.recipient_id.0,
        leftpad_bool(data.is_bid)
    )
}

/// Compute the EIP-712 domain separator for a Derive Matching deployment.
pub fn matching_domain_separator(chain_id: &U, matching: Address) -> U {
    let encoded: [u8; 32 * 5] = concat_arrays!(
        EIP712_DOMAIN_TYPE_HASH.0,
        EIP712_NAME_HASH.0,
        EIP712_VERSION_HASH.0,
        chain_id.0,
        leftpad_addr(matching)
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 struct hash for a Derive action.
pub fn action_struct_hash(action: &Action<'_>) -> U {
    let encoded: [u8; 32 * 8] = concat_arrays!(
        ACTION_TYPE_HASH.0,
        action.subaccount_id.0,
        action.nonce.0,
        leftpad_addr(action.module),
        keccak256(action.data).0,
        action.expiry.0,
        leftpad_addr(action.owner),
        leftpad_addr(action.signer)
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 digest for any Derive signed action.
pub fn action_signing_hash(domain_separator: &U, action: &Action<'_>) -> U {
    let action_hash = action_struct_hash(action);
    let preimage: [u8; 2 + 32 * 2] =
        concat_arrays!([0x19, 0x01], domain_separator.0, action_hash.0);
    keccak256(&preimage)
}

/// Encode `trade` and compute the digest signed for `private/order`.
pub fn trade_action_signing_hash(
    domain_separator: &U,
    subaccount_id: U,
    nonce: U,
    trade_module: Address,
    trade: &TradeData,
    expiry: U,
    owner: Address,
    signer: Address,
) -> U {
    let data = make_trade_data(trade);
    action_signing_hash(
        domain_separator,
        &Action {
            subaccount_id,
            nonce,
            module: trade_module,
            data: &data,
            expiry,
            owner,
            signer,
        },
    )
}

const fn leftpad_bool(value: bool) -> [u8; 32] {
    let mut out = [0; 32];
    out[31] = value as u8;
    out
}

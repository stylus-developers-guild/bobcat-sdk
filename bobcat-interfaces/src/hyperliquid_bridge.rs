//! Core end-user helpers for Hyperliquid's USDC bridge flow.
//!
//! Deposits are USDC transfers to the bridge address. The bridge credits the
//! token sender, so an end user's EOA must send the transfer directly. A Stylus
//! contract executing the transfer would credit the contract and could strand
//! the funds because that contract cannot produce Hyperliquid's ECDSA withdrawal
//! signature.
//!
//! Withdrawals are initiated off-chain: sign the EIP-712 digest produced by
//! [`withdraw_signing_hash`] and submit the corresponding `withdraw3` action to
//! Hyperliquid's exchange API. No permissioned bridge calls are exposed here.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;
use bobcat_storage::{const_keccak256, keccak256};

use crate::selectors;

pub type Address = [u8; 20];

pub const WITHDRAW_TYPE_HASH: U = const_keccak256(
    b"HyperliquidTransaction:Withdraw(string hyperliquidChain,string destination,string amount,uint64 time)",
);
pub const EIP712_DOMAIN_TYPE_HASH: U = const_keccak256(
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
);
pub const EIP712_NAME_HASH: U = const_keccak256(b"HyperliquidSignTransaction");
pub const EIP712_VERSION_HASH: U = const_keccak256(b"1");

selectors! {
    SEL_TRANSFER = b"transfer(address,uint256)",
}

/// Encode `USDC.transfer(bridge, amount)` for a direct transaction from the EOA
/// that should receive the Hyperliquid credit.
pub const fn make_fn_deposit(bridge: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_TRANSFER, leftpad_addr(bridge), amount.0)
}

/// Compute the EIP-712 struct hash for a Hyperliquid `withdraw3` action.
///
/// `hyperliquid_chain`, `destination`, and `amount` must be the exact bytes used
/// in the submitted JSON action. `time` is the millisecond request nonce.
pub fn withdraw_struct_hash(
    hyperliquid_chain: &[u8],
    destination: &[u8],
    amount: &[u8],
    time: u64,
) -> U {
    let encoded: [u8; 32 * 5] = concat_arrays!(
        WITHDRAW_TYPE_HASH.0,
        keccak256(hyperliquid_chain).0,
        keccak256(destination).0,
        keccak256(amount).0,
        leftpad_u64(time)
    );
    keccak256(&encoded)
}

/// Compute Hyperliquid's EIP-712 domain separator for `signature_chain_id`.
pub fn withdraw_domain_separator(signature_chain_id: u64) -> U {
    let encoded: [u8; 32 * 5] = concat_arrays!(
        EIP712_DOMAIN_TYPE_HASH.0,
        EIP712_NAME_HASH.0,
        EIP712_VERSION_HASH.0,
        leftpad_u64(signature_chain_id),
        [0u8; 32]
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 digest an EOA signs to initiate `withdraw3`.
pub fn withdraw_signing_hash(
    signature_chain_id: u64,
    hyperliquid_chain: &[u8],
    destination: &[u8],
    amount: &[u8],
    time: u64,
) -> U {
    let domain = withdraw_domain_separator(signature_chain_id);
    let action = withdraw_struct_hash(hyperliquid_chain, destination, amount, time);
    let preimage: [u8; 2 + 32 * 2] = concat_arrays!([0x19, 0x01], domain.0, action.0);
    keccak256(&preimage)
}

const fn leftpad_u64(value: u64) -> [u8; 32] {
    concat_arrays!([0u8; 24], value.to_be_bytes())
}

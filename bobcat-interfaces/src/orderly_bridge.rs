//! Core end-user calldata builders and EIP-712 helpers for Orderly Bridge
//! flows on Arbitrum.
//!
//! Orderly is an omnichain perpetuals DEX. On each supported chain (including
//! Arbitrum One), a `Vault` contract custodies user deposits. Users deposit
//! ERC-20 tokens by calling `Vault.deposit` (or `Vault.depositTo` to credit a
//! different receiver); the Vault forwards the deposit cross-chain to the
//! Orderly Ledger via LayerZero.
//!
//! Withdrawals are initiated off-chain: the user signs an EIP-712 `Withdraw`
//! message targeting the Orderly Ledger contract and submits it to the Orderly
//! REST API. The Vault's on-chain `withdraw` function is `onlyCrossChainManager`
//! and is therefore intentionally not exposed here.
//!
//! This module exposes:
//!
//! - `make_fn_deposit` — calldata for `Vault.deposit((bytes32,bytes32,bytes32,uint128))`.
//! - `make_fn_deposit_to` — calldata for `Vault.depositTo(address,(bytes32,bytes32,bytes32,uint128))`.
//! - `make_fn_get_deposit_fee` — calldata for `Vault.getDepositFee(address,(bytes32,bytes32,bytes32,uint128))`.
//! - `withdraw_struct_hash`, `withdraw_domain_separator`, `withdraw_signing_hash` —
//!   EIP-712 helpers for the off-chain withdrawal flow.
//!
//! All admin, pause, rebalance, token-management, and cross-chain operator
//! functions are permissioned and intentionally excluded.
//!
//! # ABI
//!
//! Verified against the official Orderly `contract-evm-abi` repository
//! (`abi/latest/Vault.json`) and the deployed Vault contract on Arbitrum One.
//!
//! Sources:
//! - <https://github.com/OrderlyNetwork/contract-evm-abi>
//! - <https://orderly.network/docs/build-on-omnichain/user-flows/withdrawal-deposit>
//! - <https://orderly.network/docs/build-on-omnichain/addresses>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;
use bobcat_storage::{const_keccak256, keccak256};

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT = b"deposit((bytes32,bytes32,bytes32,uint128))",
    SEL_DEPOSIT_TO = b"depositTo(address,(bytes32,bytes32,bytes32,uint128))",
    SEL_GET_DEPOSIT_FEE = b"getDepositFee(address,(bytes32,bytes32,bytes32,uint128))",
}

// ---------------------------------------------------------------------------
// EIP-712 type hashes (compile-time constants)
// ---------------------------------------------------------------------------

/// `keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")`.
pub const EIP712_DOMAIN_TYPE_HASH: U = const_keccak256(
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
);

/// `keccak256("Orderly")`.
const EIP712_NAME_HASH: U = const_keccak256(b"Orderly");

/// `keccak256("1")`.
const EIP712_VERSION_HASH: U = const_keccak256(b"1");

/// `keccak256("Withdraw(string brokerId,uint256 chainId,address receiver,string token,uint256 amount,uint64 withdrawNonce,uint64 timestamp)")`.
pub const WITHDRAW_TYPE_HASH: U = const_keccak256(
    b"Withdraw(string brokerId,uint256 chainId,address receiver,string token,uint256 amount,uint64 withdrawNonce,uint64 timestamp)",
);

// ---------------------------------------------------------------------------
// Local padding helpers
// ---------------------------------------------------------------------------

const fn leftpad_u64(value: u64) -> [u8; 32] {
    concat_arrays!([0u8; 24], value.to_be_bytes())
}

const fn leftpad_u128(value: u128) -> [u8; 32] {
    concat_arrays!([0u8; 16], value.to_be_bytes())
}

// ---------------------------------------------------------------------------
// Deposit calldata
// ---------------------------------------------------------------------------

/// The on-chain deposit payload (`VaultDepositFE`).
///
/// All fields are static ABI types, so the tuple is encoded inline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VaultDepositFe {
    /// The user's Orderly account ID.
    pub account_id: [u8; 32],
    /// `keccak256` of the broker ID string (e.g. `keccak256("woofi_dex")`).
    pub broker_hash: [u8; 32],
    /// `keccak256` of the token string (e.g. `keccak256("USDC")`).
    pub token_hash: [u8; 32],
    /// Amount of tokens to deposit, in the token's smallest unit.
    pub token_amount: u128,
}

/// Encode `deposit((bytes32,bytes32,bytes32,uint128))`.
///
/// This is the core deposit flow: the caller locks tokens in the Vault on
/// Arbitrum to credit their Orderly account. The caller must have approved the
/// Vault to spend `token_amount` of the deposit token before sending this
/// calldata. If the token is native ETH, the caller must also send the
/// corresponding `msg.value`.
pub const fn make_fn_deposit(data: &VaultDepositFe) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT,
        data.account_id,
        data.broker_hash,
        data.token_hash,
        leftpad_u128(data.token_amount)
    )
}

/// Encode `depositTo(address,(bytes32,bytes32,bytes32,uint128))`.
///
/// Like [`make_fn_deposit`] but credits the deposit to `receiver` instead of
/// `msg.sender`.
pub const fn make_fn_deposit_to(
    receiver: Address,
    data: &VaultDepositFe,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_DEPOSIT_TO,
        leftpad_addr(receiver),
        data.account_id,
        data.broker_hash,
        data.token_hash,
        leftpad_u128(data.token_amount)
    )
}

/// Encode `getDepositFee(address,(bytes32,bytes32,bytes32,uint128))`.
///
/// View function that returns the LayerZero cross-chain fee (in wei) for a
/// prospective deposit. Call this before [`make_fn_deposit`] to determine the
/// `msg.value` needed when `depositFeeEnabled` is true.
pub const fn make_fn_get_deposit_fee(
    receiver: Address,
    data: &VaultDepositFe,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_GET_DEPOSIT_FEE,
        leftpad_addr(receiver),
        data.account_id,
        data.broker_hash,
        data.token_hash,
        leftpad_u128(data.token_amount)
    )
}

// ---------------------------------------------------------------------------
// Withdrawal EIP-712 helpers
// ---------------------------------------------------------------------------

/// Compute the EIP-712 domain separator for the Orderly Ledger contract.
///
/// The domain is `{ name: "Orderly", version: "1", chainId, verifyingContract }`
/// where `verifying_contract` is the Ledger contract address on the target
/// chain.
pub fn withdraw_domain_separator(chain_id: u64, verifying_contract: Address) -> U {
    let encoded: [u8; 32 * 5] = concat_arrays!(
        EIP712_DOMAIN_TYPE_HASH.0,
        EIP712_NAME_HASH.0,
        EIP712_VERSION_HASH.0,
        leftpad_u64(chain_id),
        leftpad_addr(verifying_contract)
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 struct hash for a `Withdraw` message.
///
/// `broker_id` and `token` are the UTF-8 bytes of the broker ID and token
/// symbol strings (e.g. `b"woofi_dex"` and `b"USDC"`). `amount` is the
/// withdrawal amount in the token's smallest unit.
pub fn withdraw_struct_hash(
    broker_id: &[u8],
    chain_id: u64,
    receiver: Address,
    token: &[u8],
    amount: &U,
    withdraw_nonce: u64,
    timestamp: u64,
) -> U {
    let encoded: [u8; 32 * 8] = concat_arrays!(
        WITHDRAW_TYPE_HASH.0,
        keccak256(broker_id).0,
        leftpad_u64(chain_id),
        leftpad_addr(receiver),
        keccak256(token).0,
        amount.0,
        leftpad_u64(withdraw_nonce),
        leftpad_u64(timestamp)
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 signing digest for an Orderly withdrawal request.
///
/// The caller signs this digest with their EOA key and submits the signature
/// to the Orderly REST API (`POST /v1/withdraw_request`) to initiate the
/// withdrawal. `verifying_contract` is the Ledger contract address for the
/// target chain.
pub fn withdraw_signing_hash(
    chain_id: u64,
    verifying_contract: Address,
    broker_id: &[u8],
    receiver: Address,
    token: &[u8],
    amount: &U,
    withdraw_nonce: u64,
    timestamp: u64,
) -> U {
    let domain = withdraw_domain_separator(chain_id, verifying_contract);
    let struct_hash = withdraw_struct_hash(
        broker_id,
        chain_id,
        receiver,
        token,
        amount,
        withdraw_nonce,
        timestamp,
    );
    let preimage: [u8; 2 + 32 * 2] = concat_arrays!([0x19, 0x01], domain.0, struct_hash.0);
    keccak256(&preimage)
}

//! Calldata builders and EIP-712 signing helpers for Aevo collateral flows.
//!
//! Aevo is a decentralized derivatives exchange on a custom OP Stack Layer 2.
//! Options and perpetual futures are settled in USDC, with off-chain order
//! matching and on-chain settlement.
//!
//! This module covers the two core end-user collateral operations:
//!
//! - **Deposit** — bridge USDC from an L1 (Ethereum mainnet or an L2 like
//!   Arbitrum) into the Aevo L2 via a standard OP Stack `L1StandardBridge`.
//!   The caller first approves the bridge to spend USDC, then calls
//!   `depositERC20`. Both `depositERC20` (deposit to caller) and
//!   `depositERC20To` (deposit to a specified recipient) are supported.
//!
//! - **Withdraw** — sign an EIP-712 `Withdraw` struct and submit it to
//!   Aevo's REST API (`POST /withdraw`). The signed withdrawal is relayed
//!   on-chain by the Aevo protocol. No direct on-chain calldata is needed
//!   for withdrawals.
//!
//! All builders are `no_std` and allocation-free.
//!
//! ABI reference (OP Stack StandardBridge):
//! <https://github.com/ethereum-optimism/optimism/blob/develop/packages/contracts-bedrock/src/universal/StandardBridge.sol>
//!
//! EIP-712 Withdraw struct (from official Aevo SDK):
//! `Withdraw(address collateral, address to, uint256 amount, uint256 salt, bytes32 data)`
//!
//! EIP-712 domain (mainnet):
//! name = "Aevo Mainnet", version = "1", chainId = 1

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_u32};
use bobcat_maths::U;
use bobcat_storage::{const_keccak256, keccak256};

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

// ---------------------------------------------------------------------------
// Selectors
// ---------------------------------------------------------------------------

selectors! {
    SEL_DEPOSIT_ERC20 = b"depositERC20(address,address,uint256,uint32,bytes)",
    SEL_DEPOSIT_ERC20_TO = b"depositERC20To(address,address,address,uint256,uint32,bytes)",
}

// ---------------------------------------------------------------------------
// EIP-712 constants
// ---------------------------------------------------------------------------

/// EIP-712 type hash for Aevo's `Withdraw` struct.
pub const WITHDRAW_TYPE_HASH: U = const_keccak256(
    b"Withdraw(address collateral,address to,uint256 amount,uint256 salt,bytes32 data)",
);

/// EIP-712 domain type hash.
pub const EIP712_DOMAIN_TYPE_HASH: U = const_keccak256(
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
);

/// EIP-712 domain name hash for Aevo mainnet.
pub const EIP712_MAINNET_NAME_HASH: U = const_keccak256(b"Aevo Mainnet");

/// EIP-712 domain name hash for Aevo testnet.
pub const EIP712_TESTNET_NAME_HASH: U = const_keccak256(b"Aevo Testnet");

/// EIP-712 domain version hash.
pub const EIP712_VERSION_HASH: U = const_keccak256(b"1");

/// EIP-712 domain chain ID for Aevo mainnet (Ethereum L1 chain ID).
pub const EIP712_MAINNET_CHAIN_ID: U = U(concat_arrays!([0u8; 31], [1u8]));

/// EIP-712 domain chain ID for Aevo testnet (Sepolia chain ID).
pub const EIP712_TESTNET_CHAIN_ID: U = U([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0xAA, 0x36, 0xA7,
]);

// ---------------------------------------------------------------------------
// depositERC20 — bridge USDC to Aevo L2 (deposit to caller)
// ---------------------------------------------------------------------------

/// Encode a `depositERC20` call with empty `extraData`.
///
/// Sends this calldata to the L1StandardBridge after approving it to spend
/// `amount` of `l1_token`. The bridge transfers `l1_token` on the source
/// chain and mints the corresponding `l2_token` on Aevo L2 to the caller.
///
/// The `extraData` parameter is empty (zero-length `bytes`), matching the
/// common-case deposit flow used by the official Aevo SDK.
pub const fn make_fn_deposit_erc20(
    l1_token: Address,
    l2_token: Address,
    amount: &U,
    min_gas_limit: u32,
) -> [u8; 4 + 32 * 6] {
    // ABI: depositERC20(address,address,uint256,uint32,bytes)
    // Head: l1Token, l2Token, amount, minGasLimit, offset_to_extraData
    // Tail: length(0) for empty bytes
    // offset = 5 head words * 32 = 160 = 0xa0
    concat_arrays!(
        SEL_DEPOSIT_ERC20,
        leftpad_addr(l1_token),
        leftpad_addr(l2_token),
        amount.0,
        leftpad_u32(min_gas_limit),
        leftpad_u32(0xa0), // offset to bytes extraData
        [0u8; 32]        // length of extraData = 0
    )
}

// ---------------------------------------------------------------------------
// depositERC20To — bridge USDC to Aevo L2 for a specified recipient
// ---------------------------------------------------------------------------

/// Encode a `depositERC20To` call with empty `extraData`.
///
/// Same as [`make_fn_deposit_erc20`] but deposits to `recipient` instead of
/// the caller.
pub const fn make_fn_deposit_erc20_to(
    l1_token: Address,
    l2_token: Address,
    recipient: Address,
    amount: &U,
    min_gas_limit: u32,
) -> [u8; 4 + 32 * 7] {
    // ABI: depositERC20To(address,address,address,uint256,uint32,bytes)
    // Head: l1Token, l2Token, to, amount, minGasLimit, offset_to_extraData
    // Tail: length(0) for empty bytes
    // offset = 6 head words * 32 = 192 = 0xc0
    concat_arrays!(
        SEL_DEPOSIT_ERC20_TO,
        leftpad_addr(l1_token),
        leftpad_addr(l2_token),
        leftpad_addr(recipient),
        amount.0,
        leftpad_u32(min_gas_limit),
        leftpad_u32(0xc0), // offset to bytes extraData
        [0u8; 32]        // length of extraData = 0
    )
}

// ---------------------------------------------------------------------------
// EIP-712 Withdraw signing
// ---------------------------------------------------------------------------

/// Compute the EIP-712 domain separator for an Aevo deployment.
///
/// `name_hash` should be [`EIP712_MAINNET_NAME_HASH`] or
/// [`EIP712_TESTNET_NAME_HASH`]. `chain_id` should be
/// [`EIP712_MAINNET_CHAIN_ID`] or [`EIP712_TESTNET_CHAIN_ID`].
/// `verifying_contract` is the Aevo exchange contract address.
pub fn domain_separator(
    name_hash: &U,
    chain_id: &U,
    verifying_contract: Address,
) -> U {
    let encoded: [u8; 32 * 5] = concat_arrays!(
        EIP712_DOMAIN_TYPE_HASH.0,
        name_hash.0,
        EIP712_VERSION_HASH.0,
        chain_id.0,
        leftpad_addr(verifying_contract)
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 struct hash for an Aevo `Withdraw` action.
///
/// `data` is the 32-byte field from the Withdraw struct, typically
/// `keccak256("")` (hash of empty bytes) when no extra data is needed.
pub fn withdraw_struct_hash(
    collateral: Address,
    to: Address,
    amount: &U,
    salt: &U,
    data: &U,
) -> U {
    let encoded: [u8; 32 * 6] = concat_arrays!(
        WITHDRAW_TYPE_HASH.0,
        leftpad_addr(collateral),
        leftpad_addr(to),
        amount.0,
        salt.0,
        data.0
    );
    keccak256(&encoded)
}

/// Compute the EIP-712 digest to sign for an Aevo withdrawal.
///
/// The resulting hash is signed by the account's private key and submitted
/// to Aevo's `POST /withdraw` REST endpoint along with the withdrawal
/// parameters and signature.
pub fn withdraw_signing_hash(
    domain_sep: &U,
    collateral: Address,
    to: Address,
    amount: &U,
    salt: &U,
    data: &U,
) -> U {
    let struct_hash = withdraw_struct_hash(collateral, to, amount, salt, data);
    let preimage: [u8; 2 + 32 * 2] =
        concat_arrays!([0x19, 0x01], domain_sep.0, struct_hash.0);
    keccak256(&preimage)
}

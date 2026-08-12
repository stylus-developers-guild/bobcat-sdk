//! Core end-user calldata builders for Railgun's `shield` and `transact`
//! entrypoints on Arbitrum.
//!
//! Railgun is a privacy protocol that shields ERC-20, ERC-721, and ERC-1155
//! tokens into zero-knowledge note commitments.  Shielded tokens can later be
//! privately transferred, unshielded, or used in adapted calls — all proven
//! valid via SNARK proofs processed on-chain by the `RailgunSmartWallet`
//! contract.
//!
//! ## End-user functions exposed
//!
//! - [`make_fn_shield`] — encode `shield(ShieldRequest[])`: deposit tokens
//!   into the privacy pool by creating note commitments.  The caller supplies
//!   one or more [`ShieldRequest`]s, each containing a commitment preimage
//!   (nullifier public key, token data, value) and an encrypted ciphertext
//!   bundle.
//! - [`make_fn_transact`] — encode `transact(Transaction[])`: submit a batch
//!   of zero-knowledge transactions (shielded transfers, unshields, or adapted
//!   calls).  Each [`Transaction`] carries a SNARK proof, Merkle root,
//!   nullifiers, commitments, bound parameters, and an optional unshield
//!   preimage.
//!
//! Both functions are `external payable` on `RailgunSmartWallet`.  Native ETH
//! is shielded by setting `token_type = ERC20` and `token_address = [0; 20]`
//! in the commitment preimage; the ETH value must be sent as `msg.value`.
//!
//! ## Prerequisites
//!
//! - **ZK proofs**: The caller must generate valid SNARK proofs off-chain
//!   (typically via the Railgun Engine library) before calling `transact`.
//!   The on-chain contract verifies proofs but does not generate them.
//! - **Token approval**: For ERC-20 / ERC-1155 shields, the caller must
//!   approve the `RailgunSmartWallet` contract to transfer the token before
//!   sending the `shield` calldata.  For ERC-721, the caller must be the NFT
//!   owner and have approved the contract for `safeTransferFrom`.
//! - **Shielded keys**: The caller must have generated Railgun spending keys
//!   and viewing keys off-chain.  The `npk` (note public key) is derived from
//!   the spending public key, nullifier key, and a random value via Poseidon.
//!
//! ## Privacy / safety constraints
//!
//! - The `shield` function **publicly reveals** the token address and shield
//!   amount on-chain (before encryption into the commitment).  Callers should
//!   be aware that shielding is not anonymous at the deposit stage — only
//!   subsequent shielded transfers are private.
//! - The `transact` function emits nullifiers (preventing double-spends) and
//!   new commitments, but the transaction details (recipient, amount, token)
//!   are encrypted in the `CommitmentCiphertext`.  Callers must ensure the
//!   `annotationData` and `memo` fields are properly encrypted before
//!   inclusion in calldata.
//! - The `adaptContract` and `adaptParams` fields in [`BoundParams`] enable
//!   "adapted" transactions that call an external contract as part of the
//!   shielded flow.  For standard shielded transfers and unshields, set
//!   `adapt_contract = [0; 20]` and `adapt_params = [0; 32]`.
//! - **Never** expose the spending private key, nullifier key, or viewing key
//!   in calldata.  These are used off-chain to generate proofs and decrypt
//!   notes.
//!
//! ## Permissioning
//!
//! `shield` and `transact` are end-user functions callable by any address.
//! Treasury management, fee configuration, token blocklist administration,
//! verifier upgrades, pause/unpause, and all other operator functions are
//! permissioned and intentionally **not** exposed here.
//!
//! ## ABI source
//!
//! Function signatures and struct layouts verified against the official
//! Railgun smart contract source:
//! - <https://github.com/Railgun-Privacy/contract> (main branch)
//! - `RailgunSmartWallet.sol` — inherits `RailgunLogic`, defines `shield` and
//!   `transact`
//! - `Globals.sol` — defines all struct and enum types
//! - Compiled ABI: `RailgunSmartWallet.json` (V2.1)
//!
//! ## no_std / no-alloc
//!
//! All types are stack-allocated with borrowed slices for dynamic data.  The
//! builder functions write into caller-provided `&mut [u8]` buffers — no
//! `Vec`, `String`, `Box`, or heap allocations are used anywhere in this
//! module.

use bobcat_cd::leftpad_addr;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall {
        /// Minimum buffer size required.
        required: usize,
    },
}

// ---------------------------------------------------------------------------
// Selectors
// ---------------------------------------------------------------------------

selectors! {
    SEL_SHIELD = b"shield(((bytes32,(uint8,address,uint256),uint120),(bytes32[3],bytes32))[])",
    SEL_TRANSACT = b"transact((((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),bytes32,bytes32[],bytes32[],(uint16,uint72,uint8,uint64,address,bytes32,(bytes32[4],bytes32,bytes32,bytes,bytes)[]),(bytes32,(uint8,address,uint256),uint120))[])",
}

// ---------------------------------------------------------------------------
// Enum types
// ---------------------------------------------------------------------------

/// Railgun token type, mirroring the Solidity `TokenType` enum.
///
/// - `Erc20` (0): fungible tokens (including native ETH via `address(0)`)
/// - `Erc721` (1): non-fungible tokens
/// - `Erc1155` (2): multi-token standard
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TokenType {
    /// ERC-20 fungible token (native ETH uses `address(0)`).
    Erc20 = 0,
    /// ERC-721 non-fungible token.
    Erc721 = 1,
    /// ERC-1155 multi-token.
    Erc1155 = 2,
}

/// Railgun unshield type, mirroring the Solidity `UnshieldType` enum.
///
/// - `None` (0): no unshield (pure shielded transfer)
/// - `Normal` (1): standard unshield to the caller
/// - `Redirect` (2): unshield with redirect to a different recipient
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UnshieldType {
    /// No unshield — the transaction is a pure shielded transfer.
    None = 0,
    /// Standard unshield — tokens leave the privacy pool to the caller.
    Normal = 1,
    /// Redirected unshield — tokens leave to a specified external address.
    Redirect = 2,
}

// ---------------------------------------------------------------------------
// Shield types
// ---------------------------------------------------------------------------

/// Token data for a Railgun note commitment.
///
/// Mirrors `struct TokenData` in `Globals.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenData {
    /// Token type (ERC-20, ERC-721, or ERC-1155).
    pub token_type: TokenType,
    /// Token contract address.  Use `[0u8; 20]` for native ETH (ERC-20 only).
    pub token_address: Address,
    /// Token sub-ID: for ERC-721 this is the NFT token ID; for ERC-20 and
    /// ERC-1155 use `0`.
    pub token_sub_id: [u8; 32],
}

/// Commitment preimage — the data committed to in a note.
///
/// Mirrors `struct CommitmentPreimage` in `Globals.sol`.  This struct is
/// reused in both `ShieldRequest` (for new notes) and `Transaction` (for the
/// unshield preimage).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitmentPreimage {
    /// Note public key: `Poseidon(Poseidon(spending_pub_key, nullifier_key),
    /// random)`.
    pub npk: [u8; 32],
    /// Token data identifying the asset.
    pub token: TokenData,
    /// Note value in the token's smallest unit.  Must fit in `uint120`
    /// (i.e. `< 2^120`).
    pub value: u128,
}

/// Encrypted ciphertext for a shield commitment.
///
/// Mirrors `struct ShieldCiphertext` in `Globals.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldCiphertext {
    /// Encrypted bundle: `[IV_shared, tag, random, IV_sender, receiver_vpk]`
    /// packed across three `bytes32` words.
    pub encrypted_bundle: [[u8; 32]; 3],
    /// Shield key: public key used to derive the shared decryption key.
    pub shield_key: [u8; 32],
}

/// A single shield request.
///
/// Mirrors `struct ShieldRequest` in `Globals.sol`.  Each request creates
/// one note commitment in the Merkle tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldRequest {
    /// The commitment preimage.
    pub preimage: CommitmentPreimage,
    /// The encrypted ciphertext.
    pub ciphertext: ShieldCiphertext,
}

// ---------------------------------------------------------------------------
// Transact types
// ---------------------------------------------------------------------------

/// G1 point on the BN254 elliptic curve.
///
/// Mirrors `struct G1Point` in `Globals.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct G1Point {
    /// X coordinate.
    pub x: [u8; 32],
    /// Y coordinate.
    pub y: [u8; 32],
}

/// G2 point on the BN254 elliptic curve.
///
/// Mirrors `struct G2Point` in `Globals.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct G2Point {
    /// X coordinate (two `uint256` field elements).
    pub x: [[u8; 32]; 2],
    /// Y coordinate (two `uint256` field elements).
    pub y: [[u8; 32]; 2],
}

/// SNARK proof for a Railgun transaction.
///
/// Mirrors `struct SnarkProof` in `Globals.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnarkProof {
    /// G1 point `a`.
    pub a: G1Point,
    /// G2 point `b`.
    pub b: G2Point,
    /// G1 point `c`.
    pub c: G1Point,
}

/// Encrypted commitment ciphertext for a shielded output note.
///
/// Mirrors `struct CommitmentCiphertext` in `Globals.sol`.  The
/// `annotation_data` and `memo` are opaque encrypted blobs — only the sender
/// (and intended receiver) can decrypt them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitmentCiphertext<'a> {
    /// Ciphertext: `[IV_tag, encodedMPK, random_amount, token]` across four
    /// `bytes32` words.
    pub ciphertext: [[u8; 32]; 4],
    /// Blinded sender viewing key.
    pub blinded_sender_viewing_key: [u8; 32],
    /// Blinded receiver viewing key.
    pub blinded_receiver_viewing_key: [u8; 32],
    /// Encrypted annotation data (sender-only metadata).  May be empty.
    pub annotation_data: &'a [u8],
    /// Encrypted memo (added to note ciphertext for decryption).  May be empty.
    pub memo: &'a [u8],
}

/// Bound parameters for a Railgun transaction.
///
/// Mirrors `struct BoundParams` in `Globals.sol`.  These parameters bind the
/// SNARK proof to specific execution constraints (tree, gas price, chain,
/// adapt contract).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundParams<'a> {
    /// Tree number in which the transaction operates.
    pub tree_number: u16,
    /// Minimum gas price for type-0 (unshield) transactions.  Must fit in
    /// `uint72` (i.e. `< 2^72`).
    pub min_gas_price: u128,
    /// Unshield type: `None`, `Normal`, or `Redirect`.
    pub unshield: UnshieldType,
    /// Chain ID for the transaction.
    pub chain_id: u64,
    /// Adapt contract address.  Use `[0u8; 20]` for no adapt.
    pub adapt_contract: Address,
    /// Adapt parameters.  Use `[0u8; 32]` for no adapt.
    pub adapt_params: [u8; 32],
    /// Commitment ciphertexts for the output notes of this transaction.
    /// Length must equal the number of new commitments minus unshields.
    pub commitment_ciphertext: &'a [CommitmentCiphertext<'a>],
}

/// A single Railgun transaction.
///
/// Mirrors `struct Transaction` in `Globals.sol`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transaction<'a> {
    /// SNARK proof verifying the transaction validity.
    pub proof: SnarkProof,
    /// Merkle root of the note tree the transaction reads from.
    pub merkle_root: [u8; 32],
    /// Nullifiers preventing double-spends of input notes.
    pub nullifiers: &'a [[u8; 32]],
    /// New commitment hashes added to the tree.
    pub commitments: &'a [[u8; 32]],
    /// Bound parameters binding the proof to execution constraints.
    pub bound_params: BoundParams<'a>,
    /// Unshield preimage (for unshield transactions; use zero values for
    /// pure shielded transfers).
    pub unshield_preimage: CommitmentPreimage,
}

// ---------------------------------------------------------------------------
// Buffer writing helpers
// ---------------------------------------------------------------------------

/// Write a 32-byte word at `offset`.
fn put_word(output: &mut [u8], offset: usize, value: &[u8; 32]) {
    output[offset..offset + 32].copy_from_slice(value);
}

/// Write a left-padded `address` at `offset`.
fn put_addr(output: &mut [u8], offset: usize, value: Address) {
    put_word(output, offset, &leftpad_addr(value));
}

/// Write a `uint8` (left-padded to 32 bytes) at `offset`.
fn put_u8(output: &mut [u8], offset: usize, value: u8) {
    let mut word = [0u8; 32];
    word[31] = value;
    put_word(output, offset, &word);
}

/// Write a `uint16` (left-padded to 32 bytes) at `offset`.
fn put_u16(output: &mut [u8], offset: usize, value: u16) {
    let mut word = [0u8; 32];
    word[30..].copy_from_slice(&value.to_be_bytes());
    put_word(output, offset, &word);
}

/// Write a `uint64` (left-padded to 32 bytes) at `offset`.
fn put_u64(output: &mut [u8], offset: usize, value: u64) {
    let mut word = [0u8; 32];
    word[24..].copy_from_slice(&value.to_be_bytes());
    put_word(output, offset, &word);
}

/// Write a `uint120` / `uint72` / `uint128` (left-padded to 32 bytes) at
/// `offset`.
fn put_u128(output: &mut [u8], offset: usize, value: u128) {
    let mut word = [0u8; 32];
    word[16..].copy_from_slice(&value.to_be_bytes());
    put_word(output, offset, &word);
}

/// Write a `usize` as a `uint256` (left-padded to 32 bytes) at `offset`.
fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let mut word = [0u8; 32];
    let bytes = value.to_be_bytes();
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    put_word(output, offset, &word);
}

/// Write a dynamic-length `bytes` or `string` tail at `offset`.
///
/// Writes: length word + data (right-padded to a multiple of 32).
/// Returns the number of bytes consumed.
fn put_bytes_tail(output: &mut [u8], offset: usize, data: &[u8]) -> usize {
    put_usize(output, offset, data.len());
    let data_start = offset + 32;
    let padded_len = data.len().div_ceil(32) * 32;
    output[data_start..data_start + data.len()].copy_from_slice(data);
    32 + padded_len
}

// ===========================================================================
// shield
// ===========================================================================

/// Each `ShieldRequest` encodes to 9 ABI words (288 bytes):
///
/// ```text
/// preimage.npk(32) + preimage.token.tokenType(32) + tokenAddress(32)
/// + tokenSubID(32) + preimage.value(32)
/// + ciphertext.encryptedBundle[0..3](3*32) + ciphertext.shieldKey(32)
/// ```
const SHIELD_REQUEST_WORDS: usize = 9;
const SHIELD_REQUEST_SIZE: usize = SHIELD_REQUEST_WORDS * 32; // 288

/// Required output length for [`make_fn_shield`] with `n` shield requests.
///
/// Layout:
/// ```text
/// selector(4) + offset_to_array(32) + array_length(32) + n * 288
/// ```
pub const fn shield_calldata_len(n: usize) -> usize {
    4 + 32 + 32 + n * SHIELD_REQUEST_SIZE
}

/// Encode `shield(ShieldRequest[])`.
///
/// This is the core deposit flow: the caller shields tokens into the Railgun
/// privacy pool by creating note commitments.  Each [`ShieldRequest`] creates
/// one commitment in the Merkle tree.
///
/// For native ETH shields, set `token_type = TokenType::Erc20` and
/// `token_address = [0u8; 20]` in the preimage, and send the ETH value as
/// `msg.value` with the transaction.
///
/// For ERC-20 shields, the caller must approve the `RailgunSmartWallet`
/// contract to transfer the token before sending this calldata.  For ERC-721,
/// the caller must be the NFT owner and have approved the contract.
///
/// The caller must supply a buffer of at least
/// [`shield_calldata_len`]`(requests.len())` bytes.  Returns the number of
/// bytes written.
pub fn make_fn_shield(
    output: &mut [u8],
    requests: &[ShieldRequest],
) -> Result<usize, EncodeError> {
    let required = shield_calldata_len(requests.len());
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_SHIELD);

    // Offset to array data (relative to start after selector) = 0x20
    put_usize(out, 4, 32);
    // Array length
    put_usize(out, 4 + 32, requests.len());

    // Array elements (static struct, packed contiguously after length)
    let elements_start = 4 + 64;
    for (i, req) in requests.iter().enumerate() {
        let base = elements_start + i * SHIELD_REQUEST_SIZE;
        let mut w = 0;
        // preimage.npk
        put_word(out, base + w * 32, &req.preimage.npk);
        w += 1;
        // preimage.token.tokenType (uint8)
        put_u8(out, base + w * 32, req.preimage.token.token_type as u8);
        w += 1;
        // preimage.token.tokenAddress
        put_addr(out, base + w * 32, req.preimage.token.token_address);
        w += 1;
        // preimage.token.tokenSubID (uint256)
        put_word(out, base + w * 32, &req.preimage.token.token_sub_id);
        w += 1;
        // preimage.value (uint120)
        put_u128(out, base + w * 32, req.preimage.value);
        w += 1;
        // ciphertext.encryptedBundle[0..3]
        put_word(out, base + w * 32, &req.ciphertext.encrypted_bundle[0]);
        w += 1;
        put_word(out, base + w * 32, &req.ciphertext.encrypted_bundle[1]);
        w += 1;
        put_word(out, base + w * 32, &req.ciphertext.encrypted_bundle[2]);
        w += 1;
        // ciphertext.shieldKey
        put_word(out, base + w * 32, &req.ciphertext.shield_key);
        w += 1;
        debug_assert_eq!(w, SHIELD_REQUEST_WORDS);
    }

    Ok(required)
}

// ===========================================================================
// transact
// ===========================================================================

/// Transaction head: 17 ABI words.
///
/// ```text
/// proof.a.x(32) + proof.a.y(32)
/// + proof.b.x[0..2](2*32) + proof.b.y[0..2](2*32)
/// + proof.c.x(32) + proof.c.y(32)
/// + merkleRoot(32)
/// + offset_to_nullifiers(32) + offset_to_commitments(32) + offset_to_boundParams(32)
/// + unshieldPreimage.npk(32) + token.tokenType(32) + token.tokenAddress(32)
/// + token.tokenSubID(32) + value(32)
/// ```
const TX_HEAD_WORDS: usize = 17;
const TX_HEAD_SIZE: usize = TX_HEAD_WORDS * 32; // 544

/// BoundParams head: 7 ABI words (6 static + 1 offset to commitmentCiphertext).
const BP_HEAD_WORDS: usize = 7;
const BP_HEAD_SIZE: usize = BP_HEAD_WORDS * 32; // 224

/// CommitmentCiphertext head: 8 ABI words (6 static + 2 offsets to bytes).
const CC_HEAD_WORDS: usize = 8;
const CC_HEAD_SIZE: usize = CC_HEAD_WORDS * 32; // 256

/// Compute the encoded size of a single `CommitmentCiphertext` element.
fn cc_element_size(annotation_data_len: usize, memo_len: usize) -> usize {
    let ad_padded = annotation_data_len.div_ceil(32) * 32;
    let memo_padded = memo_len.div_ceil(32) * 32;
    CC_HEAD_SIZE + 32 + ad_padded + 32 + memo_padded
}

/// Compute the encoded size of a `CommitmentCiphertext[]` array.
fn cc_array_size(commitment_ciphertexts: &[CommitmentCiphertext]) -> usize {
    let elements: usize = commitment_ciphertexts
        .iter()
        .map(|cc| cc_element_size(cc.annotation_data.len(), cc.memo.len()))
        .sum();
    // length word + per-element offsets + element data
    32 + commitment_ciphertexts.len() * 32 + elements
}

/// Compute the encoded size of a `BoundParams` dynamic struct.
fn bp_element_size(bound_params: &BoundParams) -> usize {
    BP_HEAD_SIZE + cc_array_size(bound_params.commitment_ciphertext)
}

/// Compute the encoded size of a single `Transaction` element.
fn tx_element_size(tx: &Transaction) -> usize {
    let nullifiers_size = 32 + tx.nullifiers.len() * 32;
    let commitments_size = 32 + tx.commitments.len() * 32;
    TX_HEAD_SIZE + nullifiers_size + commitments_size + bp_element_size(&tx.bound_params)
}

/// Required output length for [`make_fn_transact`].
///
/// The size depends on the number of transactions and the lengths of all
/// dynamic fields (nullifiers, commitments, commitment ciphertexts,
/// annotation data, and memo bytes within each transaction).
pub fn transact_calldata_len(transactions: &[Transaction]) -> Result<usize, EncodeError> {
    let elements_size: usize = transactions.iter().map(tx_element_size).sum();
    4_usize
        .checked_add(32) // offset to array
        .and_then(|s| s.checked_add(32)) // array length
        .and_then(|s| s.checked_add(transactions.len().checked_mul(32)?)) // element offsets
        .and_then(|s| s.checked_add(elements_size)) // element data
        .ok_or(EncodeError::BufferTooSmall { required: 0 })
}

/// Encode `transact(Transaction[])`.
///
/// This is the core shielded transaction flow: the caller submits a batch of
/// zero-knowledge transactions (shielded transfers, unshields, or adapted
/// calls).  Each [`Transaction`] carries a SNARK proof, the Merkle root it
/// reads from, nullifiers for spent notes, new commitments, bound parameters,
/// and an optional unshield preimage.
///
/// The caller must generate valid SNARK proofs off-chain (via the Railgun
/// Engine) before calling this function.  The on-chain contract verifies
/// proofs but does not generate them.
///
/// The caller must supply a buffer of at least
/// [`transact_calldata_len`]`(transactions)` bytes.  Returns the number of
/// bytes written.
pub fn make_fn_transact(
    output: &mut [u8],
    transactions: &[Transaction],
) -> Result<usize, EncodeError> {
    let required = transact_calldata_len(transactions)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_TRANSACT);

    // -- Outer encoding --
    // Offset to array data (relative to start after selector) = 0x20
    put_usize(out, 4, 32);

    // -- Array data --
    let array_start = 4 + 32; // position of array length word
    put_usize(out, array_start, transactions.len());

    // Element offsets (relative to array_start, i.e. position of length word)
    let offsets_start = array_start + 32;
    let mut element_rel_offset = transactions.len() * 32; // skip length + N offsets
    for (i, tx) in transactions.iter().enumerate() {
        put_usize(out, offsets_start + i * 32, element_rel_offset);
        element_rel_offset += tx_element_size(tx);
    }

    // Write each Transaction element
    let mut pos = offsets_start + transactions.len() * 32;
    for tx in transactions {
        let tx_start = pos;

        // -- Transaction head (17 words) --
        let mut w = 0;
        // proof.a (G1Point)
        put_word(out, tx_start + w * 32, &tx.proof.a.x);
        w += 1;
        put_word(out, tx_start + w * 32, &tx.proof.a.y);
        w += 1;
        // proof.b (G2Point)
        put_word(out, tx_start + w * 32, &tx.proof.b.x[0]);
        w += 1;
        put_word(out, tx_start + w * 32, &tx.proof.b.x[1]);
        w += 1;
        put_word(out, tx_start + w * 32, &tx.proof.b.y[0]);
        w += 1;
        put_word(out, tx_start + w * 32, &tx.proof.b.y[1]);
        w += 1;
        // proof.c (G1Point)
        put_word(out, tx_start + w * 32, &tx.proof.c.x);
        w += 1;
        put_word(out, tx_start + w * 32, &tx.proof.c.y);
        w += 1;
        // merkleRoot
        put_word(out, tx_start + w * 32, &tx.merkle_root);
        w += 1;

        // Offsets to dynamic fields (relative to tx_start)
        let nullifiers_tail_len = 32 + tx.nullifiers.len() * 32;
        let commitments_tail_len = 32 + tx.commitments.len() * 32;

        // offset to nullifiers = TX_HEAD_SIZE
        put_usize(out, tx_start + w * 32, TX_HEAD_SIZE);
        w += 1;
        // offset to commitments = TX_HEAD_SIZE + nullifiers_tail_len
        put_usize(out, tx_start + w * 32, TX_HEAD_SIZE + nullifiers_tail_len);
        w += 1;
        // offset to boundParams = TX_HEAD_SIZE + nullifiers_tail_len + commitments_tail_len
        put_usize(
            out,
            tx_start + w * 32,
            TX_HEAD_SIZE + nullifiers_tail_len + commitments_tail_len,
        );
        w += 1;

        // unshieldPreimage (static, 5 words)
        put_word(out, tx_start + w * 32, &tx.unshield_preimage.npk);
        w += 1;
        put_u8(
            out,
            tx_start + w * 32,
            tx.unshield_preimage.token.token_type as u8,
        );
        w += 1;
        put_addr(
            out,
            tx_start + w * 32,
            tx.unshield_preimage.token.token_address,
        );
        w += 1;
        put_word(
            out,
            tx_start + w * 32,
            &tx.unshield_preimage.token.token_sub_id,
        );
        w += 1;
        put_u128(out, tx_start + w * 32, tx.unshield_preimage.value);
        w += 1;
        debug_assert_eq!(w, TX_HEAD_WORDS);

        // -- Transaction tail --
        let mut tail = tx_start + TX_HEAD_SIZE;

        // nullifiers: length + elements
        put_usize(out, tail, tx.nullifiers.len());
        tail += 32;
        for nf in tx.nullifiers {
            put_word(out, tail, nf);
            tail += 32;
        }

        // commitments: length + elements
        put_usize(out, tail, tx.commitments.len());
        tail += 32;
        for comm in tx.commitments {
            put_word(out, tail, comm);
            tail += 32;
        }

        // -- BoundParams (dynamic struct) --
        let bp_start = tail;
        let mut bpw = 0;
        // treeNumber (uint16)
        put_u16(out, bp_start + bpw * 32, tx.bound_params.tree_number);
        bpw += 1;
        // minGasPrice (uint72)
        put_u128(out, bp_start + bpw * 32, tx.bound_params.min_gas_price);
        bpw += 1;
        // unshield (uint8)
        put_u8(out, bp_start + bpw * 32, tx.bound_params.unshield as u8);
        bpw += 1;
        // chainID (uint64)
        put_u64(out, bp_start + bpw * 32, tx.bound_params.chain_id);
        bpw += 1;
        // adaptContract (address)
        put_addr(out, bp_start + bpw * 32, tx.bound_params.adapt_contract);
        bpw += 1;
        // adaptParams (bytes32)
        put_word(out, bp_start + bpw * 32, &tx.bound_params.adapt_params);
        bpw += 1;
        // offset to commitmentCiphertext (relative to bp_start)
        put_usize(out, bp_start + bpw * 32, BP_HEAD_SIZE);
        bpw += 1;
        debug_assert_eq!(bpw, BP_HEAD_WORDS);

        // -- CommitmentCiphertext[] (dynamic array of dynamic structs) --
        let ccs = tx.bound_params.commitment_ciphertext;
        let cc_array_start = bp_start + BP_HEAD_SIZE;
        put_usize(out, cc_array_start, ccs.len());

        let cc_offsets_start = cc_array_start + 32;
        let cc_elements_start = cc_offsets_start + ccs.len() * 32;

        // Element offsets (relative to cc_array_start)
        let mut cc_rel = 32 + ccs.len() * 32;
        for (j, cc) in ccs.iter().enumerate() {
            put_usize(out, cc_offsets_start + j * 32, cc_rel);
            cc_rel += cc_element_size(cc.annotation_data.len(), cc.memo.len());
        }

        // Write each CommitmentCiphertext element
        let mut cc_pos = cc_elements_start;
        for cc in ccs {
            let cc_start = cc_pos;
            let mut ccw = 0;
            // ciphertext[0..4] (bytes32[4])
            for k in 0..4 {
                put_word(out, cc_start + ccw * 32, &cc.ciphertext[k]);
                ccw += 1;
            }
            // blindedSenderViewingKey
            put_word(out, cc_start + ccw * 32, &cc.blinded_sender_viewing_key);
            ccw += 1;
            // blindedReceiverViewingKey
            put_word(out, cc_start + ccw * 32, &cc.blinded_receiver_viewing_key);
            ccw += 1;
            // offset to annotationData (relative to cc_start)
            put_usize(out, cc_start + ccw * 32, CC_HEAD_SIZE);
            ccw += 1;
            // offset to memo (relative to cc_start)
            let ad_tail_len = 32 + (cc.annotation_data.len().div_ceil(32) * 32);
            put_usize(out, cc_start + ccw * 32, CC_HEAD_SIZE + ad_tail_len);
            ccw += 1;
            debug_assert_eq!(ccw, CC_HEAD_WORDS);

            // annotationData + memo tails
            let cc_tail = cc_start + CC_HEAD_SIZE;
            let mut cc_tail_pos = cc_tail;
            cc_tail_pos += put_bytes_tail(out, cc_tail_pos, cc.annotation_data);
            cc_tail_pos += put_bytes_tail(out, cc_tail_pos, cc.memo);

            cc_pos = cc_tail_pos;
        }

        tail = cc_pos;
        pos = tail;
    }

    debug_assert_eq!(pos, required);
    Ok(required)
}

//! Core end-user calldata for Hibachi's Arbitrum custody bridge (`Bridge2`).
//!
//! Hibachi is an off-chain CLOB exchange with on-chain custody on Arbitrum.
//! Users deposit USDC into the bridge contract via EIP-2612 `permit` + `transferFrom`
//! in a single batched call, and request withdrawals that are later finalized by
//! designated finalizers after a dispute period. This module exposes only those
//! two end-user entrypoints; all validator, finalizer, locker, pause, and
//! admin operations are permissioned and intentionally excluded.
//!
//! # Deposit flow
//!
//! `batchedDepositWithPermit((address,uint64,uint64,(uint256,uint256,uint8))[])`
//! atomically calls `permit` then `transferFrom` for each deposit. Each
//! `DepositWithPermit` carries the user's EIP-2612 signature authorising the
//! bridge to spend `usd` units of USDC. The `usd` amount is in 6-decimal USDC
//! micro-units (e.g. `1_000_000` = 1 USDC). The caller must supply the EIP-2612
//! signature over the USDC token's `Permit` struct hash, not an arbitrary
//! signature.
//!
//! For the common single-deposit case, use [`make_fn_deposit_with_permit`] which
//! returns a fixed-size array. For batched deposits, use
//! [`make_fn_batched_deposit_with_permit`] which writes into a caller-supplied
//! buffer.
//!
//! # Withdrawal flow
//!
//! `batchedRequestWithdrawals((address,address,uint64,uint64,(uint256,uint256,uint8)[])[],(uint64,address[],uint64[]))`
//! submits one or more withdrawal requests along with the current hot validator
//! set and validator signatures. Each `WithdrawalRequest` specifies the user,
//! destination, amount (`usd` in micro-units), nonce, and an array of validator
//! `Signature`s over the `requestWithdrawal` message hash. The validator set
//! (`epoch`, `validators`, `powers`) must match the bridge's stored hot
//! validator set hash. Use [`make_fn_batched_request_withdrawals`] which writes
//! into a caller-supplied buffer.
//!
//! Withdrawal finalization (`batchedFinalizeWithdrawals`) is restricted to
//! designated finalizers and is not exposed here.
//!
//! ABI verified against the deployed `Bridge2` contract on Arbitrum One at
//! `0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7` (Sourcify full match, solc
//! 0.8.9, verified 2024-08-08).
//!
//! Sources:
//! - <https://docs.hibachi.xyz/hibachi-docs/getting-started/signing-up>
//! - <https://api-doc.hibachi.xyz/>

use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

/// An ECDSA signature as stored in the bridge's `Signature` struct.
///
/// `r` and `s` are the 256-bit signature components; `v` is the recovery byte
/// (27 or 28 for raw ECDSA, or 0/1 for EIP-155 — the bridge accepts both).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Signature {
    pub r: U,
    pub s: U,
    pub v: u8,
}

/// A single deposit entry for `batchedDepositWithPermit`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepositWithPermit {
    /// The user whose USDC will be deposited.
    pub user: Address,
    /// Amount in 6-decimal USDC micro-units (1 USDC = 1_000_000).
    pub usd: u64,
    /// EIP-2612 permit deadline (unix timestamp).
    pub deadline: u64,
    /// EIP-2612 permit signature from `user` authorising the bridge to spend
    /// `usd` units of USDC.
    pub signature: Signature,
}

/// A single withdrawal request for `batchedRequestWithdrawals`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WithdrawalRequest<'a> {
    /// The user whose funds are being withdrawn.
    pub user: Address,
    /// Destination address for the withdrawn USDC.
    pub destination: Address,
    /// Amount in 6-decimal USDC micro-units.
    pub usd: u64,
    /// Unique nonce for this withdrawal.
    pub nonce: u64,
    /// Validator signatures over the withdrawal message hash.
    pub signatures: &'a [Signature],
}

/// The current validator set submitted with every batched withdrawal request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatorSet<'a> {
    pub epoch: u64,
    pub validators: &'a [Address],
    pub powers: &'a [u64],
}

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Provided buffer was too small; `required` is the minimum length.
    BufferTooSmall { required: usize },
}

selectors! {
    SEL_BATCHED_DEPOSIT_WITH_PERMIT = b"batchedDepositWithPermit((address,uint64,uint64,(uint256,uint256,uint8))[])",
    SEL_BATCHED_REQUEST_WITHDRAWALS = b"batchedRequestWithdrawals((address,address,uint64,uint64,(uint256,uint256,uint8)[])[],(uint64,address[],uint64[]))",
}

// ---------------------------------------------------------------------------
// Buffer writing helpers
// ---------------------------------------------------------------------------

fn put_word(output: &mut [u8], offset: usize, value: &[u8; 32]) {
    output[offset..offset + 32].copy_from_slice(value);
}

fn put_addr(output: &mut [u8], offset: usize, value: Address) {
    put_word(output, offset, &leftpad_addr(value));
}

fn put_u64(output: &mut [u8], offset: usize, value: u64) {
    let mut word = [0u8; 32];
    word[24..].copy_from_slice(&value.to_be_bytes());
    put_word(output, offset, &word);
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let mut word = [0u8; 32];
    let bytes = value.to_be_bytes();
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    put_word(output, offset, &word);
}

/// Encode a single `Signature` struct (3 words) at `offset`.
fn put_signature(output: &mut [u8], offset: usize, sig: &Signature) {
    put_word(output, offset, &sig.r.0);
    put_word(output, offset + 32, &sig.s.0);
    put_word(output, offset + 64, &bobcat_cd::leftpad_u8(sig.v));
}

// ---------------------------------------------------------------------------
// batchedDepositWithPermit — single-deposit convenience (fixed-size output)
// ---------------------------------------------------------------------------

/// Calldata length for a single-deposit `batchedDepositWithPermit` call.
///
/// Layout: selector(4) + offset(32) + length(32) + struct(6×32) = 4 + 8×32 = 260.
pub const DEPOSIT_WITH_PERMIT_CALLDATA_LEN: usize = 4 + 32 * 8;

/// Encode `batchedDepositWithPermit` with a single deposit.
///
/// This is the canonical end-user deposit flow: the user signs an EIP-2612
/// `permit` authorising the bridge contract to spend `deposit.usd` micro-units
/// of USDC, and the bridge atomically calls `permit` then `transferFrom` to
/// move the funds into custody.
pub fn make_fn_deposit_with_permit(deposit: &DepositWithPermit) -> [u8; DEPOSIT_WITH_PERMIT_CALLDATA_LEN] {
    let mut out = [0u8; DEPOSIT_WITH_PERMIT_CALLDATA_LEN];
    out[..4].copy_from_slice(&SEL_BATCHED_DEPOSIT_WITH_PERMIT);
    // offset to the dynamic array: 0x20 (right after this word)
    out[4 + 31] = 0x20;
    // array length: 1
    out[4 + 32 + 31] = 1;
    // struct fields: user, usd, deadline, signature(r, s, v)
    let o = 4 + 64;
    out[o..o + 20].copy_from_slice(&deposit.user);
    out[o + 32 + 24..o + 32 + 32].copy_from_slice(&deposit.usd.to_be_bytes());
    out[o + 64 + 24..o + 64 + 32].copy_from_slice(&deposit.deadline.to_be_bytes());
    out[o + 96..o + 96 + 32].copy_from_slice(&deposit.signature.r.0);
    out[o + 128..o + 128 + 32].copy_from_slice(&deposit.signature.s.0);
    out[o + 160 + 31] = deposit.signature.v;
    out
}

// ---------------------------------------------------------------------------
// batchedDepositWithPermit — batched (buffer-based output)
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_batched_deposit_with_permit`].
///
/// `batchedDepositWithPermit(T[])` where T is a 6-word static struct:
/// selector(4) + offset(32) + length(32) + N × 6 × 32.
pub fn batched_deposit_with_permit_calldata_len(n: usize) -> Result<usize, EncodeError> {
    4_usize
        .checked_add(32 * 2)
        .and_then(|s| s.checked_add(32 * 6 * n))
        .ok_or(EncodeError::BufferTooSmall { required: 0 })
}

/// Encode `batchedDepositWithPermit` for multiple deposits into `output`.
/// Returns the number of bytes written.
///
/// Each `DepositWithPermit` must contain a valid EIP-2612 `permit` signature
/// from `user` authorising the bridge to spend `usd` micro-units of USDC.
/// The caller must supply a buffer of at least
/// [`batched_deposit_with_permit_calldata_len`]`(deposits.len())` bytes.
pub fn make_fn_batched_deposit_with_permit(
    output: &mut [u8],
    deposits: &[DepositWithPermit],
) -> Result<usize, EncodeError> {
    let n = deposits.len();
    let required = batched_deposit_with_permit_calldata_len(n)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_BATCHED_DEPOSIT_WITH_PERMIT);
    // offset to the dynamic array: 0x20
    put_usize(out, 4, 32);
    // array length
    put_usize(out, 4 + 32, n);
    let mut o = 4 + 32 * 2;
    for deposit in deposits {
        put_addr(out, o, deposit.user);
        put_u64(out, o + 32, deposit.usd);
        put_u64(out, o + 64, deposit.deadline);
        put_signature(out, o + 96, &deposit.signature);
        o += 32 * 6;
    }
    Ok(required)
}

// ---------------------------------------------------------------------------
// batchedRequestWithdrawals (buffer-based output)
// ---------------------------------------------------------------------------

/// Required output length for [`make_fn_batched_request_withdrawals`].
///
/// `batchedRequestWithdrawals(WithdrawalRequest[], ValidatorSet)`:
/// - head: selector(4) + offset_to_reqs(32) + offset_to_validator_set(32)
/// - WithdrawalRequest[]: length(32) + N × head(32) + N × tail
/// - ValidatorSet: epoch(32) + 2 offsets(64) + validators tail + powers tail
///
/// Each `WithdrawalRequest` is a dynamic struct (contains `Signature[]`), so
/// its head entry is an offset, and its tail contains: user(32), dest(32),
/// usd(32), nonce(32), sig_offset(32), sig_tail(length + K×96).
pub fn batched_request_withdrawals_calldata_len(
    requests: &[WithdrawalRequest],
    n_validators: usize,
) -> Result<usize, EncodeError> {
    // head: selector + 2 offsets
    let mut len = 4usize + 32 * 2;
    // reqs array: length + N head words (offsets to dynamic structs)
    len = len.checked_add(32).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    len = len.checked_add(32 * requests.len()).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    // req tails: each has user + dest + usd + nonce + sig_offset + sig_array(length + K×3)
    for req in requests {
        len = len.checked_add(32 * 5).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
        len = len.checked_add(32).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
        len = len.checked_add(32 * 3 * req.signatures.len()).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    }
    // ValidatorSet struct (dynamic): epoch + 2 offsets
    len = len.checked_add(32 * 3).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    // validators tail: length + M words
    len = len.checked_add(32).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    len = len.checked_add(32 * n_validators).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    // powers tail: length + M words
    len = len.checked_add(32).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    len = len.checked_add(32 * n_validators).ok_or(EncodeError::BufferTooSmall { required: 0 })?;
    Ok(len)
}

/// Encode `batchedRequestWithdrawals` into `output`. Returns the number of
/// bytes written.
///
/// Each `WithdrawalRequest` contains validator `Signature`s over the
/// `requestWithdrawal` message hash. The `ValidatorSet` must match the
/// bridge's stored hot validator set hash. The caller must supply a buffer of
/// at least [`batched_request_withdrawals_calldata_len`] bytes.
pub fn make_fn_batched_request_withdrawals(
    output: &mut [u8],
    requests: &[WithdrawalRequest],
    validator_set: &ValidatorSet,
) -> Result<usize, EncodeError> {
    let n_validators = validator_set.validators.len();
    if validator_set.powers.len() != n_validators {
        return Err(EncodeError::BufferTooSmall { required: 0 });
    }
    let required = batched_request_withdrawals_calldata_len(requests, n_validators)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let out = &mut output[..required];
    out.fill(0);
    out[..4].copy_from_slice(&SEL_BATCHED_REQUEST_WITHDRAWALS);

    // --- Head: two offsets ---
    // offset to WithdrawalRequest[] = 64 (right after the two head words)
    put_usize(out, 4, 64);

    // --- WithdrawalRequest[] ---
    let reqs_base = 4 + 32 * 2; // start of reqs array section
    let mut o = reqs_base;
    // array length
    put_usize(out, o, requests.len());
    o += 32;
    // head: N offset words (each pointing to the struct tail, relative to reqs_base)
    let head_end = o + 32 * requests.len();
    let mut head_idx = o;
    let mut tail_idx = head_end;

    for req in requests {
        // offset relative to reqs_base
        let struct_offset = tail_idx - reqs_base;
        put_usize(out, head_idx, struct_offset);
        head_idx += 32;

        // struct tail: user, destination, usd, nonce, sig_offset, sig_tail
        put_addr(out, tail_idx, req.user);
        put_addr(out, tail_idx + 32, req.destination);
        put_u64(out, tail_idx + 64, req.usd);
        put_u64(out, tail_idx + 96, req.nonce);
        // offset to signatures array, relative to struct start
        // struct head = 5 words (user, dest, usd, nonce, sig_offset)
        // sig_offset points to: 5 × 32 = 160 bytes from struct start
        put_usize(out, tail_idx + 128, 5 * 32);
        tail_idx += 32 * 5;

        // signatures array: length + K × 3 words
        put_usize(out, tail_idx, req.signatures.len());
        tail_idx += 32;
        for sig in req.signatures {
            put_signature(out, tail_idx, sig);
            tail_idx += 32 * 3;
        }
    }

    // --- ValidatorSet ---
    // offset from the start of the encoded data (after selector) to ValidatorSet
    let validator_set_offset = tail_idx - 4;
    put_usize(out, 4 + 32, validator_set_offset);

    // ValidatorSet struct (dynamic): epoch, offset_to_validators, offset_to_powers
    let vs_base = tail_idx;
    put_u64(out, vs_base, validator_set.epoch);
    // offset to validators array, relative to vs_base = 3 × 32 = 96
    put_usize(out, vs_base + 32, 96);
    // offset to powers array, relative to vs_base
    let validators_len = validator_set.validators.len();
    let powers_offset = 96 + 32 + 32 * validators_len;
    put_usize(out, vs_base + 64, powers_offset);

    // validators array: length + M addr words
    let mut vs_tail = vs_base + 32 * 3;
    put_usize(out, vs_tail, validators_len);
    vs_tail += 32;
    for addr in validator_set.validators {
        put_addr(out, vs_tail, *addr);
        vs_tail += 32;
    }

    // powers array: length + M u64 words
    put_usize(out, vs_tail, validators_len);
    vs_tail += 32;
    for power in validator_set.powers {
        put_u64(out, vs_tail, *power);
        vs_tail += 32;
    }

    debug_assert_eq!(vs_tail, required);
    Ok(required)
}

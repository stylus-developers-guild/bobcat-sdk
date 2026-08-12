//! Narrow Estate Protocol core-flow calldata builders for Arbitrum.
//!
//! Estate Protocol tokenizes regulated real estate on Arbitrum. The subscription
//! flow uses a Polymath-style USDTieredSTO module attached to a security token:
//! an investor approves the STO's accepted stablecoin, then calls
//! `buyWithUSD` to receive fractional security tokens. Holders can later burn
//! their security tokens via the ERC-1410 `redeem` and `redeemByPartition`
//! entrypoints on the security token contract itself.
//!
//! `buyWithUSD` is called on the active STO module. `redeem` and
//! `redeemByPartition` are called on the security token. Both redemption
//! functions are holder-initiated (msg.sender must hold the tokens being
//! burned); operator variants (`redeemFrom`, `redeemByPartition` with
//! `_tokenHolder`) and all STO configuration, cap, finalize, and reclaim
//! functions are permissioned admin/operator flows and are intentionally
//! excluded.
//!
//! ABI reference: official marketplace frontend bundled ABI at
//! <https://www.estateprotocol.com> (USDTieredSTO module + ERC-1410 security token).

use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_BUY_WITH_USD = b"buyWithUSD(address,uint256,address)",
    SEL_REDEEM = b"redeem(uint256,bytes)",
    SEL_REDEEM_BY_PARTITION = b"redeemByPartition(bytes32,uint256,bytes)",
}

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
    BufferTooSmall { required: usize },
}

fn padded_len(length: usize) -> Option<usize> {
    length.checked_add(31).map(|n| n & !31)
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

/// Encode `buyWithUSD(_beneficiary, _investedSC, _usdToken)` on the active STO module.
///
/// Send this calldata to the STO module contract after approving it to spend
/// `_investedSC` of `_usdToken`. The caller must be KYC-verified and whitelisted
/// on the Estate Protocol platform, and the offering must be open.
pub const fn make_fn_buy_with_usd(
    beneficiary: Address,
    invested_sc: &U,
    usd_token: Address,
) -> [u8; 4 + 32 * 3] {
    use array_concat::concat_arrays;
    concat_arrays!(
        SEL_BUY_WITH_USD,
        leftpad_addr(beneficiary),
        invested_sc.0,
        leftpad_addr(usd_token)
    )
}

/// Required output length for [`make_fn_redeem`].
pub fn redeem_calldata_len(data_len: usize) -> Result<usize, EncodeError> {
    let padded = padded_len(data_len).ok_or(EncodeError::LengthOverflow)?;
    4_usize
        .checked_add(32 * 2)
        .and_then(|n| n.checked_add(32))
        .and_then(|n| n.checked_add(padded))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `redeem(_value, _data)` on the security token contract.
///
/// `msg.sender` must hold at least `_value` tokens; the call burns them. `_data`
/// is an opaque holder-provided blob that Estate Protocol's transfer
/// verification hook receives. Pass an empty slice when no metadata is needed.
pub fn make_fn_redeem(output: &mut [u8], value: &U, data: &[u8]) -> Result<usize, EncodeError> {
    let required = redeem_calldata_len(data.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }

    let output = &mut output[..required];
    output.fill(0);
    output[..4].copy_from_slice(&SEL_REDEEM);
    output[4..36].copy_from_slice(&value.0);
    // offset to _data: 2 head words (value + offset) = 64
    put_usize(output, 36, 64);
    // length of _data
    put_usize(output, 68, data.len());
    // data bytes (already zero-padded by fill)
    output[100..100 + data.len()].copy_from_slice(data);
    Ok(required)
}

/// Required output length for [`make_fn_redeem_by_partition`].
pub fn redeem_by_partition_calldata_len(data_len: usize) -> Result<usize, EncodeError> {
    let padded = padded_len(data_len).ok_or(EncodeError::LengthOverflow)?;
    4_usize
        .checked_add(32 * 3)
        .and_then(|n| n.checked_add(32))
        .and_then(|n| n.checked_add(padded))
        .ok_or(EncodeError::LengthOverflow)
}

/// Encode `redeemByPartition(_partition, _value, _data)` on the security token.
///
/// Like [`make_fn_redeem`] but scoped to a specific ERC-1410 partition.
/// `msg.sender` must hold at least `_value` tokens in `_partition`.
pub fn make_fn_redeem_by_partition(
    output: &mut [u8],
    partition: [u8; 32],
    value: &U,
    data: &[u8],
) -> Result<usize, EncodeError> {
    let required = redeem_by_partition_calldata_len(data.len())?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }

    let output = &mut output[..required];
    output.fill(0);
    output[..4].copy_from_slice(&SEL_REDEEM_BY_PARTITION);
    output[4..36].copy_from_slice(&partition);
    output[36..68].copy_from_slice(&value.0);
    // offset to _data: 3 head words (partition + value + offset) = 96
    put_usize(output, 68, 96);
    // length of _data
    put_usize(output, 100, data.len());
    // data bytes (already zero-padded by fill)
    output[132..132 + data.len()].copy_from_slice(data);
    Ok(required)
}

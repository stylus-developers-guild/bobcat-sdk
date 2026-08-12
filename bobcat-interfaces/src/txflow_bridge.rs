//! Core end-user calldata for TxFlow's Arbitrum bridge.
//!
//! TxFlow's standard wallet flow deposits native USDC by transferring it to the bridge. Send the
//! generated calldata to the USDC token contract, with the bridge supplied as `bridge`.
//!
//! Withdrawals are initiated through TxFlow and finalized on Arbitrum by designated finalizers, so
//! this module intentionally exposes no withdrawal, finalizer, validator, locker, or admin calls.
//!
//! Sources:
//! - <https://docs.txflow.com/getting-started/mainnet-onboarding>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_TRANSFER = b"transfer(address,uint256)",
}

/// Encode `USDC.transfer(bridge, amount)` for a TxFlow deposit.
///
/// Send this calldata to the native USDC token contract on the source chain. `bridge` is the
/// TxFlow bridge supplied by TxFlow; no deployment addresses are embedded here.
pub const fn make_fn_deposit(bridge: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_TRANSFER, leftpad_addr(bridge), amount.0)
}

//! Calldata builders for USD.AI's borrower-facing Loan Router V2 flow.
//!
//! The ABI follows the USD.AI Foundation's `usdai-loan-router-contracts` interfaces. Loan
//! origination and collateral release are protocol-operator actions, so this module exposes only
//! the collateral depositor and borrower calls used around those actions.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_usize};
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_SUPPLY_COLLATERAL = b"deposit(address,bytes32,address,uint256[],uint64)",
    SEL_WITHDRAW_COLLATERAL = b"cancel(address,bytes32,address,uint256[])",
    SEL_WITHDRAW_BORROWED_FUNDS = b"withdraw(address,uint256)",
    SEL_REPAY = b"repay((uint64,address,address,address,(uint16,uint8,int32),(address,bytes),uint256[],(address,uint256,uint256)[],(uint8,address,address,bytes)[],address[],bytes),uint256)",
}

/// Fixed calldata length before the collateral token IDs for `deposit`.
pub const SUPPLY_COLLATERAL_BASE_LEN: usize = 4 + 32 * 6;

/// Fixed calldata length before the collateral token IDs for `cancel`.
pub const WITHDRAW_COLLATERAL_BASE_LEN: usize = 4 + 32 * 5;

const fn leftpad_u64(value: u64) -> [u8; 32] {
    concat_arrays!([0u8; 24], value.to_be_bytes())
}

const fn copy_into<const OUT_LEN: usize, const VALUE_LEN: usize>(
    output: &mut [u8; OUT_LEN],
    offset: usize,
    value: &[u8; VALUE_LEN],
) {
    let mut i = 0;
    while i < VALUE_LEN {
        output[offset + i] = value[i];
        i += 1;
    }
}

/// Encode `CollateralTimelock.deposit` to escrow collateral for approved loan terms.
///
/// `target` is the Loan Router, `context` is the loan-terms hash, and `ALL_LEN` must equal
/// `SUPPLY_COLLATERAL_BASE_LEN + TOKEN_IDS * 32`.
pub const fn make_fn_supply_collateral<const TOKEN_IDS: usize, const ALL_LEN: usize>(
    target: Address,
    context: [u8; 32],
    token: Address,
    token_ids: &[U; TOKEN_IDS],
    expiration: u64,
) -> [u8; ALL_LEN] {
    assert!(
        ALL_LEN == SUPPLY_COLLATERAL_BASE_LEN + TOKEN_IDS * 32,
        "make_fn_supply_collateral inconsistent length"
    );

    let mut output = [0u8; ALL_LEN];
    copy_into(&mut output, 0, &SEL_SUPPLY_COLLATERAL);
    copy_into(&mut output, 4, &leftpad_addr(target));
    copy_into(&mut output, 4 + 32, &context);
    copy_into(&mut output, 4 + 32 * 2, &leftpad_addr(token));
    copy_into(&mut output, 4 + 32 * 3, &leftpad_usize(32 * 5));
    copy_into(&mut output, 4 + 32 * 4, &leftpad_u64(expiration));
    copy_into(&mut output, 4 + 32 * 5, &leftpad_usize(TOKEN_IDS));

    let mut i = 0;
    while i < TOKEN_IDS {
        copy_into(&mut output, 4 + 32 * (6 + i), &token_ids[i].0);
        i += 1;
    }
    output
}

/// Encode `CollateralTimelock.cancel` to recover collateral after its timelock expires.
///
/// `ALL_LEN` must equal `WITHDRAW_COLLATERAL_BASE_LEN + TOKEN_IDS * 32`.
pub const fn make_fn_withdraw_collateral<const TOKEN_IDS: usize, const ALL_LEN: usize>(
    target: Address,
    context: [u8; 32],
    token: Address,
    token_ids: &[U; TOKEN_IDS],
) -> [u8; ALL_LEN] {
    assert!(
        ALL_LEN == WITHDRAW_COLLATERAL_BASE_LEN + TOKEN_IDS * 32,
        "make_fn_withdraw_collateral inconsistent length"
    );

    let mut output = [0u8; ALL_LEN];
    copy_into(&mut output, 0, &SEL_WITHDRAW_COLLATERAL);
    copy_into(&mut output, 4, &leftpad_addr(target));
    copy_into(&mut output, 4 + 32, &context);
    copy_into(&mut output, 4 + 32 * 2, &leftpad_addr(token));
    copy_into(&mut output, 4 + 32 * 3, &leftpad_usize(32 * 4));
    copy_into(&mut output, 4 + 32 * 4, &leftpad_usize(TOKEN_IDS));

    let mut i = 0;
    while i < TOKEN_IDS {
        copy_into(&mut output, 4 + 32 * (5 + i), &token_ids[i].0);
        i += 1;
    }
    output
}

/// Encode `ReserveAccount.withdraw` to draw available borrowed funds.
pub const fn make_fn_withdraw_borrowed_funds(recipient: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(
        SEL_WITHDRAW_BORROWED_FUNDS,
        leftpad_addr(recipient),
        amount.0
    )
}

/// Encode `LoanRouterV2.repay` (also used by `ReserveAccount.repay`).
///
/// `loan_terms_abi` is the canonical ABI encoding of the `LoanTermsV2` tuple body, without an
/// outer offset or function selector. It must be word-aligned. `ALL_LEN` must equal
/// `4 + 64 + LOAN_TERMS_LEN`.
pub const fn make_fn_repay<const LOAN_TERMS_LEN: usize, const ALL_LEN: usize>(
    loan_terms_abi: [u8; LOAN_TERMS_LEN],
    amount: &U,
) -> [u8; ALL_LEN] {
    assert!(
        LOAN_TERMS_LEN % 32 == 0,
        "make_fn_repay loan terms must be word-aligned"
    );
    assert!(
        ALL_LEN == 4 + 64 + LOAN_TERMS_LEN,
        "make_fn_repay inconsistent length"
    );
    concat_arrays!(SEL_REPAY, leftpad_usize(64), amount.0, loan_terms_abi)
}

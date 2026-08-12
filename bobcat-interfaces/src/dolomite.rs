//! Narrow Dolomite router calldata builders for core margin-account lending.
//!
//! Deposits and withdrawals target `DepositWithdrawalRouter`. Borrow positions use
//! `BorrowPositionRouter`: open a position by moving collateral into a borrow account,
//! borrow by transferring an asset out of that account, and repay its full debt from
//! another account. Isolation-mode callers provide the vault market ID; use zero for a
//! standard margin account.
//!
//! The ABI surface is intentionally limited to these end-user flows. Native-token,
//! Par-denominated, position-closing, vault-only, view, and administrative functions are
//! not included.

use array_concat::concat_arrays;
use bobcat_cd::leftpad_u8;
use bobcat_maths::U;

use crate::selectors;

/// Post-operation non-negative balance checks used by Dolomite's account routers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BalanceCheckFlag {
    Both = 0,
    From = 1,
    To = 2,
    None = 3,
}

/// Optional event emitted by `DepositWithdrawalRouter::depositWei`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DepositEventFlag {
    None = 0,
    Borrow = 1,
}

selectors! {
    SEL_DEPOSIT_WEI = b"depositWei(uint256,uint256,uint256,uint256,uint8)",
    SEL_WITHDRAW_WEI = b"withdrawWei(uint256,uint256,uint256,uint256,uint8)",
    SEL_OPEN_BORROW_POSITION = b"openBorrowPosition(uint256,uint256,uint256,uint256,uint256,uint8)",
    SEL_TRANSFER_BETWEEN_ACCOUNTS = b"transferBetweenAccounts(uint256,uint256,uint256,uint256,uint256,uint8)",
    SEL_REPAY_ALL_FOR_BORROW_POSITION = b"repayAllForBorrowPosition(uint256,uint256,uint256,uint256,uint8)"
}

/// Encode a Wei-denominated token deposit into a margin account.
///
/// `isolation_mode_market_id` is zero for standard accounts. The router accepts
/// `U::MAX` as `amount_wei` to deposit the caller's entire token balance.
pub const fn make_fn_deposit_wei(
    isolation_mode_market_id: &U,
    to_account_number: &U,
    market_id: &U,
    amount_wei: &U,
    event_flag: DepositEventFlag,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_DEPOSIT_WEI,
        isolation_mode_market_id.0,
        to_account_number.0,
        market_id.0,
        amount_wei.0,
        leftpad_u8(event_flag as u8)
    )
}

/// Encode a Wei-denominated token withdrawal from a margin account.
///
/// `isolation_mode_market_id` is zero for standard accounts. The router accepts
/// `U::MAX` as `amount_wei` to withdraw the account's full positive balance.
pub const fn make_fn_withdraw_wei(
    isolation_mode_market_id: &U,
    from_account_number: &U,
    market_id: &U,
    amount_wei: &U,
    balance_check_flag: BalanceCheckFlag,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_WITHDRAW_WEI,
        isolation_mode_market_id.0,
        from_account_number.0,
        market_id.0,
        amount_wei.0,
        leftpad_u8(balance_check_flag as u8)
    )
}

/// Encode moving collateral from an existing account into a borrow account.
///
/// This opens the position; use [`make_fn_transfer_between_accounts`] to move the
/// borrowed asset from the borrow account to a receiving account.
pub const fn make_fn_open_borrow_position(
    isolation_mode_market_id: &U,
    from_account_number: &U,
    to_account_number: &U,
    collateral_market_id: &U,
    amount: &U,
    balance_check_flag: BalanceCheckFlag,
) -> [u8; 4 + 32 * 6] {
    concat_arrays!(
        SEL_OPEN_BORROW_POSITION,
        isolation_mode_market_id.0,
        from_account_number.0,
        to_account_number.0,
        collateral_market_id.0,
        amount.0,
        leftpad_u8(balance_check_flag as u8)
    )
}

/// Encode an asset transfer between margin accounts.
///
/// To borrow, transfer the borrowed market from the borrow account to a receiving
/// account and select a balance check that permits the borrow account's debt.
pub const fn make_fn_transfer_between_accounts(
    isolation_mode_market_id: &U,
    from_account_number: &U,
    to_account_number: &U,
    market_id: &U,
    amount: &U,
    balance_check_flag: BalanceCheckFlag,
) -> [u8; 4 + 32 * 6] {
    concat_arrays!(
        SEL_TRANSFER_BETWEEN_ACCOUNTS,
        isolation_mode_market_id.0,
        from_account_number.0,
        to_account_number.0,
        market_id.0,
        amount.0,
        leftpad_u8(balance_check_flag as u8)
    )
}

/// Encode repaying the full debt for one market in a borrow account.
pub const fn make_fn_repay_all_for_borrow_position(
    isolation_mode_market_id: &U,
    from_account_number: &U,
    borrow_account_number: &U,
    market_id: &U,
    balance_check_flag: BalanceCheckFlag,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_REPAY_ALL_FOR_BORROW_POSITION,
        isolation_mode_market_id.0,
        from_account_number.0,
        borrow_account_number.0,
        market_id.0,
        leftpad_u8(balance_check_flag as u8)
    )
}

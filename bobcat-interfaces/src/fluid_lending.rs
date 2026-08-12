//! Calldata builders for Fluid Vault T1 lending positions.
//!
//! Fluid Vault T1 exposes supply, withdraw, borrow, and repay through
//! `operate(uint256,int256,int256,address)`.
//!
//! ABI references:
//! - <https://docs.fluid.instadapp.io/fluid-integration/borrow/write/deposit-borrow-t1.html>
//! - <https://docs.fluid.instadapp.io/fluid-integration/borrow/write/repay-withdraw-t1.html>
//! - <https://github.com/Instadapp/fluid-contracts-public/blob/main/contracts/protocols/vault/interfaces/iVaultT1.sol>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::{I, U};

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_OPERATE = b"operate(uint256,int256,int256,address)"
}

const fn positive_amount(amount: &U) -> [u8; 32] {
    assert!(amount.0[0] & 0x80 == 0, "amount exceeds int256::MAX");
    amount.0
}

const fn negative_amount(amount: &U) -> [u8; 32] {
    let mut valid = amount.0[0] < 0x80;
    if amount.0[0] == 0x80 {
        valid = true;
        let mut i = 1;
        while i < 32 {
            if amount.0[i] != 0 {
                valid = false;
            }
            i += 1;
        }
    }
    assert!(valid, "amount exceeds int256 minimum magnitude");

    let mut out = [0u8; 32];
    let mut carry = 1u16;
    let mut i = 32;
    while i > 0 {
        i -= 1;
        let value = (!amount.0[i]) as u16 + carry;
        out[i] = value as u8;
        carry = value >> 8;
    }
    out
}

/// Supplies collateral. Pass NFT id zero to open a new position.
pub const fn make_fn_supply(nft_id: &U, amount: &U, to: Address) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_OPERATE,
        nft_id.0,
        positive_amount(amount),
        I::ZERO.0,
        leftpad_addr(to)
    )
}

/// Withdraws collateral from an existing position.
pub const fn make_fn_withdraw(nft_id: &U, amount: &U, to: Address) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_OPERATE,
        nft_id.0,
        negative_amount(amount),
        I::ZERO.0,
        leftpad_addr(to)
    )
}

/// Borrows debt from a position.
pub const fn make_fn_borrow(nft_id: &U, amount: &U, to: Address) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_OPERATE,
        nft_id.0,
        I::ZERO.0,
        positive_amount(amount),
        leftpad_addr(to)
    )
}

/// Repays debt on a position.
pub const fn make_fn_repay(nft_id: &U, amount: &U, to: Address) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_OPERATE,
        nft_id.0,
        I::ZERO.0,
        negative_amount(amount),
        leftpad_addr(to)
    )
}

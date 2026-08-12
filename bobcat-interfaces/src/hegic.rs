//! Core end-user calldata builders for Hegic V8888 options on Arbitrum.
//!
//! Hegic options are ERC-721 tokenised positions managed by an `OptionsManager`.
//! Users **buy** options through the `Facade` contract, which pulls the premium
//! from the buyer and calls `HegicPool.sellOption`. Users **exercise** directly
//! on the `HegicPool` that backs their option token (the pool address is
//! available from `OptionsManager.tokenPool(tokenId)`).
//!
//! Only the two core end-user flows are exposed. Liquidity-provider
//! (`provideFrom`, `withdraw`), pool configuration, staking, `unlock`, and
//! `Exerciser` auto-exercise are protocol-operator or secondary flows and are
//! intentionally omitted.
//!
//! Flow references:
//! - Hegic V8888 `Facade.createOption` — buy an option
//! - Hegic V8888 `HegicPool.exercise` — exercise an ITM option before expiry

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_CREATE_OPTION = b"createOption(address,uint256,uint256,uint256,address[],uint256)",
    SEL_EXERCISE = b"exercise(uint256)",
}

/// Fixed calldata length before the `swappath` array elements for `createOption`.
///
/// `4 (selector) + 6 * 32 (head) + 32 (array length)` = `4 + 7 * 32`.
pub const CREATE_OPTION_BASE_LEN: usize = 4 + 32 * 7;

/// Offset from the start of the parameter block to the `swappath` dynamic data.
/// Equals `6 * 32` (six head words before the tail begins).
const SWAPPATH_OFFSET: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 192,
];

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

/// Encode `Facade.createOption(pool, period, amount, strike, swappath, acceptablePrice)`.
///
/// Buy a Hegic V8888 option through the Facade. The caller must first approve
/// `swappath[0]` (the payment token) to spend at least the option premium on the
/// Facade contract. When `swappath` has a single element (the pool's settlement
/// token) no swap is performed; when it has two or more the Facade routes through
/// the configured Uniswap V2 router to convert the payment token into the pool's
/// token.
///
/// `ALL_LEN` must equal `CREATE_OPTION_BASE_LEN + SWAPPATH_LEN * 32`.
pub const fn make_fn_create_option<const SWAPPATH_LEN: usize, const ALL_LEN: usize>(
    pool: Address,
    period: &U,
    amount: &U,
    strike: &U,
    swappath: &[Address; SWAPPATH_LEN],
    acceptable_price: &U,
) -> [u8; ALL_LEN] {
    assert!(
        ALL_LEN == CREATE_OPTION_BASE_LEN + SWAPPATH_LEN * 32,
        "make_fn_create_option inconsistent length"
    );

    let mut output = [0u8; ALL_LEN];
    copy_into(&mut output, 0, &SEL_CREATE_OPTION);
    copy_into(&mut output, 4, &leftpad_addr(pool));
    copy_into(&mut output, 4 + 32, &period.0);
    copy_into(&mut output, 4 + 32 * 2, &amount.0);
    copy_into(&mut output, 4 + 32 * 3, &strike.0);
    copy_into(&mut output, 4 + 32 * 4, &SWAPPATH_OFFSET);
    copy_into(&mut output, 4 + 32 * 5, &acceptable_price.0);

    // Tail: array length followed by each address left-padded to 32 bytes.
    let length_word = leftpad_usize(SWAPPATH_LEN);
    copy_into(&mut output, 4 + 32 * 6, &length_word);

    let mut i = 0;
    while i < SWAPPATH_LEN {
        copy_into(
            &mut output,
            4 + 32 * (7 + i),
            &leftpad_addr(swappath[i]),
        );
        i += 1;
    }
    output
}

/// Encode `HegicPool.exercise(id)`.
///
/// Exercise an in-the-money V8888 option before expiry. Send this calldata to
/// the `HegicPool` contract that backs the option (retrievable via
/// `OptionsManager.tokenPool(tokenId)`). The caller must be the owner of the
/// option ERC-721 token, or approved by the owner.
pub const fn make_fn_exercise(option_id: &U) -> [u8; 4 + 32] {
    concat_arrays!(SEL_EXERCISE, option_id.0)
}

const fn leftpad_usize(value: usize) -> [u8; 32] {
    concat_arrays!([0u8; 32 - core::mem::size_of::<usize>()], value.to_be_bytes())
}

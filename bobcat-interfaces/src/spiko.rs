use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];

selectors! {
    SEL_INITIATE_MINT = b"initiateMint(address,address,uint256,bytes32)",
    SEL_TRANSFER_AND_CALL = b"transferAndCall(address,uint256,bytes)",
    SEL_MINT_STATE_STATUS = b"mintStateStatus(bytes32)",
    SEL_DETAILS = b"details(bytes32)",
}

const DYNAMIC_DATA_OFFSET: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96,
];
const REDEMPTION_DATA_LENGTH: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64,
];

/// Encode ERC-1363 `transferAndCall(redemption_manager, amount, abi.encode(output, salt))`.
///
/// Send this calldata to a supported Spiko fund token, not to the redemption manager.
pub const fn make_fn_transfer_and_call(
    redemption_manager: Address,
    amount: &U,
    output: Address,
    salt: [u8; 32],
) -> [u8; 4 + 32 * 6] {
    concat_arrays!(
        SEL_TRANSFER_AND_CALL,
        leftpad_addr(redemption_manager),
        amount.0,
        DYNAMIC_DATA_OFFSET,
        REDEMPTION_DATA_LENGTH,
        leftpad_addr(output),
        salt
    )
}

/// Encode `Minter.mintStateStatus(id)`.
pub const fn make_fn_mint_state_status(id: [u8; 32]) -> [u8; 4 + 32] {
    concat_arrays!(SEL_MINT_STATE_STATUS, id)
}

/// Encode `Redemption.details(id)`.
pub const fn make_fn_details(id: [u8; 32]) -> [u8; 4 + 32] {
    concat_arrays!(SEL_DETAILS, id)
}

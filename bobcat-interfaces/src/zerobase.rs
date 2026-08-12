use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_STAKE = b"stake_66380860(address,uint256)",
    SEL_REQUEST_CLAIM = b"requestClaim_8135334(address,uint256)",
    SEL_CLAIM = b"claim_41202704(uint256,address)",
    SEL_CANCEL_CLAIM = b"cancelClaim(uint256,address)",
    SEL_FLASH_WITHDRAW = b"flashWithdrawWithPenalty(address,uint256)",
}

/// Encode `stake_66380860(_token, _stakedAmount)` — deposit tokens into the ZEROBASE vault.
///
/// The caller must first approve the vault to spend `_token` via ERC-20 `approve`.
pub const fn make_fn_stake(token: Address, staked_amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_STAKE, leftpad_addr(token), staked_amount.0)
}

/// Encode `requestClaim_8135334(_token, _amount)` — initiate a withdrawal request.
///
/// Returns a queue ID that is later used with `claim_41202704` to complete the withdrawal
/// after the protocol's waiting period elapses.
pub const fn make_fn_request_claim(token: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_REQUEST_CLAIM, leftpad_addr(token), amount.0)
}

/// Encode `claim_41202704(_queueID, _token)` — complete a previously requested withdrawal.
///
/// `_queueID` is the value returned by `requestClaim_8135334`. The waiting period must
/// have elapsed before this can be called successfully.
pub const fn make_fn_claim(queue_id: &U, token: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_CLAIM, queue_id.0, leftpad_addr(token))
}

/// Encode `cancelClaim(_queueId, _token)` — cancel a pending withdrawal request.
pub const fn make_fn_cancel_claim(queue_id: &U, token: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_CANCEL_CLAIM, queue_id.0, leftpad_addr(token))
}

/// Encode `flashWithdrawWithPenalty(_token, _amount)` — withdraw immediately with a penalty,
/// bypassing the normal waiting period.
pub const fn make_fn_flash_withdraw(token: Address, amount: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_FLASH_WITHDRAW, leftpad_addr(token), amount.0)
}

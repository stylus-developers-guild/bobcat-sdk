//! Renzo liquid-restaking calldata builders for core end-user deposit and
//! withdrawal-request flows.
//!
//! Renzo is a liquid-restaking protocol built on top of EigenLayer.  On
//! Arbitrum, Renzo deploys `xRenzoDepositNativeBridge` — a canonical bridge that
//! accepts native ETH, WETH, or wstETH deposits and mints L2 ezETH (xezETH) at
//! the current ezETH/ETH price, charging a small bridge fee and a time-discount
//! fee to account for the 7-day canonical bridge delay.  The deposited assets
//! are periodically swept to Ethereum mainnet and restaked into EigenLayer.
//!
//! ## Deposit flows (Arbitrum)
//!
//! - [`make_fn_deposit_eth`] encodes `depositETH(uint256,uint256)`: sends
//!   `msg.value` of native ETH and mints xezETH to the caller.  The caller
//!   must supply a minimum output (`_minOut`) and a deadline.
//! - [`make_fn_deposit`] encodes `deposit(address,uint256,uint256,uint256)`:
//!   transfers an ERC20 deposit token (WETH or wstETH) from `msg.sender` into
//!   the bridge and mints xezETH.  The caller must have approved the bridge to
//!   spend `_amountIn` of `_token`.
//!
//! ## Withdrawal request flow (Ethereum mainnet)
//!
//! To withdraw, a user bridges their L2 xezETH back to L1 ezETH and then calls
//! `WithdrawQueue.withdraw(uint256,address)`, which burns ezETH and creates a
//! withdrawal request.  After a cooldown period the user claims the withdrawn
//! assets via `WithdrawQueue.claim(uint256,address)`.
//!
//! - [`make_fn_withdraw`] encodes the `withdraw(uint256,address)` call that
//!   creates the withdrawal request.  `_assetOut` specifies the output token
//!   (e.g. the zero address for native ETH, or a supported collateral token
//!   address).  The caller must have approved the WithdrawQueue to spend the
//!   `_amount` of ezETH.
//!
//! Claim (`claim`) and all admin/operator/sweep functions are permissioned or
//! outside the core end-user lifecycle and are intentionally excluded.
//!
//! ABI references:
//! - `xRenzoDepositNativeBridge.sol`:
//!   <https://github.com/Renzo-Protocol/contracts-public/blob/master/contracts/Bridge/L2/xRenzoDepositNativeBridge.sol>
//! - `IxRenzoDeposit.sol`:
//!   <https://github.com/Renzo-Protocol/contracts-public/blob/master/contracts/Bridge/L2/IxRenzoDeposit.sol>
//! - `WithdrawQueue.sol`:
//!   <https://github.com/Renzo-Protocol/contracts-public/blob/master/contracts/Withdraw/WithdrawQueue.sol>
//! - `IRestakeManager.sol`:
//!   <https://github.com/Renzo-Protocol/contracts-public/blob/master/contracts/IRestakeManager.sol>
//!
//! Protocol documentation:
//! - <https://docs.renzoprotocol.com/docs>
//! - L2 Native Restaking: <https://docs.renzoprotocol.com/docs/integrations/l2-native-restaking>
//! - Contracts: <https://docs.renzoprotocol.com/docs/contracts/layer-2s/arbitrum>

use array_concat::concat_arrays;
use bobcat_cd::leftpad_addr;
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

selectors! {
    SEL_DEPOSIT_ETH = b"depositETH(uint256,uint256)",
    SEL_DEPOSIT = b"deposit(address,uint256,uint256,uint256)",
    SEL_WITHDRAW = b"withdraw(uint256,address)",
}

/// Encode `xRenzoDepositNativeBridge.depositETH(uint256 _minOut, uint256 _deadline)`.
///
/// Sends `msg.value` of native ETH to the bridge and mints L2 ezETH (xezETH)
/// to the caller.  The bridge deducts a fee (up to 5 bps of the deposit, capped
/// at the 32-ETH threshold) and a time-discount fee before computing the ETH
/// value used for minting.
///
/// `_minOut` is the minimum amount of xezETH to accept (slippage protection).
/// `_deadline` is the latest unix timestamp at which the transaction is still
/// valid.
///
/// The caller must send the desired ETH amount as `msg.value` alongside this
/// calldata.
pub const fn make_fn_deposit_eth(min_out: &U, deadline: &U) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_DEPOSIT_ETH, min_out.0, deadline.0)
}

/// Encode `xRenzoDepositNativeBridge.deposit(address _token, uint256 _amountIn,
/// uint256 _minOut, uint256 _deadline)`.
///
/// Transfers `_amountIn` of `_token` (WETH or wstETH, 18 decimals) from
/// `msg.sender` into the bridge and mints L2 ezETH (xezETH).  The caller must
/// have approved the bridge to spend `_amountIn` of `_token`.
///
/// The bridge converts the deposit token to its ETH value using a Chainlink
/// oracle, deducts the bridge fee and time-discount fee, then mints xezETH at
/// the current ezETH/ETH price.  WETH deposits are unwrapped to ETH before
/// minting.
///
/// `_minOut` is the minimum amount of xezETH to accept (slippage protection).
/// `_deadline` is the latest unix timestamp at which the transaction is still
/// valid.
pub const fn make_fn_deposit(
    token: Address,
    amount_in: &U,
    min_out: &U,
    deadline: &U,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT,
        leftpad_addr(token),
        amount_in.0,
        min_out.0,
        deadline.0
    )
}

/// Encode `WithdrawQueue.withdraw(uint256 _amount, address _assetOut)`.
///
/// Creates a withdrawal request by burning `_amount` of ezETH and queuing it
/// for withdrawal into `_assetOut`.  The caller must have approved the
/// WithdrawQueue to spend `_amount` of ezETH.
///
/// `_assetOut` specifies the output token.  Pass the zero address
/// (`[0u8; 20]`) for native ETH, or the address of a supported collateral
/// token.  Only tokens configured in `withdrawalBufferTarget` are accepted;
/// unsupported assets revert.
///
/// After the cooldown period elapses, the user (or anyone on their behalf)
/// calls `claim(withdrawRequestIndex, user)` to receive the withdrawn assets.
/// The claim step is not covered by this module.
pub const fn make_fn_withdraw(amount: &U, asset_out: Address) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW, amount.0, leftpad_addr(asset_out))
}

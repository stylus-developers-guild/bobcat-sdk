use stylus_sdk::{
    alloy_primitives::*,
    alloy_sol_types::{SolCall, sol},
    prelude::*,
};

extern crate alloc;

use alloc::{vec, vec::Vec};

// It would be better in this example to use a safe transfer method by
// checking the codesize beforehand, since not every ERC20 will revert if
// something is wrong. But, we're being kind to this example, so we won't
// check.

#[entrypoint]
#[storage]
pub struct Swapper;

sol! {
    interface ICamelotSwapRouter {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 limitSqrtPrice;
        }

        function exactInputSingle(
            ExactInputSingleParams memory params
        ) external payable returns (uint256 amountOut);
    }
}

sol_interface! {
    interface IERC20 {
        function transferFrom(address sender, address recipient, uint256 amount) external;
    }
}

pub const SWAP_ROUTER: Address = address!("C216fCdEb961EEF95657Cb45dEe20e379C7624B8");

#[public]
impl Swapper {
    pub fn make_swap(
        &mut self,
        token_in: IERC20,
        token_out: IERC20,
        amount_in: U256,
        amount_out_min: U256,
    ) -> Result<U256, Vec<u8>> {
        let sender = self.vm().msg_sender();
        let contract_addr = self.vm().contract_address();
        let deadline = self.vm().block_timestamp() + 1;
        let transfer = Call::new_mutating(self);
        token_in.transfer_from(self.vm(), transfer, sender, contract_addr, amount_in)?;
        let swap = Call::new_mutating(self);
        let c = call(
            self.vm(),
            swap,
            SWAP_ROUTER,
            &ICamelotSwapRouter::exactInputSingleCall {
                params: ICamelotSwapRouter::ExactInputSingleParams {
                    tokenIn: *token_in,
                    tokenOut: *token_out,
                    recipient: self.vm().msg_sender(),
                    deadline: U256::from(deadline),
                    amountIn: amount_in,
                    amountOutMinimum: amount_out_min,
                    limitSqrtPrice: U160::MAX,
                },
            }
            .abi_encode(),
        )
        .unwrap();
        let amount_out = ICamelotSwapRouter::exactInputSingleCall::abi_decode_returns(&c).unwrap();
        Ok(amount_out)
    }
}

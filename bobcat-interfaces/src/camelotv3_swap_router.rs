use bobcat_maths::U;

use bobcat_cd::{leftpad_addr, leftpad_u24};

use crate::selectors;

use array_concat::concat_arrays;

type Address = [u8; 20];

pub type U24 = [u8; 3];

pub type U160 = [u8; 20];

selectors! {
    SEL_EXACT_INPUT_SINGLE = b"exactInputSingle((address,address,address,uint256,uint256,uint256,uint160))",
    SEL_EXACT_OUTPUT_SINGLE = b"exactOutputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))"
}

pub const fn make_fn_exact_input_single(
    token_in: Address,
    token_out: Address,
    recipient: Address,
    deadline: U,
    amount_in: U,
    amount_out_min: U,
    limit_sqrt_price: U160,
) -> [u8; 4 + 32 * 7] {
    concat_arrays!(
        SEL_EXACT_INPUT_SINGLE,
        leftpad_addr(token_in),
        leftpad_addr(token_out),
        leftpad_addr(recipient),
        deadline.0,
        amount_in.0,
        amount_out_min.0,
        leftpad_addr(limit_sqrt_price)
    )
}

pub const fn make_fn_exact_output_single(
    token_in: Address,
    token_out: Address,
    fee: U24,
    recipient: Address,
    deadline: U,
    amount_out: U,
    amount_in_maximum: U,
    limit_sqrt_price: U160,
) -> [u8; 4 + 32 * 8] {
    // Note: 8 parameters now
    concat_arrays!(
        SEL_EXACT_OUTPUT_SINGLE,
        leftpad_addr(token_in),
        leftpad_addr(token_out),
        leftpad_u24(fee),
        leftpad_addr(recipient),
        deadline.0,
        amount_out.0,
        amount_in_maximum.0,
        leftpad_addr(limit_sqrt_price)
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use super::*;

    use proptest::prelude::*;

    use alloy_sol_macro::sol;

    use alloy_sol_types::SolCall;

    use alloy_primitives::{Address as AAddress, U160, U256 as AU, aliases::U24 as AU24};

    sol! {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 limitSqrtPrice;
        }

        struct ExactOutputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountOut;
            uint256 amountInMaximum;
            uint160 limitSqrtPrice;
        }

        function exactInputSingle(
            ExactInputSingleParams memory params
        ) external payable returns (uint256 amountOut);

        function exactOutputSingle(
            ExactOutputSingleParams memory params
        ) external payable returns (uint256 amountIn);
    }

    proptest! {
        #[test]
        fn test_exact_input_single_encoding(
            token_in in any::<Address>(),
            token_out in any::<Address>(),
            recipient in any::<Address>(),
            deadline in any::<U>(),
            amount_in in any::<U>(),
            amount_out_minimum in any::<U>(),
            limit_sqrt_price in any::<[u8; 20]>()
        ) {
            let v = make_fn_exact_input_single(
                token_in,
                token_out,
                recipient,
                deadline,
                amount_in,
                amount_out_minimum,
                limit_sqrt_price
            )
                .to_vec();
            let exp = exactInputSingleCall { params: ExactInputSingleParams {
                tokenIn: AAddress::from(token_in),
                tokenOut: AAddress::from(token_out),
                recipient: AAddress::from(recipient),
                deadline: AU::from_be_bytes(*deadline),
                amountIn: AU::from_be_bytes(*amount_in),
                amountOutMinimum: AU::from_be_bytes(*amount_out_minimum),
                limitSqrtPrice: U160::from_be_bytes(limit_sqrt_price)
            } }.abi_encode();
            assert_eq!(exp, v, "{} != {}", const_hex::encode(exp.clone()), const_hex::encode(v.clone()));
        }

        #[test]
        fn test_exact_output_single_encoding(
            token_in in any::<Address>(),
            token_out in any::<Address>(),
            fee in any::<[u8; 3]>(),
            recipient in any::<Address>(),
            deadline in any::<U>(),
            amount_out in any::<U>(),
            amount_in_maximum in any::<U>(),
            limit_sqrt_price in any::<[u8; 20]>()
        ) {
            let v = make_fn_exact_output_single(
                token_in,
                token_out,
                fee,
                recipient,
                deadline,
                amount_out,
                amount_in_maximum,
                limit_sqrt_price
            )
                .to_vec();
            let exp = exactOutputSingleCall { params: ExactOutputSingleParams {
                tokenIn: AAddress::from(token_in),
                tokenOut: AAddress::from(token_out),
                fee: AU24::from_be_bytes(fee),
                recipient: AAddress::from(recipient),
                deadline: AU::from_be_bytes(*deadline),
                amountOut: AU::from_be_bytes(*amount_out),
                amountInMaximum: AU::from_be_bytes(*amount_in_maximum),
                limitSqrtPrice: U160::from_be_bytes(limit_sqrt_price)
            } }.abi_encode();
            assert_eq!(exp, v, "{} != {}", const_hex::encode(exp.clone()), const_hex::encode(v.clone()));
        }
    }
}

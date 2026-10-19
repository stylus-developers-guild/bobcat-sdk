//! Calldata builders for the Ramses V3 SwapRouter deployed on Robinhood Chain.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use bobcat_cd::{EvmCdAddress, EvmCdBytes, EvmCdI24, EvmCdSerialise, EvmCdU160};
use bobcat_maths::U;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactInputSingleParams {
    pub token_in: EvmCdAddress,
    pub token_out: EvmCdAddress,
    pub tick_spacing: EvmCdI24,
    pub recipient: EvmCdAddress,
    pub deadline: U,
    pub amount_in: U,
    pub amount_out_minimum: U,
    pub sqrt_price_limit_x96: EvmCdU160,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactOutputSingleParams {
    pub token_in: EvmCdAddress,
    pub token_out: EvmCdAddress,
    pub tick_spacing: EvmCdI24,
    pub recipient: EvmCdAddress,
    pub deadline: U,
    pub amount_out: U,
    pub amount_in_maximum: U,
    pub sqrt_price_limit_x96: EvmCdU160,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactInputParamsSlice<const PATH_LEN: usize> {
    pub path: [u8; PATH_LEN],
    pub recipient: EvmCdAddress,
    pub deadline: U,
    pub amount_in: U,
    pub amount_out_minimum: U,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactOutputParamsSlice<const PATH_LEN: usize> {
    /// Exact-output paths are encoded in reverse order by the Ramses router.
    pub path: [u8; PATH_LEN],
    pub recipient: EvmCdAddress,
    pub deadline: U,
    pub amount_out: U,
    pub amount_in_maximum: U,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactInputParamsVec {
    pub path: Vec<u8>,
    pub recipient: EvmCdAddress,
    pub deadline: U,
    pub amount_in: U,
    pub amount_out_minimum: U,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactOutputParamsVec {
    /// Exact-output paths are encoded in reverse order by the Ramses router.
    pub path: Vec<u8>,
    pub recipient: EvmCdAddress,
    pub deadline: U,
    pub amount_out: U,
    pub amount_in_maximum: U,
}

#[derive(EvmCdSerialise)]
struct DerivedExactInputParams<Path> {
    path: Path,
    recipient: EvmCdAddress,
    deadline: U,
    amount_in: U,
    amount_out_minimum: U,
}

#[derive(EvmCdSerialise)]
struct DerivedExactOutputParams<Path> {
    path: Path,
    recipient: EvmCdAddress,
    deadline: U,
    amount_out: U,
    amount_in_maximum: U,
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedSingleCall {
    #[evm_selector(
        "exactInputSingle((address,address,int24,address,uint256,uint256,uint256,uint160))"
    )]
    ExactInputSingle {
        token_in: EvmCdAddress,
        token_out: EvmCdAddress,
        tick_spacing: EvmCdI24,
        recipient: EvmCdAddress,
        deadline: U,
        amount_in: U,
        amount_out_minimum: U,
        sqrt_price_limit_x96: EvmCdU160,
    },
    #[evm_selector(
        "exactOutputSingle((address,address,int24,address,uint256,uint256,uint256,uint160))"
    )]
    ExactOutputSingle {
        token_in: EvmCdAddress,
        token_out: EvmCdAddress,
        tick_spacing: EvmCdI24,
        recipient: EvmCdAddress,
        deadline: U,
        amount_out: U,
        amount_in_maximum: U,
        sqrt_price_limit_x96: EvmCdU160,
    },
}

#[derive(EvmCdSerialise)]
#[evm_selector]
enum DerivedPathCall<Path> {
    #[evm_selector("exactInput((bytes,address,uint256,uint256,uint256))")]
    ExactInput(DerivedExactInputParams<Path>),
    #[evm_selector("exactOutput((bytes,address,uint256,uint256,uint256))")]
    ExactOutput(DerivedExactOutputParams<Path>),
}

const SINGLE_CALLDATA_LEN: usize = 4 + 32 * 8;
const PATH_CALLDATA_BASE_LEN: usize = 4 + 32 * 7;

const fn padded_len(length: usize) -> usize {
    let Some(padded) = length.checked_add(31) else {
        panic!("Ramses V3 path length overflows calldata length");
    };
    padded & !31
}

/// Encoded `exactInput`/`exactOutput` length for a packed path of `path_len` bytes.
pub const fn exact_path_calldata_len(path_len: usize) -> usize {
    let Some(length) = PATH_CALLDATA_BASE_LEN.checked_add(padded_len(path_len)) else {
        panic!("Ramses V3 path length overflows calldata length");
    };
    length
}

pub fn make_fn_exact_input_single(params: ExactInputSingleParams) -> [u8; SINGLE_CALLDATA_LEN] {
    DerivedSingleCall::ExactInputSingle {
        token_in: params.token_in,
        token_out: params.token_out,
        tick_spacing: params.tick_spacing,
        recipient: params.recipient,
        deadline: params.deadline,
        amount_in: params.amount_in,
        amount_out_minimum: params.amount_out_minimum,
        sqrt_price_limit_x96: params.sqrt_price_limit_x96,
    }
    .to_evm_array()
    .expect("Ramses V3 exactInputSingle calldata has a fixed 260-byte encoding")
}

pub fn make_fn_exact_output_single(params: ExactOutputSingleParams) -> [u8; SINGLE_CALLDATA_LEN] {
    DerivedSingleCall::ExactOutputSingle {
        token_in: params.token_in,
        token_out: params.token_out,
        tick_spacing: params.tick_spacing,
        recipient: params.recipient,
        deadline: params.deadline,
        amount_out: params.amount_out,
        amount_in_maximum: params.amount_in_maximum,
        sqrt_price_limit_x96: params.sqrt_price_limit_x96,
    }
    .to_evm_array()
    .expect("Ramses V3 exactOutputSingle calldata has a fixed 260-byte encoding")
}

pub fn make_fn_exact_input_slice<const PATH_LEN: usize, const ALL: usize>(
    params: ExactInputParamsSlice<PATH_LEN>,
) -> [u8; ALL] {
    assert_eq!(
        ALL,
        exact_path_calldata_len(PATH_LEN),
        "calldata output array has the wrong length"
    );
    DerivedPathCall::ExactInput(DerivedExactInputParams {
        path: EvmCdBytes::<PATH_LEN>::from(params.path),
        recipient: params.recipient,
        deadline: params.deadline,
        amount_in: params.amount_in,
        amount_out_minimum: params.amount_out_minimum,
    })
    .to_evm_array()
    .expect("Ramses V3 exactInput calldata serialization failed")
}

pub fn make_fn_exact_output_slice<const PATH_LEN: usize, const ALL: usize>(
    params: ExactOutputParamsSlice<PATH_LEN>,
) -> [u8; ALL] {
    assert_eq!(
        ALL,
        exact_path_calldata_len(PATH_LEN),
        "calldata output array has the wrong length"
    );
    DerivedPathCall::ExactOutput(DerivedExactOutputParams {
        path: EvmCdBytes::<PATH_LEN>::from(params.path),
        recipient: params.recipient,
        deadline: params.deadline,
        amount_out: params.amount_out,
        amount_in_maximum: params.amount_in_maximum,
    })
    .to_evm_array()
    .expect("Ramses V3 exactOutput calldata serialization failed")
}

#[cfg(feature = "alloc")]
pub fn make_fn_exact_input_vec(params: ExactInputParamsVec) -> Vec<u8> {
    exact_path_calldata_len(params.path.len());
    DerivedPathCall::ExactInput(DerivedExactInputParams {
        path: params.path,
        recipient: params.recipient,
        deadline: params.deadline,
        amount_in: params.amount_in,
        amount_out_minimum: params.amount_out_minimum,
    })
    .serialise_to_vec()
    .expect("Ramses V3 exactInput calldata serialization failed")
}

#[cfg(feature = "alloc")]
pub fn make_fn_exact_output_vec(params: ExactOutputParamsVec) -> Vec<u8> {
    exact_path_calldata_len(params.path.len());
    DerivedPathCall::ExactOutput(DerivedExactOutputParams {
        path: params.path,
        recipient: params.recipient,
        deadline: params.deadline,
        amount_out: params.amount_out,
        amount_in_maximum: params.amount_in_maximum,
    })
    .serialise_to_vec()
    .expect("Ramses V3 exactOutput calldata serialization failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{
        aliases::I24 as AlloyI24, Address as AlloyAddress, U160 as AlloyU160, U256,
    };
    use alloy_sol_macro::sol;
    use alloy_sol_types::SolCall;

    sol! {
        struct AlloyExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            int24 tickSpacing;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }

        struct AlloyExactOutputSingleParams {
            address tokenIn;
            address tokenOut;
            int24 tickSpacing;
            address recipient;
            uint256 deadline;
            uint256 amountOut;
            uint256 amountInMaximum;
            uint160 sqrtPriceLimitX96;
        }

        struct AlloyExactInputParams {
            bytes path;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
        }

        struct AlloyExactOutputParams {
            bytes path;
            address recipient;
            uint256 deadline;
            uint256 amountOut;
            uint256 amountInMaximum;
        }

        function exactInputSingle(AlloyExactInputSingleParams params) external payable returns (uint256 amountOut);
        function exactOutputSingle(AlloyExactOutputSingleParams params) external payable returns (uint256 amountIn);
        function exactInput(AlloyExactInputParams params) external payable returns (uint256 amountOut);
        function exactOutput(AlloyExactOutputParams params) external payable returns (uint256 amountIn);
    }

    fn word(value: u64) -> U {
        U::from_u64(value)
    }

    #[test]
    fn exact_input_single_matches_ramses_abi() {
        let params = ExactInputSingleParams {
            token_in: EvmCdAddress::new([0x11; 20]),
            token_out: EvmCdAddress::new([0x22; 20]),
            tick_spacing: EvmCdI24::new([0xff, 0xff, 0xc4]),
            recipient: EvmCdAddress::new([0x33; 20]),
            deadline: word(1),
            amount_in: word(2),
            amount_out_minimum: word(3),
            sqrt_price_limit_x96: EvmCdU160::new([0x44; 20]),
        };

        let actual = make_fn_exact_input_single(params);
        let expected = exactInputSingleCall {
            params: AlloyExactInputSingleParams {
                tokenIn: AlloyAddress::from(params.token_in.into_array()),
                tokenOut: AlloyAddress::from(params.token_out.into_array()),
                tickSpacing: AlloyI24::unchecked_from(-60),
                recipient: AlloyAddress::from(params.recipient.into_array()),
                deadline: U256::from_be_bytes(*params.deadline),
                amountIn: U256::from_be_bytes(*params.amount_in),
                amountOutMinimum: U256::from_be_bytes(*params.amount_out_minimum),
                sqrtPriceLimitX96: AlloyU160::from_be_bytes(
                    params.sqrt_price_limit_x96.into_array(),
                ),
            },
        }
        .abi_encode();

        assert_eq!(actual.as_slice(), expected);
    }

    #[test]
    fn exact_output_single_matches_ramses_abi() {
        let params = ExactOutputSingleParams {
            token_in: EvmCdAddress::new([0x11; 20]),
            token_out: EvmCdAddress::new([0x22; 20]),
            tick_spacing: EvmCdI24::new([0x00, 0x00, 0x01]),
            recipient: EvmCdAddress::new([0x33; 20]),
            deadline: word(4),
            amount_out: word(5),
            amount_in_maximum: word(6),
            sqrt_price_limit_x96: EvmCdU160::new([0x44; 20]),
        };

        let actual = make_fn_exact_output_single(params);
        let expected = exactOutputSingleCall {
            params: AlloyExactOutputSingleParams {
                tokenIn: AlloyAddress::from(params.token_in.into_array()),
                tokenOut: AlloyAddress::from(params.token_out.into_array()),
                tickSpacing: AlloyI24::unchecked_from(1),
                recipient: AlloyAddress::from(params.recipient.into_array()),
                deadline: U256::from_be_bytes(*params.deadline),
                amountOut: U256::from_be_bytes(*params.amount_out),
                amountInMaximum: U256::from_be_bytes(*params.amount_in_maximum),
                sqrtPriceLimitX96: AlloyU160::from_be_bytes(
                    params.sqrt_price_limit_x96.into_array(),
                ),
            },
        }
        .abi_encode();

        assert_eq!(actual.as_slice(), expected);
    }

    #[test]
    fn exact_input_path_matches_ramses_abi_without_allocating() {
        const PATH_LEN: usize = 43;
        const CALLDATA_LEN: usize = exact_path_calldata_len(PATH_LEN);
        let params = ExactInputParamsSlice {
            path: [0x55; PATH_LEN],
            recipient: EvmCdAddress::new([0x66; 20]),
            deadline: word(7),
            amount_in: word(8),
            amount_out_minimum: word(9),
        };

        let actual: [u8; CALLDATA_LEN] = make_fn_exact_input_slice(params);
        let expected = exactInputCall {
            params: AlloyExactInputParams {
                path: params.path.into(),
                recipient: AlloyAddress::from(params.recipient.into_array()),
                deadline: U256::from_be_bytes(*params.deadline),
                amountIn: U256::from_be_bytes(*params.amount_in),
                amountOutMinimum: U256::from_be_bytes(*params.amount_out_minimum),
            },
        }
        .abi_encode();

        assert_eq!(actual.as_slice(), expected);

        #[cfg(feature = "alloc")]
        {
            let actual_vec = make_fn_exact_input_vec(ExactInputParamsVec {
                path: params.path.to_vec(),
                recipient: params.recipient,
                deadline: params.deadline,
                amount_in: params.amount_in,
                amount_out_minimum: params.amount_out_minimum,
            });
            assert_eq!(actual_vec, expected);
        }
    }

    #[test]
    fn exact_output_path_matches_ramses_abi_without_allocating() {
        const PATH_LEN: usize = 44;
        const CALLDATA_LEN: usize = exact_path_calldata_len(PATH_LEN);
        let params = ExactOutputParamsSlice {
            path: [0x77; PATH_LEN],
            recipient: EvmCdAddress::new([0x88; 20]),
            deadline: word(10),
            amount_out: word(11),
            amount_in_maximum: word(12),
        };

        let actual: [u8; CALLDATA_LEN] = make_fn_exact_output_slice(params);
        let expected = exactOutputCall {
            params: AlloyExactOutputParams {
                path: params.path.into(),
                recipient: AlloyAddress::from(params.recipient.into_array()),
                deadline: U256::from_be_bytes(*params.deadline),
                amountOut: U256::from_be_bytes(*params.amount_out),
                amountInMaximum: U256::from_be_bytes(*params.amount_in_maximum),
            },
        }
        .abi_encode();

        assert_eq!(actual.as_slice(), expected);

        #[cfg(feature = "alloc")]
        {
            let actual_vec = make_fn_exact_output_vec(ExactOutputParamsVec {
                path: params.path.to_vec(),
                recipient: params.recipient,
                deadline: params.deadline,
                amount_out: params.amount_out,
                amount_in_maximum: params.amount_in_maximum,
            });
            assert_eq!(actual_vec, expected);
        }
    }
}

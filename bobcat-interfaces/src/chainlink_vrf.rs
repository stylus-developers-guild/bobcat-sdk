use bobcat_cd::{leftpad_u8, leftpad_u16, leftpad_u32, leftpad_usize};

use bobcat_maths::U;

use array_concat::concat_arrays;

use crate::selectors;

selectors! {
    SEL_ESTIMATE_REQUEST_PRICE_NATIVE = b"estimateRequestPriceNative(uint32,uint32,uint256)",
    SEL_REQUEST_RANDOM_WORDS_IN_NATIVE = b"requestRandomWordsInNative(uint32,uint16,uint32,bytes)",
    SEL_LINK = b"link()",
    SEL_LINK_NATIVE_FEED = b"linkNativeFeed()"
}

pub const fn make_fn_estimate_request_price_native(
    callback_gas_limit: u32,
    num_words: u32,
    request_gas_price_wei: U,
) -> [u8; 32 * 3 + 4] {
    concat_arrays!(
        SEL_ESTIMATE_REQUEST_PRICE_NATIVE,
        leftpad_u32(callback_gas_limit),
        leftpad_u32(num_words),
        request_gas_price_wei.0
    )
}

/// The native length of make_fn_request_words_in_native_slice (the
/// words, then the length of the arguments, and the offset).
pub const REQUEST_WORDS_NATIVE_SLICE_BASE: usize = 32 * 5 + 4;

/// Create the calculate request words in native function. ALL_LEN must be
/// 32 * 5 + BASE_LEN + 4 on stable Rust

/// Make a request words in native slice argument without using the
/// allocator. The base length must be the length of the array without any
/// padding, and the all length must be the entire allocation, inclusive
/// of any padding that's needed. To find padding needed, ((x + 31) & ~31) - x
pub const fn make_fn_request_words_in_native_slice<
    const BASE_LEN: usize,
    const PADDING: usize,
    const ALL_LEN: usize,
>(
    callback_gas_limit: u32,
    request_confirmations: u16,
    num_words: u32,
    extra_args: [u8; BASE_LEN],
) -> [u8; ALL_LEN] {
    // The following words are packed: [callback gas limit, request
    // confirmations, num words, extra args offset, extra args length,
    // extra args... + padding]
    assert!(
        ALL_LEN >= BASE_LEN + PADDING + REQUEST_WORDS_NATIVE_SLICE_BASE,
        "make_fn_request_words_in_native_slice inconsistent length"
    );
    assert!(
        BASE_LEN % 32 == 0 || PADDING + BASE_LEN == (BASE_LEN + 31) & !31,
        "padding inconsistent"
    );
    assert!(
        (ALL_LEN - 4) % 32 == 0,
        "length needs extra word to be % 32 = 0"
    );
    concat_arrays!(
        SEL_REQUEST_RANDOM_WORDS_IN_NATIVE,
        leftpad_u32(callback_gas_limit),
        leftpad_u16(request_confirmations),
        leftpad_u32(num_words),
        leftpad_u8(32 * 4),
        leftpad_usize(BASE_LEN),
        extra_args,
        [0u8; PADDING]
    )
}

pub fn make_fn_request_words_in_native_no_bytes(
    callback_gas_limit: u32,
    request_confirmations: u16,
    num_words: u32,
) -> [u8; REQUEST_WORDS_NATIVE_SLICE_BASE] {
    make_fn_request_words_in_native_slice::<0, 0, REQUEST_WORDS_NATIVE_SLICE_BASE>(
        callback_gas_limit,
        request_confirmations,
        num_words,
        [],
    )
}

pub const fn make_fn_link() -> [u8; 4] {
    SEL_LINK
}

pub const fn make_fn_link_native_feed() -> [u8; 4] {
    SEL_LINK_NATIVE_FEED
}

#[cfg(test)]
mod test {
    use super::*;

    use proptest::prelude::*;

    use alloy_sol_macro::sol;

    use alloy_sol_types::SolCall;

    use alloy_primitives::Bytes;

    sol! {
       function requestRandomWordsInNative(
           uint32 callbackGasLimit,
           uint16 requestConfirmations,
           uint32 numWords,
           bytes calldata extraArgs
       ) external payable returns (uint256 requestId);
    }

    proptest! {
        #[test]
        fn test_make_fn_request_words_in_native_no_bytes(
            callback_gas_limit in any::<u32>(),
            request_confirmations in any::<u16>(),
            num_words in any::<u32>()
        ) {
            let e = requestRandomWordsInNativeCall {
                callbackGasLimit: callback_gas_limit,
                requestConfirmations: request_confirmations,
                numWords: num_words,
                extraArgs: Bytes::new()
            }
            .abi_encode();
            let v = make_fn_request_words_in_native_no_bytes(
                callback_gas_limit,
                request_confirmations,
                num_words
            );
            assert_eq!(
              e,
              v,
              "{} != {}",
              const_hex::encode(&e),
              const_hex::encode(&v)
            );
        }

        #[test]
        fn test_make_fn_request_words_in_native_slice_tiny(
            callback_gas_limit in any::<u32>(),
            request_confirmations in any::<u16>(),
            num_words in any::<u32>(),
            extra_args in any::<[u8; 2]>()
        ) {
            let e = requestRandomWordsInNativeCall {
                callbackGasLimit: callback_gas_limit,
                requestConfirmations: request_confirmations,
                numWords: num_words,
                extraArgs: Bytes::copy_from_slice(&extra_args)
            }
            .abi_encode();
            let v = make_fn_request_words_in_native_slice::<
                2,
                30,
                { 32 + REQUEST_WORDS_NATIVE_SLICE_BASE },

            > (
                callback_gas_limit,
                request_confirmations,
                num_words,
                extra_args
            );
            assert_eq!(
              e,
              v,
              "{} != {}",
              const_hex::encode(&e),
              const_hex::encode(&v)
            );
        }

        #[test]
        fn test_make_fn_request_words_in_native_slice_even(
            callback_gas_limit in any::<u32>(),
            request_confirmations in any::<u16>(),
            num_words in any::<u32>(),
            extra_args in any::<[u8; 32]>()
        ) {
            // Simulate packing when the word length is even. There should be a
            // runtime check to make sure that the length is correct to include an
            // extra word if it's needed.
            let e = requestRandomWordsInNativeCall {
                callbackGasLimit: callback_gas_limit,
                requestConfirmations: request_confirmations,
                numWords: num_words,
                extraArgs: Bytes::copy_from_slice(&extra_args)
            }
            .abi_encode();
            let v = make_fn_request_words_in_native_slice::<
                32,
                0,
                { 32 + REQUEST_WORDS_NATIVE_SLICE_BASE },

            > (
                callback_gas_limit,
                request_confirmations,
                num_words,
                extra_args
            );
            assert_eq!(
              e,
              v,
              "{} != {}",
              const_hex::encode(&e),
              const_hex::encode(&v)
            );
        }

        #[test]
        fn test_make_fn_request_words_in_native_slice_odd(
            callback_gas_limit in any::<u32>(),
            request_confirmations in any::<u16>(),
            num_words in any::<u32>(),
            extra_args in any::<[u8; 100]>()
        ) {
            // Simulate packing when the word length is odd + word to pad out.
            let e = requestRandomWordsInNativeCall {
                callbackGasLimit: callback_gas_limit,
                requestConfirmations: request_confirmations,
                numWords: num_words,
                extraArgs: Bytes::copy_from_slice(&extra_args)
            }
            .abi_encode();
            let v = make_fn_request_words_in_native_slice::<
                100,
                28,
                { 100 + 28 + REQUEST_WORDS_NATIVE_SLICE_BASE },
            > (
                callback_gas_limit,
                request_confirmations,
                num_words,
                extra_args
            );
            assert_eq!(
              e,
              v,
              "{} != {}",
              const_hex::encode(&e),
              const_hex::encode(&v)
            );
        }

        #[test]
        fn test_make_fn_request_words_in_native_slice_huge(
            callback_gas_limit in any::<u32>(),
            request_confirmations in any::<u16>(),
            num_words in any::<u32>(),
            extra_args in any::<[u8; 1000]>()
        ) {
            // Simulate packing when the word length is odd + word to pad out.
            let e = requestRandomWordsInNativeCall {
                callbackGasLimit: callback_gas_limit,
                requestConfirmations: request_confirmations,
                numWords: num_words,
                extraArgs: Bytes::copy_from_slice(&extra_args)
            }
            .abi_encode();
            //(1000 + 31) & ~31
            let v = make_fn_request_words_in_native_slice::<
                1000,
                24,
                { 1000 + 24 + REQUEST_WORDS_NATIVE_SLICE_BASE },
            > (
                callback_gas_limit,
                request_confirmations,
                num_words,
                extra_args
            );
            assert_eq!(
              e,
              v,
              "{} != {}",
              const_hex::encode(&e),
              const_hex::encode(&v)
            );
        }
    }
}

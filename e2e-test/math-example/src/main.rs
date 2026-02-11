#![no_main]
#![no_std]

use bobcat_maths_zone::{bobcat_math, maths_zone};
use bobcat_sdk::{
    cd::{const_keccak_sel, read_words},
    entry::*,
    prelude::*,
};

bobcat_allocator!();

const SEL_GET_CONSTANT: [u8; 4] = const_keccak_sel(b"getConstant()");
const SEL_ADD: [u8; 4] = const_keccak_sel(b"add(uint256,uint256)");
const SEL_MUL: [u8; 4] = const_keccak_sel(b"mul(uint256,uint256)");

/// Compile-time constant: 2**8 + 5 = 261
const CONSTANT: U = maths_zone!("2**8 + 5");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = read_args_safe!(args_len, { (32 * 2) + 4 });
    let sel: [u8; 4] = args[..4].try_into().unwrap();

    match sel {
        SEL_GET_CONSTANT => {
            write_result_word(&CONSTANT);
        }
        SEL_ADD => {
            let (a, b) = read_words!(&args[4..], 2);
            let result = bobcat_math!(
                r#"
                sum = a + b
                return sum
            "#
            )
            .unwrap();
            write_result_word(&result);
        }
        SEL_MUL => {
            let (a, b) = read_words!(&args[4..], 2);
            let result = bobcat_math!(
                r#"
                product = a * b
                return product
            "#
            )
            .unwrap();
            write_result_word(&result);
        }
        _ => unimplemented!(),
    }
    0
}

#![no_main]
#![no_std]

use bobcat_sdk::{
    cd::{const_keccak_sel, read_words},
    entry::*,
    prelude::*,
};

bobcat_allocator!();

const SEL_UNISWAP: [u8; 4] = const_keccak_sel(b"uniswap(uint256,uint256,uint256)");
const SEL_RUINT: [u8; 4] = const_keccak_sel(b"ruint(uint256,uint256,uint256)");
const SEL_WIDENING: [u8; 4] = const_keccak_sel(b"widening(uint256,uint256,uint256)");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { (32 * 3) + 4 });
    let (x, y, z) = read_words!(&args[4..], 3);
    write_result_word(&match args[..4].try_into().unwrap() {
        SEL_UNISWAP => x.mul_div(&y, *z).unwrap_or_default().0,
        SEL_RUINT => x.ruint_mul_div(&y, *z).unwrap_or_default().0,
        SEL_WIDENING => x.widening_mul_div(&y, *z).unwrap_or_default().0,
        _ => unimplemented!("{:?}", &args[..4]),
    });
    0
}

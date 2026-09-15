#![no_main]
#![no_std]

use bobcat_sdk::{
    cd::{EvmCdDeserialise, EvmCdSerialise},
    entry::*,
    maths::U,
};

#[derive(Debug, Clone, EvmCdSerialise, EvmCdDeserialise)]
#[evm_entrypoint]
pub enum Entry {
    Hello(U, U),
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    match read_cd::<Entry>(args_len) {
        Entry::Hello(x, y) => {
            write_result_slice(&x.mul_div(&y, U::from(100u32)).unwrap().0.0);
        }
    }
    0
}

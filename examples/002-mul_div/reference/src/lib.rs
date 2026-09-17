#![no_main]
#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};

use stylus_sdk::{alloy_primitives::U256, prelude::*};

#[storage]
#[entrypoint]
struct Storage;

// This is the function that we use in 9lives:
fn r_mul_div(a: U256, b: U256, mut denom_and_rem: U256) -> Option<(U256, U256)> {
    if denom_and_rem == U256::ZERO {
        return None;
    }
    let mut mul_and_quo = a.widening_mul::<256, 4, 512, 8>(b);
    unsafe {
        stylus_sdk::alloy_primitives::ruint::algorithms::div(
            mul_and_quo.as_limbs_mut(),
            denom_and_rem.as_limbs_mut(),
        );
    }
    let limbs = mul_and_quo.into_limbs();
    if limbs[4..] != [0_u64; 4] {
        return None;
    }
    Some((U256::from_limbs_slice(&limbs[0..4]), denom_and_rem))
}

#[public]
impl Storage {
    pub fn hello(x: U256, y: U256) -> U256 {
        r_mul_div(x, y, U256::from(100u32)).unwrap().0
    }
}

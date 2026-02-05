#![no_main]

use libfuzzer_sys::{
    arbitrary::{self, Arbitrary},
    fuzz_target,
};

use bobcat_maths::U;

use ruint::aliases::U256;

#[derive(Arbitrary, Debug)]
struct Sub {
    x: U,
    y: U,
}

macro_rules! assert_eq_t {
    ($e:expr, $x:expr, $($o:expr),*) => {
        assert_eq!($e.to_be_bytes::<32>(), $x.0, $($o),*)
    };
}

fuzz_target!(|data: Sub| {
    let ex = U256::from_be_bytes(data.x.0);
    let ey = U256::from_be_bytes(data.y.0);
    let Sub { x, y } = data;
    assert_eq_t!(ex.wrapping_sub(ey), x.wrapping_sub(&y),);
    match (ex.checked_sub(ey), bobcat_maths::checked_sub_opt(&x, &y)) {
        (None, None) => (),
        (Some(x), Some(y)) => {
            assert_eq_t!(x, y, "{x} != {y} ({}, {})", data.x, data.y)
        }
        (x, y) => panic!("bad checked, {x:?} != {y:?}"),
    }
    assert_eq_t!(ex.saturating_sub(ey), x.saturating_sub(&y), );
});

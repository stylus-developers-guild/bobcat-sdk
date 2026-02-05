#![no_main]

use libfuzzer_sys::{
    arbitrary::{self, Arbitrary},
    fuzz_target,
};

use bobcat_maths::U;

use ruint::aliases::U256;

#[derive(Arbitrary, Debug)]
struct Add {
    x: U,
    y: U,
}

macro_rules! assert_eq_t {
    ($e:expr, $x:expr, $($o:expr),*) => {
        assert_eq!($e.to_be_bytes::<32>(), $x.0, $($o),*)
    };
}

fuzz_target!(|data: Add| {
    let ex = U256::from_be_bytes(data.x.0);
    let ey = U256::from_be_bytes(data.y.0);
    let Add { x, y } = data;
    assert_eq_t!(ex.wrapping_add(ey), x.wrapping_add(&y),);
    match (ex.checked_add(ey), bobcat_maths::checked_add_opt(&x, &y)) {
        (None, None) => (),
        (Some(x), Some(y)) => {
            assert_eq_t!(x, y, "{x} != {y} ({}, {})", data.x, data.y)
        }
        (x, y) => panic!("bad checked, {x:?} != {y:?}"),
    }
    assert_eq_t!(ex.saturating_add(ey), x.saturating_add(&y), );
});

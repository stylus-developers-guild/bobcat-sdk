#![no_main]

use ruint::aliases::U256;

use bobcat_maths::U;

use libfuzzer_sys::{arbitrary, fuzz_target};

use arbitrary::Arbitrary;

// Though the rooti implementation could feature any combination of U, we
// do it this way to reduce the search space of the inputs that
// will likely take place in practice. We don't check so much whether the code
// here is blowing out when it shouldn't.
#[derive(Arbitrary, Debug)]
struct Root {
    x: U,
    y: u32,
}

// We also seek to see if the result is the same as the approximation
// method in the 9lives repo.
fn ninelives_root(x: U256, n: u32) -> Option<U256> {
    if n == 0 {
        return None;
    }
    if x.is_zero() {
        return Some(U256::ZERO);
    }
    if n == 1 {
        return Some(x);
    }
    if x == U256::from(4) && n == 2 {
        return Some(U256::from(2));
    }
    let n_u256 = U256::from(n);
    let n_1 = n_u256 - U256::from(1);
    let mut b = 0;
    let mut t = x;
    while t != U256::ZERO {
        b += 1;
        t >>= 1;
    }
    let shift = (b + n as usize - 1) / n as usize;
    let mut z = U256::from(1) << shift;
    let mut y = x;
    while z < y {
        y = z;
        let p = z.checked_pow(n_1)?;
        z = ((x / p) + (z * n_1)) / n_u256;
    }
    if y.checked_pow(n_u256)? > x {
        y -= U256::from(1);
    }
    Some(y)
}

fuzz_target!(|t: Root| {
    let Root { x, y } = t;
    let Some(r) = x.checked_rooti(y) else {
        return;
    };
    if y == 0 {
        assert_eq!(U::ZERO, r);
    }
    assert_eq!(
        U::from(
            U256::from_be_slice(&x.0)
                .root(y as usize)
                .to_be_bytes::<32>()
        ),
        r
    );
    assert_eq!(ninelives_root(U256::from_be_bytes(x.0), y), r);
});

use crate::U;

use proptest::{prelude::*, strategy::Strategy};

/// Simple strategy that generates values up to a million.
pub fn strat_tiny_u256() -> impl proptest::prelude::Strategy<Value = U> {
    (0..1_000_000u32).prop_map(U::from)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Uintsize {
    Small,
    Medium,
    Large,
}

pub fn strat_u(u: Uintsize) -> impl proptest::prelude::Strategy<Value = U> {
    // This has a 33% chance of filling out a third of the lower bits, which,
    // in our interpretation, is decoded as big endian in the next function,
    // so the right side, a 33% chance of two thirds, and a 33% chance of
    // everything is potentially filled out.
    (0..3).prop_perturb(move |s, mut rng| {
        let mut x = U::ZERO;
        let q = 32 / 3;
        if s == 2 && u == Uintsize::Large {
            for i in q * 2..32 {
                x[32 - i - 1] = rng.random();
            }
        }
        if s >= 1 && u != Uintsize::Small {
            for i in q..q * 2 {
                x[32 - i - 1] = rng.random();
            }
        }
        for i in 0..q {
            x[32 - i - 1] = rng.random();
        }
        x
    })
}

pub fn strat_small_u() -> impl Strategy<Value = U> {
    strat_u(Uintsize::Small)
}

pub fn strat_medium_u() -> impl Strategy<Value = U> {
    strat_u(Uintsize::Medium)
}

pub fn strat_large_u() -> impl Strategy<Value = U> {
    strat_u(Uintsize::Large)
}

pub fn strat_addr_not_empty() -> impl Strategy<Value = [u8; 20]> {
    any::<[u8; 20]>().prop_filter("address must be non-zero", |addr| {
        addr.iter().any(|&byte| byte != 0)
    })
}

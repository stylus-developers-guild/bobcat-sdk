#![cfg_attr(not(feature = "std"), no_std)]

use keccak_const::Keccak256;

use array_concat::concat_arrays;

pub use bobcat_maths::U;

use bobcat_maths::wrapping_sub;

pub use bobcat_host as host;

pub use bobcat_maths as maths;

macro_rules! storage_ops {
    ($($prefix:ident),* $(,)?) => {
        $(
            paste::paste! {
                pub fn [<$prefix _load>](x: &U) -> U {
                    let mut b = [0u8; 32];
                    unsafe { $crate::host::[<$prefix _load_bytes32>](x.as_ptr(), b.as_mut_ptr()) }
                    U(b)
                }

                pub fn [<$prefix _load_bool>](x: &U) -> bool {
                    [<$prefix _load>](x).into()
                }

                /// Attempt to "exchange" a value, returning whether the expected value was set.
                pub fn [<$prefix _exchange>](k: &U, exp: &U, new: &U) -> bool {
                    let t = [<$prefix _load>](k);
                    if &t != exp {
                        return false;
                    }
                    $crate::[<$prefix _store>](k, new);
                    true
                }

                /// Attempt to exchange the value, returning what was set before if it
                /// doesn't match the expected value.
                pub fn [<$prefix _exchange_res>](k: &U, exp: &U, new: &U) -> Result<(), U> {
                    let t = [<$prefix _load>](k);
                    if &t != exp {
                        return Err(t);
                    }
                    [<$prefix _store>](k, new);
                    Ok(())
                }
            }
        )*
    };
}

pub fn storage_store(x: &U, y: &U) {
    unsafe { host::storage_cache_bytes32(x.as_ptr(), y.as_ptr()) }
}

pub fn storage_store_bool(x: &U, y: bool) {
    storage_store(x, &U::from(y))
}

pub fn transient_store(x: &U, y: &U) {
    unsafe { host::transient_store_bytes32(x.as_ptr(), y.as_ptr()) }
}

pub fn transient_store_bool(x: &U, y: bool) {
    transient_store(x, &U::from(y))
}

pub fn flush_cache() {
    unsafe { host::storage_flush_cache(false) }
}

/// Boring flush guard function that runs `flush_cache` once the thunk
/// has run.
pub fn flush_guard<R, F: FnOnce() -> R>(f: F) -> R {
    let r = f();
    flush_cache();
    r
}

/// Version of the flush_guard function that cleans the storage state
/// before running using some host functions if it can. If it can't (for
/// example, it runs on wasm32, then it's the same as `flush_guard`.
/// Useful for testing without thinking much about it if the program
/// fails so there's no pollution of the storage and transient storage space.
pub fn flush_guard_fresh<R, F: FnOnce() -> R>(f: F) -> R {
    #[cfg(all(
        feature = "std",
        not(any(
            all(target_family = "wasm", target_os = "unknown"),
            all(target_arch = "riscv32", target_os = "none")
        ))
    ))]
    host::storage_reset();
    let r = f();
    flush_cache();
    r
}

storage_ops!(storage, transient);

macro_rules! storage_mutate_ops {
    ($prefix:ident, $($op:expr),* $(,)?) => {
        $(
            paste::paste! {
                pub fn [<$prefix _wrapping_ $op>](x: &U, new: &U) {
                    [<$prefix _store>](x, &maths::[<wrapping_ $op>](&[<$prefix _load>](x), new))
                }

                pub fn [<$prefix _saturating_ $op>](x: &U, new: &U) {
                    [<$prefix _store>](x, &maths::[<saturating_ $op>](&[<$prefix _load>](x), new))
                }

                pub fn [<$prefix _checked_ $op>](x: &U, new: &U) -> Option<()> {
                    let y = [<$prefix _load>](x);
                    let v = maths::[<checked_ $op>](&y, new);
                    [<$prefix _store>](x, &v);
                    Some(())
                }

                pub fn [<$prefix _checked_ $op _res>](x: &U, new: &U) -> U {
                    let y = [<$prefix _load>](x);
                    let v = maths::[<checked_ $op>](&y, new);
                    [<$prefix _store>](x, &v);
                    v
                }
            }
        )*
    };
}

storage_mutate_ops!(storage, add, sub, mul, div);
storage_mutate_ops!(transient, add, sub, mul, div);

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn slot_map_slot(k: &U, p: &U) -> U {
    const_slot_map(k, p)
}

pub fn reentrancy_guard_entry(x: &U) {
    assert!(x.len() <= 32, "too large");
    assert!(transient_exchange(x, &U::ZERO, &U::ONE), "reentrancy alarm")
}

pub fn reentrancy_guard_exit(x: &U) {
    assert!(x.len() <= 32, "too large");
    transient_store(x, &U::ZERO);
}

pub fn reentrancy_guard<R>(k: &U, f: impl FnOnce() -> R) -> R {
    reentrancy_guard_entry(k);
    let v = f();
    reentrancy_guard_exit(k);
    v
}

pub fn reentrancy_guard_sel<R>(k: &[u8; 4], f: impl FnOnce() -> R) -> R {
    reentrancy_guard::<R>(&U::from(k), f)
}

/// Compute the slot for a slice, and take it off the curve. Useful for
/// storage slot accesses (and more).
pub const fn const_slot_off_curve(b: &[u8]) -> U {
    wrapping_sub(&const_keccak256(b), &U::ONE)
}

pub fn slot_off_curve(b: &[u8]) -> U {
    // This won't result in 0 from the keccak, so we can use checked_sub to
    // use the code the host gives us for a slightly lower codesize profile.
    maths::checked_sub_opt(&keccak256(b), &U::ONE).unwrap()
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn keccak256(b: &[u8]) -> U {
    let mut out = [0u8; 32];
    unsafe {
        host::native_keccak256(b.as_ptr(), b.len(), out.as_mut_ptr());
    }
    U(out)
}

pub const fn const_keccak256(b: &[u8]) -> U {
    U(Keccak256::new().update(b).finalize())
}

pub const fn const_keccak256_two(x: &[u8], y: &[u8]) -> U {
    U(Keccak256::new().update(x).update(y).finalize())
}

pub const fn const_keccak256_two_off_curve(x: &[u8], y: &[u8]) -> U {
    wrapping_sub(&const_keccak256_two(x, y), &U::ONE)
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn keccak256(b: &[u8]) -> U {
    const_keccak256(b)
}

pub fn reentrancy_guard_const_keccak<R>(k: &[u8], f: impl FnOnce() -> R) -> R {
    reentrancy_guard(&const_keccak256(k), f)
}

pub fn reentrancy_guard_keccak<R>(k: &[u8], f: impl FnOnce() -> R) -> R {
    reentrancy_guard(&keccak256(k), f)
}

/// Find the storage map slot using keccak_const. Don't do this during
/// your runtime code, unless you want to pay the codesize price.
/// Make sure to reverse your arguments if you're shooting for EVM
/// storage equivalence. Same as slot_map.
pub const fn const_slot_map(x: &U, y: &U) -> U {
    let a: [u8; 32 * 2] = concat_arrays!(x.0, y.0);
    const_keccak256(&a)
}

/// Find the slot map item given by keccak256(k . p) with padding.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn slot_map(x: &U, y: &U) -> U {
    let a: [u8; 32 * 2] = concat_arrays!(x.0, y.0);
    keccak256(&a)
}

/// Find the slot map item given by keccak256(k . p) with padding.
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn slot_map(x: &U, y: &U) -> U {
    const_slot_map(x, y)
}

#[test]
fn test_slot_edd25519_count() {
    assert_eq!(
        U::from(
            const_hex::const_decode_to_array::<32>(
                b"709318ac04e7c3155ef66c30be7220b3243d7e2378fa4153b5f14ebd3ea771ab"
            )
            .unwrap()
        ),
        const_slot_off_curve(b"superposition.passport.ed25519_count")
    );
}

#[cfg(all(feature = "std", test))]
mod test {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_reentrancy_guard(x in any::<[u8; 8]>()) {
            reentrancy_guard(&U::from(x), || {
                assert!(transient_load(&U::from(x)).is_true());
            });
            assert!(transient_load(&U::from(x)).is_zero());
        }

        #[test]
        fn test_reentrancy_guard_bad(x in any::<[u8; 8]>()) {
             let x = U::from(x);
             transient_store(&x, &U::from(false));
             assert!(transient_exchange_bool(&x, true));
             assert!(!transient_exchange_bool(&x, true));
            assert!(transient_load(&x).is_some());
        }

        #[test]
        fn test_reentrancy_guard_sel(x in any::<[u8; 4]>()) {
            reentrancy_guard_sel(&x, || {
                assert!(transient_load(&U::from(x)).is_true());
            });
            assert!(transient_load(&U::from(x)).is_zero());
        }
    }
}

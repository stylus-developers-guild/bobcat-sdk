#![cfg_attr(not(feature = "std"), no_std)]

use keccak_const::Keccak256;

use array_concat::concat_arrays;

pub use bobcat_maths::U;

use bobcat_maths::wrapping_sub;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
    fn storage_load_bytes32(key: *const u8, out: *mut u8);
    fn storage_cache_bytes32(key: *const u8, from: *const u8);
    fn transient_load_bytes32(key: *const u8, dest: *mut u8);
    fn transient_store_bytes32(key: *const u8, value: *const u8);
    fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8);
    pub fn storage_flush_cache(clear: bool);
}

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    not(feature = "mutex"),
    feature = "std"
))]
pub mod storage_host {
    use super::*;

    use std::{cell::RefCell, collections::HashMap, ptr::copy_nonoverlapping};

    type WordHashMap = HashMap<U, U>;

    thread_local! {
        pub static STORAGE: RefCell<WordHashMap> = RefCell::default();
        pub static TRANSIENT: RefCell<WordHashMap> = RefCell::default();
    }

    pub fn storage_clear() {
        STORAGE.with(|s| s.borrow_mut().clear())
    }

    pub fn transient_clear() {
        TRANSIENT.with(|s| s.borrow_mut().clear())
    }

    unsafe fn read_word(key: *const u8) -> U {
        let mut r = [0u8; 32];
        unsafe {
            copy_nonoverlapping(key, r.as_mut_ptr(), 32);
        }
        U(r)
    }

    unsafe fn write_word(key: *mut u8, val: U) {
        unsafe {
            copy_nonoverlapping(val.as_ptr(), key, 32);
        }
    }

    pub(crate) unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = STORAGE.with(|s| match s.borrow().get(&k) {
            Some(v) => *v,
            None => U::ZERO,
        });
        unsafe { write_word(out, value) };
    }

    pub(crate) unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        STORAGE.with(|s| s.borrow_mut().insert(k, v));
    }

    pub(crate) unsafe fn transient_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = TRANSIENT.with(|s| match s.borrow().get(&k) {
            Some(v) => *v,
            None => U::ZERO,
        });
        unsafe { write_word(out, value) };
    }

    pub(crate) unsafe fn transient_store_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        TRANSIENT.with(|s| s.borrow_mut().insert(k, v));
    }

    pub unsafe fn storage_flush_cache(clear: bool) {
        if clear {
            storage_clear()
        }
    }
}

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    feature = "mutex",
    feature = "std"
))]
pub mod storage_host {
    use super::*;

    use std::{
        collections::HashMap,
        ptr::copy_nonoverlapping,
        sync::{LazyLock, Mutex},
    };

    type WordHashMap = HashMap<U, U>;

    pub static STORAGE: LazyLock<Mutex<WordHashMap>> = LazyLock::new(|| Mutex::default());
    pub static TRANSIENT: LazyLock<Mutex<WordHashMap>> = LazyLock::new(|| Mutex::default());

    pub fn storage_clear() {
        STORAGE.lock().unwrap().clear()
    }

    pub fn transient_clear() {
        TRANSIENT.lock().unwrap().clear()
    }

    unsafe fn read_word(key: *const u8) -> U {
        let mut r = [0u8; 32];
        unsafe {
            copy_nonoverlapping(key, r.as_mut_ptr(), 32);
        }
        U(r)
    }

    unsafe fn write_word(key: *mut u8, val: U) {
        unsafe {
            copy_nonoverlapping(val.as_ptr(), key, 32);
        }
    }

    pub(crate) unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = match STORAGE.lock().unwrap().get(&k) {
            Some(v) => *v,
            None => U::ZERO,
        };
        unsafe { write_word(out, value) };
    }

    pub(crate) unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        STORAGE.lock().unwrap().insert(k, v);
    }

    pub(crate) unsafe fn transient_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = match TRANSIENT.lock().unwrap().get(&k) {
            Some(v) => *v,
            None => U::ZERO,
        };
        unsafe { write_word(out, value) };
    }

    pub(crate) unsafe fn transient_store_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        TRANSIENT.lock().unwrap().insert(k, v);
    }

    pub unsafe fn storage_flush_cache(clear: bool) {
        if clear {
            storage_clear()
        }
    }
}

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    not(feature = "std")
))]
mod storage_host {
    pub(crate) unsafe fn storage_load_bytes32(_: *const u8, _: *mut u8) {}

    pub(crate) unsafe fn storage_cache_bytes32(_: *const u8, _: *const u8) {}

    pub(crate) unsafe fn transient_load_bytes32(_: *const u8, _: *mut u8) {}

    pub(crate) unsafe fn transient_store_bytes32(_: *const u8, _: *const u8) {}

    pub unsafe fn storage_flush_cache(_: bool) {}
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use storage_host::*;

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub use storage_host::storage_flush_cache;

macro_rules! storage_ops {
    ($($prefix:ident),* $(,)?) => {
        $(
            paste::paste! {
                pub fn [<$prefix _load>](x: &U) -> U {
                    let mut b = [0u8; 32];
                    unsafe { [<$prefix _load_bytes32>](x.as_ptr(), b.as_mut_ptr()) }
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
                    [<$prefix _store>](k, new);
                    true
                }

                pub fn [<$prefix _exchange_res>](k: &U, exp: &U, new: &U) -> Result<(), U> {
                    let t = [<$prefix _load>](k);
                    if &t != exp {
                        return Err(t);
                    }
                    [<$prefix _store>](k, new);
                    Ok(())
                }

                /// Set the value given, checking that the value passed has the inverse
                /// set set currently. So, passing true would check if false is set.
                pub fn [<$prefix _exchange_bool>](k: &U, new: bool) -> bool {
                    [<$prefix _exchange>](k, &U::from(!new), &U::from(new))
                }

                pub fn [<$prefix _exchange_bool_res>](k: &U, new: bool) -> Result<(), bool> {
                   let x = [<$prefix _exchange_bool>](k, new);
                   if x == !new {
                       Ok(())
                   } else {
                       Err(x)
                   }
                }
            }
        )*
    };
}

pub fn storage_store(x: &U, y: &U) {
    unsafe { storage_cache_bytes32(x.as_ptr(), y.as_ptr()) }
}

pub fn storage_store_bool(x: &U, y: bool) {
    storage_store(x, &U::from(y))
}

pub fn transient_store(x: &U, y: &U) {
    unsafe { transient_store_bytes32(x.as_ptr(), y.as_ptr()) }
}

pub fn transient_store_bool(x: &U, y: bool) {
    transient_store(x, &U::from(y))
}

pub fn flush_cache() {
    unsafe { storage_flush_cache(false) }
}

pub fn flush_guard<R, F: FnOnce() -> R>(f: F) -> R {
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
                    [<$prefix _store>](x, &bobcat_maths::[<wrapping_ $op>](&[<$prefix _load>](x), new))
                }

                pub fn [<$prefix _saturating_ $op>](x: &U, new: &U) {
                    [<$prefix _store>](x, &bobcat_maths::[<saturating_ $op>](&[<$prefix _load>](x), new))
                }

                pub fn [<$prefix _checked_ $op>](x: &U, new: &U) -> Option<()> {
                    let y = [<$prefix _load>](x);
                    let v = bobcat_maths::[<checked_ $op>](&y, new);
                    [<$prefix _store>](x, &v);
                    Some(())
                }

                pub fn [<$prefix _checked_ $op _res>](x: &U, new: &U) -> U {
                    let y = [<$prefix _load>](x);
                    let v = bobcat_maths::[<checked_ $op>](&y, new);
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
    assert!(transient_exchange_bool(x, true), "reentrancy alarm")
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
    bobcat_maths::checked_sub_opt(&keccak256(b), &U::ONE).unwrap()
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn keccak256(b: &[u8]) -> U {
    let mut out = [0u8; 32];
    unsafe {
        native_keccak256(b.as_ptr(), b.len(), out.as_mut_ptr());
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
pub const fn const_slot_map(k: &U, p: &U) -> U {
    let a: [u8; 32 * 2] = concat_arrays!(k.0, p.0);
    const_keccak256(&a)
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn slot_map(k: &U, p: &U) -> U {
    let b: [u8; 32 * 2] = concat_arrays!(k.0, p.0);
    keccak256(&b)
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn slot_map(k: &U, p: &U) -> U {
    const_slot_map(k, p)
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

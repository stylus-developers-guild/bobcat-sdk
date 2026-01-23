#[cfg(not(feature = "mutex"))]
mod impls {
    use std::{cell::RefCell, collections::HashMap, ptr::copy_nonoverlapping};

    type WordHashMap = HashMap<[u8; 32], [u8; 32]>;

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

    unsafe fn read_word(key: *const u8) -> [u8; 32] {
        let mut r = [0u8; 32];
        unsafe {
            copy_nonoverlapping(key, r.as_mut_ptr(), 32);
        }
        r
    }

    unsafe fn write_word(key: *mut u8, val: [u8; 32]) {
        unsafe {
            copy_nonoverlapping(val.as_ptr(), key, 32);
        }
    }

    pub unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = STORAGE.with(|s| match s.borrow().get(&k) {
            Some(v) => *v,
            None => [0u8; 32],
        });
        unsafe { write_word(out, value) };
    }

    pub unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        STORAGE.with(|s| s.borrow_mut().insert(k, v));
    }

    pub unsafe fn transient_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = TRANSIENT.with(|s| match s.borrow().get(&k) {
            Some(v) => *v,
            None => [0u8; 32],
        });
        unsafe { write_word(out, value) };
    }

    pub unsafe fn transient_store_bytes32(key: *const u8, value: *const u8) {
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

#[cfg(feature = "mutex")]
mod impls {
    use super::*;

    use std::{
        collections::HashMap,
        ptr::copy_nonoverlapping,
        sync::{LazyLock, Mutex},
    };

    type WordHashMap = HashMap<[u8; 32], [u8; 32]>;

    pub static STORAGE: LazyLock<Mutex<WordHashMap>> = LazyLock::new(|| Mutex::default());
    pub static TRANSIENT: LazyLock<Mutex<WordHashMap>> = LazyLock::new(|| Mutex::default());

    pub fn storage_clear() {
        STORAGE.lock().unwrap().clear()
    }

    pub fn transient_clear() {
        TRANSIENT.lock().unwrap().clear()
    }

    unsafe fn read_word(key: *const u8) -> [u8; 32] {
        let mut r = [0u8; 32];
        unsafe {
            copy_nonoverlapping(key, r.as_mut_ptr(), 32);
        }
        [u8; 32](r)
    }

    unsafe fn write_word(key: *mut u8, val: [u8; 32]) {
        unsafe {
            copy_nonoverlapping(val.as_ptr(), key, 32);
        }
    }

    pub unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = match STORAGE.lock().unwrap().get(&k) {
            Some(v) => *v,
            None => [0u8; 32],
        };
        unsafe { write_word(out, value) };
    }

    pub unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        STORAGE.lock().unwrap().insert(k, v);
    }

    pub unsafe fn transient_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = match TRANSIENT.lock().unwrap().get(&k) {
            Some(v) => *v,
            None => [0u8; 32],
        };
        unsafe { write_word(out, value) };
    }

    pub unsafe fn transient_store_bytes32(key: *const u8, value: *const u8) {
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

pub use impls::*;

pub fn log_txt(_: *const u8, _: usize) {}

pub unsafe fn call_contract(
    _contract: *const u8,
    _calldata: *const u8,
    _calldata_len: usize,
    _value: *const u8,
    _gas: u64,
    _return_data_len: *mut usize,
) -> u8 {
    0
}

pub unsafe fn static_call_contract(
    _contract: *const u8,
    _calldata: *const u8,
    _calldata_len: usize,
    _gas: u64,
    _return_data_len: *mut usize,
) -> u8 {
    0
}

pub unsafe fn delegate_call_contract(
    _contract: *const u8,
    _calldata: *const u8,
    _calldata_len: usize,
    _gas: u64,
    _return_data_len: *mut usize,
) -> u8 {
    0
}

pub unsafe fn read_return_data(_: *mut u8, _: usize, _: usize) -> usize {
    0
}

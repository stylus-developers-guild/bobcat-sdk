#[cfg(not(feature = "mutex"))]
mod impls {
    use std::{
        cell::RefCell,
        cmp::min,
        collections::{HashMap, VecDeque},
        ptr::copy_nonoverlapping,
        slice::from_raw_parts,
    };

    use keccak_const::Keccak256;

    type U = [u8; 32];
    type Address = [u8; 20];

    type VersionedWord = (u32, [u8; 32]);
    type WordHashMap = HashMap<[u8; 32], VecDeque<VersionedWord>>;

    const MAX_STORAGE_VERSIONS: usize = 10;

    // The storage uses versioning, allowing uncommitted native writes to
    // be rolled back to the last flushed version. Loads always read the
    // latest cached value; rollback removes newer uncommitted entries.
    // There can only be 10 versions in circulation at a time.
    thread_local! {
        pub static STORAGE: RefCell<WordHashMap> = RefCell::default();
        pub static TRANSIENT: RefCell<WordHashMap> = RefCell::default();
        pub static VERSION: RefCell<u32> = RefCell::default();
        static COMMITTED_VERSION: RefCell<u32> = RefCell::default();
    }

    /// Copy the underlying storage for logging purposes.
    pub fn storage_copy() -> WordHashMap {
        STORAGE.with(|s| s.borrow().clone())
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

    fn current_version() -> u32 {
        VERSION.with(|v| *v.borrow())
    }

    fn committed_version() -> u32 {
        COMMITTED_VERSION.with(|v| *v.borrow())
    }

    fn set_committed_version(version: u32) {
        COMMITTED_VERSION.with(|v| *v.borrow_mut() = version);
    }

    fn load_versioned_word(words: &VecDeque<VersionedWord>) -> [u8; 32] {
        words.back().map(|(_, value)| *value).unwrap_or([0u8; 32])
    }

    fn rollback_uncommitted_storage(storage: &mut WordHashMap) {
        let committed = committed_version();
        storage.retain(|_, words| {
            while matches!(words.back(), Some((version, _)) if *version > committed) {
                words.pop_back();
            }
            !words.is_empty()
        });
    }

    fn store_versioned_word(storage: &mut WordHashMap, key: [u8; 32], value: [u8; 32]) {
        let version = current_version();
        let words = storage.entry(key).or_default();
        words.push_back((version, value));
        while words.len() > MAX_STORAGE_VERSIONS {
            words.pop_front();
        }
    }

    pub unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = STORAGE.with(|s| match s.borrow().get(&k) {
            Some(v) => load_versioned_word(v),
            None => [0u8; 32],
        });
        unsafe { write_word(out, value) };
    }

    pub unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        STORAGE.with(|s| store_versioned_word(&mut s.borrow_mut(), k, v));
    }

    pub unsafe fn transient_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = TRANSIENT.with(|s| match s.borrow().get(&k) {
            Some(v) => load_versioned_word(v),
            None => [0u8; 32],
        });
        unsafe { write_word(out, value) };
    }

    pub unsafe fn transient_store_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        TRANSIENT.with(|s| store_versioned_word(&mut s.borrow_mut(), k, v));
    }

    pub fn rollback_storage() {
        STORAGE.with(|s| rollback_uncommitted_storage(&mut s.borrow_mut()));
    }

    pub fn persist_storage() {
        set_committed_version(current_version());
    }

    pub unsafe fn storage_flush_cache(clear: bool) {
        if clear {
            rollback_storage();
        } else {
            persist_storage();
        }
    }

    pub fn storage_reset() {
        storage_clear();
        transient_clear();
        set_version(0);
        set_committed_version(0);
    }

    fn set_version(version: u32) {
        VERSION.with(|v| *v.borrow_mut() = version);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn storage_loads_latest_even_when_current_version_is_older() {
            storage_reset();
            let key = [1u8; 32];
            let value_0 = [2u8; 32];
            let value_1 = [3u8; 32];
            let mut out = [0u8; 32];

            unsafe { storage_cache_bytes32(key.as_ptr(), value_0.as_ptr()) };
            set_version(1);
            unsafe { storage_cache_bytes32(key.as_ptr(), value_1.as_ptr()) };

            set_version(0);
            unsafe { storage_load_bytes32(key.as_ptr(), out.as_mut_ptr()) };
            assert_eq!(value_1, out);
        }

        #[test]
        fn transient_loads_latest_even_when_current_version_is_older() {
            storage_reset();
            let key = [4u8; 32];
            let value_0 = [5u8; 32];
            let value_1 = [6u8; 32];
            let mut out = [0u8; 32];
            unsafe { transient_store_bytes32(key.as_ptr(), value_0.as_ptr()) };
            set_version(1);
            unsafe { transient_store_bytes32(key.as_ptr(), value_1.as_ptr()) };
            set_version(0);
            unsafe { transient_load_bytes32(key.as_ptr(), out.as_mut_ptr()) };
            assert_eq!(value_1, out);
        }

        #[test]
        fn storage_rollback_discards_versions_newer_than_committed() {
            storage_reset();
            let key = [8u8; 32];
            let value_0 = [9u8; 32];
            let value_1 = [10u8; 32];
            let mut out = [0u8; 32];
            unsafe { storage_cache_bytes32(key.as_ptr(), value_0.as_ptr()) };
            unsafe { storage_flush_cache(false) };
            set_version(1);
            unsafe { storage_cache_bytes32(key.as_ptr(), value_1.as_ptr()) };
            unsafe { storage_load_bytes32(key.as_ptr(), out.as_mut_ptr()) };
            assert_eq!(value_1, out);
            unsafe { storage_flush_cache(true) };
            unsafe { storage_load_bytes32(key.as_ptr(), out.as_mut_ptr()) };
            assert_eq!(value_0, out);
        }

        #[test]
        fn storage_rollback_keeps_flushed_versions() {
            storage_reset();
            let key = [11u8; 32];
            let value_0 = [12u8; 32];
            let value_1 = [13u8; 32];
            let mut out = [0u8; 32];
            unsafe { storage_cache_bytes32(key.as_ptr(), value_0.as_ptr()) };
            unsafe { storage_flush_cache(false) };
            set_version(1);
            unsafe { storage_cache_bytes32(key.as_ptr(), value_1.as_ptr()) };
            unsafe { storage_flush_cache(false) };
            unsafe { storage_flush_cache(true) };
            unsafe { storage_load_bytes32(key.as_ptr(), out.as_mut_ptr()) };
            assert_eq!(value_1, out);
        }

        #[test]
        fn storage_versions_are_capped_to_ten_entries() {
            storage_reset();
            let key = [7u8; 32];
            for version in 0..11 {
                set_version(version);
                let value = [version as u8; 32];
                unsafe { storage_cache_bytes32(key.as_ptr(), value.as_ptr()) };
            }
            STORAGE.with(|s| {
                let storage = s.borrow();
                let words = storage.get(&key).unwrap();
                assert_eq!(MAX_STORAGE_VERSIONS, words.len());
                assert_eq!(Some(&(1, [1u8; 32])), words.front());
                assert_eq!(Some(&(10, [10u8; 32])), words.back());
            });
        }
    }

    thread_local! {
        static ACCOUNT_BALANCE: RefCell<HashMap<Address, U>> = RefCell::default();
        static ARGS: RefCell<Vec<u8>> = RefCell::default();
        static MSG_SENDER: RefCell<[u8; 20]> = RefCell::default();
        static CONTRACT_ADDRESS: RefCell<Address> = RefCell::default();
        static MSG_VALUE: RefCell<U> = RefCell::default();
        static CHAIN_ID: RefCell<u64> = RefCell::default();
        static BLOCK_TIMESTAMP: RefCell<u64> = RefCell::default();
        static ACCOUNT_CODE: RefCell<HashMap<Address, Vec<u8>>> = RefCell::default();
    }

    const EMPTY_HASH: U = [
        0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03,
        0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85,
        0xa4, 0x70,
    ];

    pub unsafe fn account_balance(addr_: *const u8, out: *mut u8) {
        ACCOUNT_BALANCE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 32);
            }
            let h = s.borrow();
            let amt = h.get(&addr).unwrap_or(&[0u8; 32]);
            unsafe {
                copy_nonoverlapping(amt.as_ptr(), out, 32);
            }
        })
    }

    #[allow(unused)]
    pub unsafe fn pay_for_memory_grow(_: u16) {}

    pub unsafe fn write_result(d: *const u8, l: usize) {
        println!("{}", const_hex::encode(unsafe { from_raw_parts(d, l) }));
    }

    pub unsafe fn return_data_size() -> usize {
        0
    }

    pub fn set_args(x: Vec<u8>) {
        ARGS.with(|s| *s.borrow_mut() = x)
    }

    pub fn args_len() -> usize {
        ARGS.with(|s| s.borrow().len())
    }

    pub unsafe fn read_args(out: *mut u8) {
        ARGS.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, b.len());
            }
        })
    }

    pub fn set_msg_sender(x: Address) {
        MSG_SENDER.with(|s| *s.borrow_mut() = x)
    }

    pub unsafe fn msg_sender(out: *mut u8) {
        MSG_SENDER.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, 20);
            }
        })
    }

    pub fn set_contract_address(x: Address) {
        CONTRACT_ADDRESS.with(|s| {
            *s.borrow_mut() = x;
        })
    }

    pub fn set_account_code(x: Address, code: Vec<u8>) {
        ACCOUNT_CODE.with(|s| s.borrow_mut().insert(x, code));
    }

    pub unsafe fn contract_address(out: *mut u8) {
        CONTRACT_ADDRESS.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, 20);
            }
        })
    }

    pub unsafe fn msg_value(out: *mut u8) {
        MSG_VALUE.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, 32);
            }
        })
    }

    pub unsafe fn chainid() -> u64 {
        CHAIN_ID.with(|s| *s.borrow())
    }

    pub unsafe fn account_code_size(addr_: *const u8) -> usize {
        ACCOUNT_CODE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 20);
            }
            s.borrow().get(&addr).map(|s| s.len()).unwrap_or(0)
        })
    }

    pub unsafe fn account_code(
        addr_: *const u8,
        offset: usize,
        size: usize,
        out: *mut u8,
    ) -> usize {
        ACCOUNT_CODE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 20);
            }
            let b = s.borrow();
            match b.get(&addr) {
                Some(b) => {
                    if offset >= b.len() {
                        return 0;
                    }
                    let src = &b[offset..];
                    let len = min(size, src.len());
                    unsafe {
                        copy_nonoverlapping(src.as_ptr(), out, len);
                    }
                    len
                }
                None => 0,
            }
        })
    }

    fn keccak256(h: &[u8]) -> [u8; 32] {
        Keccak256::new().update(h).finalize()
    }

    pub unsafe fn account_codehash(addr_: *const u8, out: *mut u8) {
        ACCOUNT_CODE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 20);
            }
            let b = s.borrow();
            let h = match b.get(&addr) {
                None => EMPTY_HASH,
                Some(b) if b.is_empty() => EMPTY_HASH,
                Some(b) => keccak256(b),
            };
            unsafe {
                copy_nonoverlapping(h.as_ptr(), out, 32);
            }
        })
    }

    pub fn set_block_timestamp(n: u64) {
        BLOCK_TIMESTAMP.with(|s| *s.borrow_mut() = n)
    }

    pub unsafe fn block_timestamp() -> u64 {
        BLOCK_TIMESTAMP.with(|s| *s.borrow())
    }

    pub unsafe fn block_basefee(_: *mut u8) {}

    pub unsafe fn evm_gas_left() -> u64 {
        0
    }

    pub unsafe fn evm_ink_left() -> u64 {
        0
    }
}

#[cfg(feature = "mutex")]
mod impls {
    use std::{
        collections::{HashMap, VecDeque},
        ptr::copy_nonoverlapping,
        sync::{LazyLock, Mutex},
    };

    type VersionedWord = (u32, [u8; 32]);
    type WordHashMap = HashMap<[u8; 32], VecDeque<VersionedWord>>;

    const MAX_STORAGE_VERSIONS: usize = 10;

    pub static STORAGE: LazyLock<Mutex<WordHashMap>> = LazyLock::new(|| Mutex::default());
    pub static TRANSIENT: LazyLock<Mutex<WordHashMap>> = LazyLock::new(|| Mutex::default());
    pub static VERSION: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::default());
    static COMMITTED_VERSION: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::default());

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
        r
    }

    unsafe fn write_word(key: *mut u8, val: [u8; 32]) {
        unsafe {
            copy_nonoverlapping(val.as_ptr(), key, 32);
        }
    }

    fn current_version() -> u32 {
        *VERSION.lock().unwrap()
    }

    fn committed_version() -> u32 {
        *COMMITTED_VERSION.lock().unwrap()
    }

    fn set_committed_version(version: u32) {
        *COMMITTED_VERSION.lock().unwrap() = version;
    }

    fn load_versioned_word(words: &VecDeque<VersionedWord>) -> [u8; 32] {
        words.back().map(|(_, value)| *value).unwrap_or([0u8; 32])
    }

    fn rollback_uncommitted_storage(storage: &mut WordHashMap) {
        let committed = committed_version();
        storage.retain(|_, words| {
            while matches!(words.back(), Some((version, _)) if *version > committed) {
                words.pop_back();
            }
            !words.is_empty()
        });
    }

    fn store_versioned_word(storage: &mut WordHashMap, key: [u8; 32], value: [u8; 32]) {
        let version = current_version();
        let words = storage.entry(key).or_default();
        words.push_back((version, value));
        while words.len() > MAX_STORAGE_VERSIONS {
            words.pop_front();
        }
    }

    pub unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = match STORAGE.lock().unwrap().get(&k) {
            Some(v) => load_versioned_word(v),
            None => [0u8; 32],
        };
        unsafe { write_word(out, value) };
    }

    pub unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        store_versioned_word(&mut STORAGE.lock().unwrap(), k, v);
    }

    pub unsafe fn transient_load_bytes32(key: *const u8, out: *mut u8) {
        let k = unsafe { read_word(key) };
        let value = match TRANSIENT.lock().unwrap().get(&k) {
            Some(v) => load_versioned_word(v),
            None => [0u8; 32],
        };
        unsafe { write_word(out, value) };
    }

    pub unsafe fn transient_store_bytes32(key: *const u8, value: *const u8) {
        let k = unsafe { read_word(key) };
        let v = unsafe { read_word(value) };
        store_versioned_word(&mut TRANSIENT.lock().unwrap(), k, v);
    }

    pub unsafe fn storage_flush_cache(clear: bool) {
        if clear {
            rollback_uncommitted_storage(&mut STORAGE.lock().unwrap());
        } else {
            set_committed_version(current_version());
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

// We let the user decide how they want to provide these symbols:

unsafe extern "C" {
    pub fn math_div(x: *mut u8, y: *const u8);
    pub fn math_mod(x: *mut u8, y: *const u8);
    pub fn math_add_mod(a: *mut u8, b: *const u8, c: *const u8);
    pub fn math_mul_mod(a: *mut u8, b: *const u8, c: *const u8);
}

pub unsafe fn exit_early(_: i32) -> ! {
    todo!("implement dispatch function");
}

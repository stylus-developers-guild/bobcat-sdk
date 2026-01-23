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

pub unsafe fn storage_load_bytes32(_: *const u8, _: *mut u8) {}

pub unsafe fn storage_cache_bytes32(_: *const u8, _: *const u8) {}

pub unsafe fn transient_load_bytes32(_: *const u8, _: *mut u8) {}

pub unsafe fn transient_store_bytes32(_: *const u8, _: *const u8) {}

pub unsafe fn storage_flush_cache(_: bool) {}

pub fn log_txt(_: *const u8, _: usize) {}

unsafe extern "C" {
    pub fn math_div(x: *mut u8, y: *const u8);
    pub fn math_mod(x: *mut u8, y: *const u8);
    pub fn math_add_mod(a: *mut u8, b: *const u8, c: *const u8);
    pub fn math_mul_mod(a: *mut u8, b: *const u8, c: *const u8);
}

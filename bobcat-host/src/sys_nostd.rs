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

pub unsafe fn account_balance(_: *const u8, _: *mut u8) {}

pub unsafe fn pay_for_memory_grow(_: u16) {}

pub unsafe fn write_result(_: *const u8, _: usize) {}

pub unsafe fn return_data_size() -> usize {
    0
}

pub unsafe fn read_args(_out: *mut u8) {}

pub unsafe fn msg_sender(_: *mut u8) {}

pub unsafe fn contract_address(_: *mut u8) {}

pub unsafe fn msg_value(_: *mut u8) {}

pub unsafe fn chainid() -> u64 {
    0
}

pub unsafe fn account_code_size(_: *const u8) -> usize {
    0
}

pub unsafe fn account_code(_: *const u8, _: usize, _: usize, _: *mut u8) -> usize {
    0
}

pub unsafe fn account_codehash(_: *const u8, _: *mut u8) {}

pub unsafe fn block_timestamp() -> u64 {
    0
}

pub unsafe fn block_basefee(_: *mut u8) {}

pub unsafe fn evm_gas_left() -> u64 {
    0
}

pub unsafe fn evm_ink_left() -> u64 {
    0
}

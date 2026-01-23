#[link(wasm_import_module = "console")]
#[cfg(feature = "console")]
unsafe extern "C" {
    pub fn log_txt(ptr: *const u8, len: usize);
}

#[link(wasm_import_module = "vm_hooks")]
#[allow(unused)]
unsafe extern "C" {
    pub fn exit_early(code: i32) -> !;
    pub fn write_result(d: *const u8, l: usize);
    pub fn transient_store_bytes32(key: *const u8, value: *const u8);
    pub fn transient_load_bytes32(key: *const u8, value: *const u8);
    pub fn math_div(x: *mut u8, y: *const u8);
    pub fn math_mod(x: *mut u8, y: *const u8);
    pub fn math_add_mod(a: *mut u8, b: *const u8, c: *const u8);
    pub fn math_mul_mod(a: *mut u8, b: *const u8, c: *const u8);
    pub fn create1(
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize,
    );
    pub fn create2(
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        salt: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize,
    );
    pub fn storage_load_bytes32(key: *const u8, out: *mut u8);
    pub fn storage_cache_bytes32(key: *const u8, from: *const u8);
    pub fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8);
    pub fn storage_flush_cache(clear: bool);
    pub fn emit_log(data: *const u8, len: usize, topics: usize);
    pub fn call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        value: *const u8,
        gas: u64,
        return_data_len: *mut usize,
    ) -> u8;
    pub fn static_call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        gas: u64,
        return_data_len: *mut usize,
    ) -> u8;
    pub fn delegate_call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        gas: u64,
        return_data_len: *mut usize,
    ) -> u8;
    pub fn read_return_data(dest: *mut u8, offset: usize, size: usize) -> usize;
    pub fn account_balance(addr: *const u8, dest: *mut u8);
    #[allow(unused)]
    pub fn pay_for_memory_grow(pages: u16);
    pub fn return_data_size() -> usize;
    pub fn read_args(out: *mut u8);
    pub fn msg_sender(addr: *mut u8);
    pub fn contract_address(addr: *mut u8);
    pub fn msg_value(value: *mut u8);
    pub fn chainid() -> u64;
    pub fn account_code_size(address: *const u8) -> usize;
    pub fn account_code(address: *const u8, offset: usize, size: usize, dest: *mut u8) -> usize;
    pub fn account_codehash(address: *const u8, dest: *mut u8);
    pub fn block_timestamp() -> u64;
    pub fn block_basefee(out: *mut u8);
    pub fn evm_gas_left() -> u64;
    pub fn evm_ink_left() -> u64;
}

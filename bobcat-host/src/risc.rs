// This risc code doesn't use the coprocessor-extended functionality for
// EVM operations.

use core::{arch::asm, prelude::rust_2024::derive};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Ecall {
    AccountBalance = 0,
    AccountCode = 1,
    AccountCodeSize = 2,
    AccountCodehash = 3,
    EthereumLoad = 4,
    EthereumStore = 5,
    StorageFlushCache = 6,
    BlockBasefee = 7,
    Chainid = 8,
    BlockCoinbase = 9,
    BlockGasLimit = 10,
    BlockNumber = 11,
    BlockTimestamp = 12,
    CallContract = 13,
    ContractAddress = 14,
    Create1 = 15,
    Create2 = 16,
    DelegateCallContract = 17,
    EmitLog = 18,
    EvmGasLeft = 19,
    EvmInkLeft = 20,
    PayForMemoryGrow = 21,
    MsgReentrant = 22,
    MsgSender = 23,
    MsgValue = 24,
    NativeKeccak256 = 25,
    ReadArgs = 26,
    ReadReturnData = 27,
    WriteResult = 28,
    ReturnDataSize = 29,
    StaticCallContract = 30,
    TxGasPrice = 31,
    TxInkPrice = 32,
    TxOrigin = 33,
    ArgsLen = 34,
    Console = 35,
}

pub unsafe fn log_txt(ptr: *const u8, len: usize) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::Console as usize,
            in("a0") ptr,
            in("a1") len,
            options(nostack)
        );
    }
}

pub unsafe fn write_result(d: *const u8, l: usize) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::WriteResult as usize,
            in("a0") d,
            in("a1") l,
            options(nostack)
        );
    }
}

pub unsafe fn storage_load_bytes32(key: *const u8, out: *mut u8) {
    unsafe {
        core::ptr::copy_nonoverlapping(key, out, 32);
        asm!(
            "ecall",
            in("a7") Ecall::EthereumLoad as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

pub unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::EthereumStore as usize,
            in("a0") key,
            in("a1") value,
            options(nostack)
        );
    }
}

pub unsafe fn storage_flush_cache(clear: bool) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::StorageFlushCache as usize,
            in("a1") clear as usize,
            options(nostack)
        );
    }
}

pub unsafe fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8) {
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, output, 32);
        asm!(
            "ecall",
            in("a7") Ecall::NativeKeccak256 as usize,
            in("a0") output,
            in("a1") len,
            options(nostack)
        );
    }
}

pub unsafe fn emit_log(data: *const u8, len: usize, topics: usize) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::EmitLog as usize,
            in("a0") data,
            in("a1") len,
            in("a2") topics,
            options(nostack)
        );
    }
}

pub unsafe fn account_balance(addr: *const u8, dest: *mut u8) {
    unsafe {
        core::ptr::copy_nonoverlapping(addr, dest, 20);
        asm!(
            "ecall",
            in("a7") Ecall::AccountBalance as usize,
            in("a0") dest,
            options(nostack)
        );
    }
}

pub unsafe fn account_code(address: *const u8, offset: usize, size: usize, dest: *mut u8) -> usize {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::AccountCode as usize,
            in("a0") address,
            in("a1") offset,
            in("a2") size,
            in("a3") dest,
            options(nostack)
        );
    }
    size
}

pub unsafe fn account_code_size(address: *const u8) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::AccountCodeSize as usize,
            inout("a0") address => ret,
            options(nostack)
        );
    }
    ret
}

pub unsafe fn account_codehash(address: *const u8, dest: *mut u8) {
    unsafe {
        core::ptr::copy_nonoverlapping(address, dest, 20);
        asm!(
            "ecall",
            in("a7") Ecall::AccountCodehash as usize,
            in("a0") dest,
            options(nostack)
        );
    }
}

pub unsafe fn block_basefee(out: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::BlockBasefee as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

pub unsafe fn chainid() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::Chainid as usize,
            lateout("a0") hi,
            lateout("a1") lo,
            options(nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub unsafe fn block_coinbase(out: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::BlockCoinbase as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

pub unsafe fn block_gas_limit() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::BlockGasLimit as usize,
            lateout("a0") hi,
            lateout("a1") lo,
            options(nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub unsafe fn block_number() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::BlockNumber as usize,
            lateout("a0") hi,
            lateout("a1") lo,
            options(nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub unsafe fn block_timestamp() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::BlockTimestamp as usize,
            lateout("a0") hi,
            lateout("a1") lo,
            options(nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub unsafe fn call_contract(
    contract: *const u8,
    calldata: *const u8,
    calldata_len: usize,
    value: *const u8,
    gas: u64,
    return_data_len: *mut usize,
) -> u8 {
    let ret: usize;
    let gas_hi = (gas >> 32) as u32;
    let gas_lo = gas as u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::CallContract as usize,
            inout("a0") contract as usize => ret,
            in("a1") calldata,
            in("a2") calldata_len,
            in("a3") value,
            in("a4") gas_hi,
            in("a5") gas_lo,
            in("a6") return_data_len,
            options(nostack)
        );
    }
    ret as u8
}

pub unsafe fn static_call_contract(
    contract: *const u8,
    calldata: *const u8,
    calldata_len: usize,
    gas: u64,
    return_data_len: *mut usize,
) -> u8 {
    let ret: usize;
    let gas_hi = (gas >> 32) as u32;
    let gas_lo = gas as u32;
    let dummy_value: [u8; 32] = [0; 32];
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::StaticCallContract as usize,
            inout("a0") contract as usize => ret,
            in("a1") calldata,
            in("a2") calldata_len,
            in("a3") dummy_value.as_ptr(),
            in("a4") gas_hi,
            in("a5") gas_lo,
            in("a6") return_data_len,
            options(nostack)
        );
    }
    ret as u8
}

pub unsafe fn delegate_call_contract(
    contract: *const u8,
    calldata: *const u8,
    calldata_len: usize,
    gas: u64,
    return_data_len: *mut usize,
) -> u8 {
    let ret: usize;
    let gas_hi = (gas >> 32) as u32;
    let gas_lo = gas as u32;
    let dummy_value: [u8; 32] = [0; 32];
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::DelegateCallContract as usize,
            inout("a0") contract as usize => ret,
            in("a1") calldata,
            in("a2") calldata_len,
            in("a3") dummy_value.as_ptr(),
            in("a4") gas_hi,
            in("a5") gas_lo,
            in("a6") return_data_len,
            options(nostack)
        );
    }
    ret as u8
}

pub unsafe fn contract_address(addr: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::ContractAddress as usize,
            in("a0") addr,
            options(nostack)
        );
    }
}

pub unsafe fn create1(
    code: *const u8,
    code_len: usize,
    endowment: *const u8,
    contract: *mut u8,
    revert_data_len: *mut usize,
) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::Create1 as usize,
            in("a0") contract,
            in("a1") code_len,
            in("a2") endowment,
            in("a3") code,
            in("a4") revert_data_len,
            options(nostack)
        );
    }
}

pub unsafe fn create2(
    code: *const u8,
    code_len: usize,
    endowment: *const u8,
    salt: *const u8,
    contract: *mut u8,
    revert_data_len: *mut usize,
) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::Create2 as usize,
            in("a0") contract,
            in("a1") code_len,
            in("a2") endowment,
            in("a3") salt,
            in("a4") code,
            in("a5") revert_data_len,
            options(nostack)
        );
    }
}

pub unsafe fn evm_gas_left() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::EvmGasLeft as usize,
            lateout("a0") hi,
            lateout("a1") lo,
            options(nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub unsafe fn evm_ink_left() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::EvmInkLeft as usize,
            lateout("a0") hi,
            lateout("a1") lo,
            options(nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub unsafe fn pay_for_memory_grow(pages: u16) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::PayForMemoryGrow as usize,
            in("a0") pages as usize,
            options(nostack)
        );
    }
}

pub unsafe fn msg_reentrant() -> bool {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::MsgReentrant as usize,
            lateout("a0") ret,
            options(nostack)
        );
    }
    ret != 0
}

pub unsafe fn msg_sender(addr: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::MsgSender as usize,
            in("a0") addr,
            options(nostack)
        );
    }
}

pub unsafe fn msg_value(value: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::MsgValue as usize,
            in("a0") value,
            options(nostack)
        );
    }
}

pub unsafe fn read_args(out: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::ReadArgs as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

pub unsafe fn args_len() -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::ArgsLen as usize,
            lateout("a0") ret,
            options(nostack)
        );
    }
    ret
}

pub unsafe fn read_return_data(dest: *mut u8, offset: usize, size: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::ReadReturnData as usize,
            inout("a0") dest => ret,
            in("a1") offset,
            in("a2") size,
            options(nostack)
        );
    }
    ret
}

pub unsafe fn return_data_size() -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::ReturnDataSize as usize,
            lateout("a0") ret,
            options(nostack)
        );
    }
    ret
}

pub unsafe fn tx_gas_price(out: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::TxGasPrice as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

pub unsafe fn tx_ink_price(out: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::TxInkPrice as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

pub unsafe fn tx_origin(out: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") Ecall::TxOrigin as usize,
            in("a0") out,
            options(nostack)
        );
    }
}

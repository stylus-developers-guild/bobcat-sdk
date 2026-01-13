#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bobcat_maths::U;

use bobcat_entry::code_hash;

#[allow(unused)]
use bobcat_panic::panic_on_err_bad_decoding_bool;

type Address = [u8; 20];

#[cfg(target_arch = "wasm32")]
mod impls {
    #[link(wasm_import_module = "vm_hooks")]
    unsafe extern "C" {
        pub(crate) fn call_contract(
            contract: *const u8,
            calldata: *const u8,
            calldata_len: usize,
            value: *const u8,
            gas: u64,
            return_data_len: *mut usize,
        ) -> u8;

        pub(crate) fn static_call_contract(
            contract: *const u8,
            calldata: *const u8,
            calldata_len: usize,
            gas: u64,
            return_data_len: *mut usize,
        ) -> u8;

        pub(crate) fn delegate_call_contract(
            contract: *const u8,
            calldata: *const u8,
            calldata_len: usize,
            gas: u64,
            return_data_len: *mut usize,
        ) -> u8;

        pub(crate) fn read_return_data(dest: *mut u8, offset: usize, size: usize) -> usize;
    }
}

pub fn read_return_data_slice<const CAP: usize>(offset: usize, size: usize) -> ([u8; CAP], usize) {
    let mut b = [0u8; CAP];
    let rd = unsafe { impls::read_return_data(b.as_mut_ptr(), offset, size) };
    (b, rd)
}

#[cfg(feature = "alloc")]
pub fn read_return_data_vec(offset: usize, size: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(size);
    let rd = unsafe { impls::read_return_data(b.as_mut_ptr(), offset, size) };
    unsafe {
        b.set_len(rd);
    }
    b
}

#[cfg(not(target_arch = "wasm32"))]
mod impls {
    pub(crate) unsafe fn call_contract(
        _contract: *const u8,
        _calldata: *const u8,
        _calldata_len: usize,
        _value: *const u8,
        _gas: u64,
        _return_data_len: *mut usize,
    ) -> u8 {
        0
    }

    pub(crate) unsafe fn static_call_contract(
        _contract: *const u8,
        _calldata: *const u8,
        _calldata_len: usize,
        _gas: u64,
        _return_data_len: *mut usize,
    ) -> u8 {
        0
    }

    pub(crate) unsafe fn delegate_call_contract(
        _contract: *const u8,
        _calldata: *const u8,
        _calldata_len: usize,
        _gas: u64,
        _return_data_len: *mut usize,
    ) -> u8 {
        0
    }

    pub(crate) unsafe fn read_return_data(_: *mut u8, _: usize, _: usize) -> usize {
        0
    }
}

use impls::{
    call_contract as call, delegate_call_contract as delegate_call,
    static_call_contract as static_call,
};

pub fn addr_has_code(addr: Address) -> bool {
    // It costs to use the length instead of the codehash, so we do it this
    // way for free. We compare it against the zero code hash:
    code_hash(addr)
        != [
            197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182,
            83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112,
        ]
}

macro_rules! generate_call_variants {
    ($base_fn:ident, has_value) => {
        generate_call_variants!(@impl $base_fn, value: &U);
    };
    ($base_fn:ident) => {
        generate_call_variants!(@impl $base_fn,);
    };
    (@impl $base_fn:ident, $($value_param:ident: $value_ty:ty)?) => {
        paste::paste! {
            /// Call a contract with the given parameters. Returns a tuple of
            /// (success, return_data_length).
            pub fn [<$base_fn _partial>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> (bool, usize) {
                let mut return_data_len = 0usize;
                let status = unsafe {
                    $base_fn(
                        contract.as_ptr(),
                        calldata.as_ptr(),
                        calldata.len(),
                        $($value_param.as_ptr(),)?
                        gas,
                        &mut return_data_len as *mut usize,
                    )
                };
                (status == 0, return_data_len)
            }

            /// Call a contract, ignoring its returndata. Returns true if contract
            /// invoked successfully.
            pub fn [<$base_fn _unit>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64
            ) -> bool {
                [<$base_fn _partial>](contract, calldata, $($value_param,)? gas).0
            }

            /// Call a contract, ignoring its returndata. Returns Some(()) if the
            /// contract invoked successfully.
            pub fn [<$base_fn _unit_opt>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> Option<()> {
                if [<$base_fn _unit>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas,
                ) {
                    Some(())
                } else {
                    None
                }
            }

            /// Call a contract, writing its returndata to the slice given. Returns
            /// true for if the contract ran without issue, or false if a revert
            /// happened, and the slice isn't read to if a revert happens.
            pub fn [<$base_fn _slice>]<const DATA_CAP: usize>(
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> (bool, usize, [u8; DATA_CAP]) {
                let mut b = [0u8; DATA_CAP];
                let (rc, rd_len) = [<$base_fn _partial>](contract, calldata, $($value_param,)? gas);
                if !rc {
                    return (false, rd_len, b);
                }
                panic_on_err_bad_decoding_bool!(
                    rd_len > offset,
                    "offset greater than rd len ok"
                );
                let size = rd_len - offset;
                panic_on_err_bad_decoding_bool!(
                    DATA_CAP >= size,
                    "not enough slice capacity"
                );
                unsafe { impls::read_return_data(b.as_mut_ptr(), offset, DATA_CAP) };
                (rc, rd_len, b)
            }

            /// Call a contract, writing its returndata to the slice given. Returns
            /// true for if the contract ran without issue, or false if a revert
            /// happened. An offset can be used to start reading the return data from,
            /// writing to the buffer given. The code will not read from the offset
            /// given if a revert has happened. The function will panic if the
            /// returndata exceeds the capacity. Enforces write control if static, and
            /// delegates if delegatecall. The function will also panic if the offset
            /// is greater than the size of the returndata. Programmers making this
            /// mistake must be making an error with the decoding.
            pub fn [<$base_fn _err_slice>]<const DATA_CAP: usize>(
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> (bool, usize, [u8; DATA_CAP]) {
                let (rc, rd_len) = [<$base_fn _partial>](contract, calldata, $($value_param,)? gas);
                // When it comes to a revert, we don't cut it up like we do a normal slice.
                let size = if rc {
                    panic_on_err_bad_decoding_bool!(
                        rd_len > offset,
                        "offset greater than rd len ok"
                    );
                    rd_len - offset
                } else {
                    rd_len
                };
                // Capacity errors if the contract was in error where the revert value
                // size isn't known and we want to show it to the user should use err_vec
                // functions instead with the allocator.
                panic_on_err_bad_decoding_bool!(
                    DATA_CAP >= size,
                    "not enough _slice capacity"
                );
                let mut b = [0u8; DATA_CAP];
                unsafe { impls::read_return_data(b.as_mut_ptr(), offset, DATA_CAP) };
                (rc, size, b)
            }

            /// Call a contract, returning a word of the returndata/revertdata. Complains
            /// if the other party does not write exactly a word, regardless of the reason!
            pub fn [<$base_fn _word>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> (bool, U) {
                let (rc, len, v) = [<$base_fn _slice>]::<32>(contract, calldata, $($value_param,)? gas, offset);
                panic_on_err_bad_decoding_bool!(len == 32, "response didn't write 32");
                (rc, U::from(v))
            }

            /// Same as the other slice function, though returning Option if error.
            pub fn [<$base_fn _slice_opt>]<const DATA_CAP: usize>(
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> Option<(usize, [u8; DATA_CAP])> {
                let (rc, len, c) = [<$base_fn _slice>]::<DATA_CAP>(
                    contract,
                    calldata,
                    $($value_param,)?
                    gas,
                    offset,
                );
                if rc {
                    Some((len, c))
                } else {
                    None
                }
            }

            /// Same as the other slice function, returning Result depending on
            /// return or revert.
            pub fn [<$base_fn _slice_res>]<const DATA_CAP: usize>(
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> Result<(usize, [u8; DATA_CAP]), (usize, [u8; DATA_CAP])> {
                let (rc, len, c) = [<$base_fn _slice>]::<DATA_CAP>(
                    contract,
                    calldata,
                    $($value_param,)?
                    gas,
                    offset,
                );
                if rc {
                    Ok((len, c))
                } else {
                    Err((len, c))
                }
            }

            /// Same as the slice variant, except check the length of the code for the
            /// address first. If code doesn't exist, then we return None.
            pub fn [<safe_ $base_fn _slice>]<const DATA_CAP: usize>(
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> Option<(bool, usize, [u8; DATA_CAP])> {
                if addr_has_code(contract) {
                    Some([<$base_fn _slice>]::<DATA_CAP>(
                        contract, calldata, $($value_param,)? gas, offset,
                    ))
                } else {
                    None
                }
            }

            /// Invoke call, only reading a single byte at the first word for a
            /// check.
            pub fn [<$base_fn _bool>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> bool {
                let (rc, _, v) = [<$base_fn _slice>]::<1>(
                    contract, calldata, $($value_param,)? gas, 31,
                );
                rc && v[0] == 1
            }

            /// Invoke call, only reading a single byte at the first word for a
            /// check. Returns Some if success.
            pub fn [<$base_fn _bool_opt>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> Option<()> {
                if [<$base_fn _bool>](contract, calldata, $($value_param,)? gas) {
                    Some(())
                } else {
                    None
                }
            }

            /// Check the codesize before invoking call, only reading a single byte
            /// at the location for a bool check. If the contract doesn't return anything,
            /// then we assume everything went okay. If it does, then we check for true.
            /// We don't return anything.
            pub fn [<safe_ $base_fn _bool>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> bool {
                if addr_has_code(contract) {
                    match [<$base_fn _partial>](contract, calldata, $($value_param,)? gas) {
                        (true, 32) => {
                            let mut b = [0u8; 1];
                            unsafe { impls::read_return_data(b.as_mut_ptr(), 31, 1) };
                            b[0] == 1
                        }
                        (true, 0) => true,
                        (true, _) => panic_on_err_bad_decoding_bool!("word not returned"),
                        _ => false
                    }
                } else {
                    false
                }
            }

            /// Check the codesize before invoking call, returning a Option<()> if
            /// the contract call worked, and the return value is true.
            pub fn [<safe_ $base_fn _bool_opt>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> Option<()> {
                if [<safe_ $base_fn _bool>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas
                ) {
                    Some(())
                } else {
                    None
                }
            }

            /// Call a contract, writing its returndata to the vector given. Behaves
            /// the same way as the slice function.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> (bool, Vec<u8>) {
                let (rc, rd_len) = [<$base_fn _partial>](contract, calldata, $($value_param,)? gas);
                let size = rd_len - offset;
                let mut b = Vec::with_capacity(size);
                unsafe { b.set_len(size) }
                unsafe { impls::read_return_data(b.as_mut_ptr(), offset, size) };
                (rc, b)
            }

            /// Same as the other vec function, returning Option if error.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _vec_opt>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> Option<Vec<u8>> {
                let (rc, v) = [<$base_fn _vec>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas,
                    offset,
                );
                if rc {
                    Some(v)
                } else {
                    None
                }
            }

            /// Same as the other vec function, returning Result depending on
            /// return or revert.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _vec_res>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> Result<Vec<u8>, Vec<u8>> {
                let (rc, rd) = [<$base_fn _vec>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas,
                    offset,
                );
                if rc {
                    Ok(rd)
                } else {
                    Err(rd)
                }
            }

            /// Return a word if successful, a vector if a revert.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _word_err_vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> (bool, U, Option<Vec<u8>>) {
                // Why use a vector for this entirely? Normally, you'd prefer stack space
                // for data that might be used in a way with a performance context, like
                // a word for an addition. For locality reasons. But, we want to save
                // codesize by reducing complexity! So, we prefer to just depend on the
                // allocator if that's what's in use here.
                let (rc, v) = [<$base_fn _vec>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas,
                    0
                );
                if rc {
                    panic_on_err_bad_decoding_bool!(v.len() == 32, "not return for word");
                    let v: [u8; 32] = v.try_into().unwrap();
                    (rc, U::from(v), None)
                } else {
                    (rc, U::ZERO, Some(v))
                }
            }

            /// Allocates a slice with a single u8 if a word was returned, or
            /// returns a Vec<u8> if something went wrong.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _bool_err_vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64
            ) -> (bool, Option<Vec<u8>>) {
                let (rc, rd_len) = [<$base_fn _partial>](contract, calldata, $($value_param,)? gas);
                if rc {
                    panic_on_err_bad_decoding_bool!(rd_len == 32, "not return for bool word");
                    let mut b = [0u8; 1];
                    unsafe {
                        impls::read_return_data(b.as_mut_ptr(), 31, 1);
                    }
                    (b[0] == 1, None)
                } else {
                    let mut b = Vec::with_capacity(rd_len);
                    unsafe {
                        impls::read_return_data(b.as_mut_ptr(), 0, rd_len);
                        b.set_len(rd_len);
                    }
                    (false, Some(b))
                }
            }

            /// Call a function, returning whether the call returned true if the
            /// contract returned something, or true if the contract returned nothing
            /// but does exist. The vector contains revertdata if the call was
            /// unsuccessful. Does not read returndata.
            #[cfg(feature = "alloc")]
            pub fn [<safe_ $base_fn _bool_err_vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64
            ) -> (bool, Option<Vec<u8>>) {
                if !addr_has_code(contract) {
                    return (false, None)
                }
                let (rc, rd_len) = [<$base_fn _partial>](contract, calldata, $($value_param,)? gas);
                match (rc, rd_len) {
                    (true, 32) => {
                        let mut b = [0u8; 1];
                        unsafe {
                            impls::read_return_data(b.as_mut_ptr(), 31, 1);
                        }
                        (b[0] == 1, None)
                    }
                    (true, 0) => (true, None),
                    (true, _) => panic_on_err_bad_decoding_bool!(
                        "word not returned for safe_ _bool_err_vec"
                    ),
                    (false, rd_len) => {
                        let mut b = Vec::with_capacity(rd_len);
                        unsafe {
                            impls::read_return_data(b.as_mut_ptr(), 0, rd_len);
                            b.set_len(rd_len);
                        }
                        (false, Some(b))
                    }
                }
            }

            /// Return if the call was successful, and the vector of the revertdata.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _unit_err_vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64
            ) -> (bool, Option<Vec<u8>>) {
                let (rc, rd_len) = [<$base_fn _partial>](contract, calldata, $($value_param,)? gas);
                if rc {
                    (true, None)
                } else {
                    let mut b = Vec::with_capacity(rd_len);
                    unsafe {
                        impls::read_return_data(b.as_mut_ptr(), 0, rd_len);
                        b.set_len(rd_len);
                    }
                    (false, Some(b))
                }
            }

            /// Result equivalent of _unit_err_vec.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _unit_err_res>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64
            ) -> Result<(), Vec<u8>> {
                match [<$base_fn _unit_err_vec>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas
                )
                {
                    (false, Some(e)) => Err(e),
                    (false, None) => {
                        // How did this happen?
                        unimplemented!()
                    }
                    (true, None) | (true, Some(_)) => Ok(())
                }
            }

            /// Call a function, returning whether the call was successful. The vector contains
            /// revertdata if the call was unsuccessful. Does not read returndata.
            #[cfg(feature = "alloc")]
            pub fn [<safe_ $base_fn _unit_err_vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64
            ) -> (bool, Option<Vec<u8>>) {
                if addr_has_code(contract) {
                    [<$base_fn _unit_err_vec>](contract, calldata, $($value_param,)? gas)
                } else {
                    (false, None)
                }
            }

            /// Return a U word using a Result, or the vector for an error.
            #[cfg(feature = "alloc")]
            pub fn [<$base_fn _word_err_vec_res>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
            ) -> Result<U, Vec<u8>> {
                let (rc, w, v) = [<$base_fn _word_err_vec>](
                    contract,
                    calldata,
                    $($value_param,)?
                    gas
                );
                if rc {
                    Ok(w)
                } else {
                    Err(v.expect("vec not containing anything"))
                }
            }

            /// Safely call a contract, with the same checks as the safe vec variant.
            #[cfg(feature = "alloc")]
            pub fn [<safe_ $base_fn _vec>](
                contract: Address,
                calldata: &[u8],
                $($value_param: $value_ty,)?
                gas: u64,
                offset: usize,
            ) -> Option<(bool, Vec<u8>)> {
                if addr_has_code(contract) {
                    Some([<$base_fn _vec>](contract, calldata, $($value_param,)? gas, offset))
                } else {
                    None
                }
            }
        }
    };
}

generate_call_variants!(call, has_value);
generate_call_variants!(static_call);
generate_call_variants!(delegate_call);

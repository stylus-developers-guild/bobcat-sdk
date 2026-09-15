#![no_main]
#![no_std]

use bobcat_sdk::{
    call::{call_unit, call_word_opt, safe_call_unit},
    cd::{EvmCdAddress, EvmCdDeserialise, EvmCdSerialise},
    entry::*,
    interfaces::{
        camelotv3_swap_router::make_fn_exact_input_single,
        eip20::{make_fn_approve, make_fn_transfer_from},
    },
    maths::U,
};

#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
    fn msg_reentrant() -> bool;
}

const SWAP_ROUTER: [u8; 20] =
    match const_hex::const_decode_to_array::<20>(b"6221a9c005f6e47eb398fd867784cacfdcfff4e7") {
        Ok(v) => v,
        Err(_) => panic!(),
    };

#[derive(Debug, Clone, EvmCdSerialise, EvmCdDeserialise)]
#[evm_entrypoint]
pub enum Entry {
    MakeSwap(EvmCdAddress, EvmCdAddress, U, U),
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    assert!(!unsafe { msg_reentrant() });
    match read_cd::<Entry>(args_len) {
        Entry::MakeSwap(token_in, token_out, amount_in, amount_out_min) => {
            let sender = msg_sender();
            assert!(
                safe_call_unit(
                    token_in.into(),
                    &make_fn_transfer_from(msg_sender(), contract_address(), &amount_in),
                    &U::ZERO,
                    u64::MAX
                ),
                "camelot swap router error"
            );
            assert!(
                call_unit(
                    token_in.into(),
                    &make_fn_approve(SWAP_ROUTER, &amount_in),
                    &U::ZERO,
                    u64::MAX
                ),
                "erc20 approve error"
            );
            let deadline = block_timestamp() + 1;
            let w = call_word_opt(
                SWAP_ROUTER,
                &make_fn_exact_input_single(
                    token_in.into_array(),
                    token_out.into_array(),
                    sender,
                    U::from(deadline),
                    amount_in,
                    amount_out_min,
                    [255u8; 20], // This is U160::MAX
                ),
                &U::ZERO,
                u64::MAX,
                0,
            )
            .unwrap();
            write_result_word(&w);
        }
    }
    0
}

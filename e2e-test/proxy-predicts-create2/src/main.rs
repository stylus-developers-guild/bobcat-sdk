#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

bobcat_allocator!();

const SEL_DEPLOY: [u8; 4] = const_keccak_sel(b"deploy(address)");
const SEL_PREDICT: [u8; 4] = const_keccak_sel(b"predict(address)");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { 4 + 32 });
    let addr: U = args[4..].try_into().unwrap();
    let code = make_metamorphic_proxy(addr.into());
    write_result_word(
        &match args[..4].try_into().unwrap() {
            SEL_DEPLOY => create2_pre_unit(&code, U::ZERO, &msg_sender()).unwrap(),
            SEL_PREDICT => {
                let exp = estimate_addr_pre(contract_address(), &code, &msg_sender());
                assert_eq!(
                    exp,
                    const_estimate_addr_pre(contract_address(), &code, &msg_sender())
                );
                assert_eq!(
                    exp,
                    const_estimate_addr_post(
                        contract_address(),
                        keccak256(&code),
                        keccak256(&msg_sender())
                    )
                );
                exp
            }
            _ => panic!("call not supported"),
        }
        .into(),
    );
    0
}

type Address = [u8; 20];

use bobcat_sdk::maths::U;

#[cfg(any(target_arch = "wasm32", target_arch = "riscv32"))]
mod impls {
    use super::*;

    use bobcat_sdk::{
        call::safe_call_bool_err_vec,
        interfaces::eip20::{make_fn_transfer, make_fn_transfer_from},
    };

    pub fn transfer_from(addr: Address, from: Address, to: Address, amt: &U) -> Option<()> {
        match safe_call_bool_err_vec(
            addr,
            &make_fn_transfer_from(from, to, amt),
            &U::ZERO,
            u64::MAX,
        ) {
            (true, _) => Some(()),
            _ => None,
        }
    }

    pub fn transfer(addr: Address, recipient: Address, amt: &U) -> Option<()> {
        match safe_call_bool_err_vec(addr, &make_fn_transfer(recipient, amt), &U::ZERO, u64::MAX) {
            (true, _) => Some(()),
            _ => None,
        }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "riscv32")))]
mod impls {
    use super::*;

    use std::{cell::RefCell, collections::HashMap};

    use bobcat_sdk::entry::contract_address;

    thread_local! {
        static BALANCES: RefCell<HashMap<Address, U>> = RefCell::default();
    }

    pub fn balance_of(spender: Address) -> U {
        BALANCES
            .with(|b| b.borrow().get(&spender).map(|x| *x))
            .unwrap_or_default()
    }

    pub fn transfer_from(_: Address, from: Address, to: Address, amt: &U) -> Option<()> {
        let amt = *amt;
        BALANCES.with(|b| {
            let mut b = b.borrow_mut();
            let from_bal = b.get_mut(&from)?;
            if *from_bal < amt {
                return None;
            }
            *from_bal -= amt;
            b.entry(to).and_modify(|v| *v += amt).or_insert(amt);
            Some(())
        })
    }

    pub fn transfer(addr: Address, recipient: Address, amt: &U) -> Option<()> {
        transfer_from(addr, contract_address(), recipient, amt)
    }

    pub fn clear() {
        BALANCES.with(|b| b.borrow_mut().clear())
    }

    pub fn give(recipient: Address, amt: U) {
        BALANCES.with(|b| {
            *b.borrow_mut()
                .entry(recipient)
                .and_modify(|v| *v += amt)
                .or_insert(amt)
        });
    }
}

pub use impls::*;

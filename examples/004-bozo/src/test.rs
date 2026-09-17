use proptest::prelude::*;

use bobcat_sdk::{
    entry::{contract_address, entry_host, msg_sender},
    maths::{
        U, mul_div_round_up,
        strategies::{strat_addr_not_empty, strat_small_u},
    },
    storage::storage_host,
};

use crate::{
    ADDR_OPERATOR, FEE_DAO, FEE_OWNER, SCALING_FACTOR, eip20, get_min_deposit, pick_epoch,
};

type Address = [u8; 20];

proptest! {
    #[test]
    fn test_get_min_deposit_similar_to_ref(pool_size in any::<u64>(), last_deposit in any::<u64>()) {
        let floor_pct = 0.005;
        let escalation_multiplier = 1.05;
        let cap_pct = 0.02;
        let floor = floor_pct * pool_size as f64;
        let escalation = escalation_multiplier * last_deposit as f64;
        let cap = cap_pct * pool_size as f64;
        let e = {
            let m = if escalation > cap { cap } else { escalation };
            if floor > m { floor } else { m }
        };
        let v = get_min_deposit(&U::from(pool_size), &U::from(last_deposit)).unwrap();
        let e = U::from(e as u64);
        let d = if e > v { e - v } else { v - e };
        assert!(U::from_u32(100) > d, "{e} != {v}");
    }

    #[test]
    fn test_solvencies(
        asset in any::<Address>(),
        epochs in proptest::collection::vec(
            (
                any::<U>(),
                proptest::collection::vec(
                    (strat_addr_not_empty(), strat_small_u(), any::<U>()),
                    1..10
                )
            ),
            1..10
        )
    ) {
        storage_host::storage_clear();
        eip20::clear();
        let owner = msg_sender();
        assert_eq!(0, crate::state_init(owner, asset));
        let mut pool = U::ZERO;
        for (i, (rng, users)) in epochs.into_iter().enumerate() {
            let epoch = U::from(i);
            let mut last_deposited = U::ZERO;
            let mut round_pool = U::ZERO;
            assert_eq!(epoch, pick_epoch().0);
            for (addr, amt, comment) in users.into_iter() {
                let amt = amt + get_min_deposit(&round_pool, &last_deposited).unwrap();
                let extra_fee_scale = SCALING_FACTOR + FEE_OWNER + FEE_DAO;
                let amt_with_fee = amt.mul_div_round_up(&extra_fee_scale, SCALING_FACTOR)
                        .unwrap();
                last_deposited = amt;
                round_pool += amt_with_fee;
                eip20::give(addr, amt_with_fee);
                entry_host::set_msg_sender(addr);
                assert_eq!(0, crate::state_play(amt_with_fee, addr, &comment, &i.into()));
            }
            pool += round_pool;
            // We set the deadline to one, and we mock out the current timestamp as 2
            // to trick the contract into thinking the time has expired.
            crate::storage::ts_deadline::set(&epoch, &U::ONE);
            entry_host::set_msg_sender(crate::ADDR_OPERATOR);
            entry_host::set_block_timestamp(2);
            assert_eq!(0, crate::state_distribute_rewards(&epoch, &rng));
        }
        let pool_fee = mul_div_round_up(&pool, &(FEE_OWNER + FEE_DAO), SCALING_FACTOR).unwrap();
        entry_host::set_msg_sender(owner);
        assert_eq!(0, crate::state_owner_collect_fees());
        assert_eq!(0, crate::state_dao_collect_fees());
        prop_assume!(crate::ADDR_OPERATOR != contract_address());
        let op_bal = eip20::balance_of(ADDR_OPERATOR);
        assert!(pool_fee.abs_diff(&op_bal) < U::from_u32(1000), "{pool_fee} != {}", op_bal);
    }
}

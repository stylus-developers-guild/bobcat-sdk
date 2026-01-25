#![cfg_attr(any(target_arch = "wasm32", target_arch = "riscv32"), no_std)]

use core::cmp::{max, min};

pub use bobcat_sdk;

use bobcat_sdk::{
    cd::{address, const_keccak_sel, read_words},
    entry::{
        block_timestamp, contract_address, msg_sender, read_args_safe, write_result_slice,
        write_result_word,
    },
    events::emit,
    interfaces::eip1967::{TOPIC_ADMIN_CHANGED, TOPIC_UPGRADED},
    maths::{u, U},
    storage::{
        const_keccak256, const_slot_off_curve, flush_guard, keccak256, reentrancy_guard_sel,
        storage_load, storage_store,
    },
};

use array_concat::concat_arrays;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

pub mod storage;

mod eip20;

#[cfg(test)]
mod test;

type Address = [u8; 20];

/// Slot that contains the admin functionality, for an admin to come in
/// and replace the implementation. May be set to zero to prevent this
/// contract from being upgraded.
const SLOT_ADMIN: U = const_slot_off_curve(b"eip1967.proxy.admin");

/// Slot that contains the implementation address for the proxy to use.
const SLOT_IMPL: U = const_slot_off_curve(b"eip1967.proxy.implementation");

/// Operator that's able to trigger the reset cron.
const ADDR_OPERATOR: [u8; 20] = address!(b"6221a9c005f6e47eb398fd867784cacfdcfff4e7");

/// Event emitted when a deposit is made.
const TOPIC_DEPOSIT_MADE: U = const_keccak256(b"DepositMade(address,uint256,uint256)");

/// Some points were collected by a user!
const TOPIC_POINTS_COLLECTED: U = const_keccak256(b"PointsCollected(address,uint256)");

/// A winner was chosen for a game!
const TOPIC_WINNER_CHOSEN: U = const_keccak256(b"WinnerChosen(address,uint256,bool)");

/// Event emitted when the epoch is bumped.
const TOPIC_NEW_EPOCH: U = const_keccak256(b"NewEpoch(uint256)");

/// Someone posted a comment when they played the game.
const TOPIC_COMMENT_POSTED: U = const_keccak256(b"CommentPosted(address,bytes32)");

const FLOOR_PCT: U = u!(5);

const ESCALATION_MULTIPLIER: U = u!(1050);

const CAP_PCT: U = u!(20);

const SCALING_FACTOR: U = u!(1000);

/// Owner fee taken from the users. 3% fee.
const FEE_OWNER: U = U::from_u32(30);

/// Fee paid to the DAO for the game. 2%.
const FEE_DAO: U = U::from_u32(20);

/// 40 minutes extra time.
const EXTRA_TIME: U = U::from_u32(2400);

// ~~~~~ View functions: ~~~~
//
const SEL_POOL_SIZE: [u8; 4] = const_keccak_sel(b"poolSize()");
const SEL_POOL_ASSET: [u8; 4] = const_keccak_sel(b"poolAsset()");
const SEL_LAST_BETTOR_AMOUNT: [u8; 4] = const_keccak_sel(b"lastBettorAmount()");
const SEL_LAST_BETTOR_ADDRESS: [u8; 4] = const_keccak_sel(b"lastBettorAddress()");
const SEL_MIN_DEPOSIT: [u8; 4] = const_keccak_sel(b"minDeposit()");
const SEL_DEADLINE: [u8; 4] = const_keccak_sel(b"deadline()");
const SEL_EPOCH_DEADLINE: [u8; 4] = const_keccak_sel(b"epochDeadline(uint256)");
const SEL_PLAYER_COUNT: [u8; 4] = const_keccak_sel(b"playerCount()");
const SEL_TICKET_COUNT: [u8; 4] = const_keccak_sel(b"ticketCount()");
const SEL_CURRENT_EPOCH: [u8; 4] = const_keccak_sel(b"currentEpoch()");
const SEL_WAS_EPOCH_COLLECTED: [u8; 4] = const_keccak_sel(b"wasEpochCollected(uint256)");

// ~~~~~ Stateful functions: ~~~~
//
const SEL_INIT: [u8; 4] = const_keccak_sel(b"initialise(address,address)");
const SEL_PLAY: [u8; 4] = const_keccak_sel(b"play(uint256,address,bytes32,uint256)");
const SEL_DISTRIBUTE_REWARDS: [u8; 4] =
    const_keccak_sel(b"distributeRewards(uint256,address,uint256)");
const SEL_UPGRADE: [u8; 4] = const_keccak_sel(b"upgrade(address)");
const SEL_CHANGE_ADMIN: [u8; 4] = const_keccak_sel(b"changeAdmin(address)");
const SEL_OWNER_COLLECT_FEES: [u8; 4] = const_keccak_sel(b"ownerCollectFees()");
const SEL_DAO_COLLECT_FEES: [u8; 4] = const_keccak_sel(b"daoCollectFees()");

fn view_deadline() -> usize {
    write_result_word(&storage::ts_deadline::get(&pick_epoch().0));
    0
}

fn view_deadline_epoch(epoch: U) -> usize {
    write_result_word(&storage::ts_deadline::get(&epoch));
    0
}

fn view_pool_size() -> usize {
    write_result_word(&storage::pool_size::get(&pick_epoch().0));
    0
}

fn view_pool_asset() -> usize {
    write_result_word(&U::from(storage::asset::get()));
    0
}

fn view_last_bettor_amount() -> usize {
    write_result_word(&storage::last_bettor_amt::get(&pick_epoch().0));
    0
}

fn view_last_bettor_address() -> usize {
    write_result_word(&storage::last_bettor_addr::get(&pick_epoch().0));
    0
}

pub(crate) fn get_min_deposit(pool_size: &U, last_deposit: &U) -> Option<U> {
    if last_deposit.is_zero() {
        return Some(U::ZERO);
    }
    let floor = pool_size.mul_div_round_up(&FLOOR_PCT, SCALING_FACTOR)?;
    let escalation = last_deposit.mul_div_round_up(&ESCALATION_MULTIPLIER, SCALING_FACTOR)?;
    let cap = CAP_PCT.mul_div_round_up(pool_size, SCALING_FACTOR)?;
    Some(max(floor, min(escalation, cap)))
}

fn view_min_deposit() -> usize {
    let epoch = pick_epoch().0;
    let v = get_min_deposit(
        &storage::pool_size::get(&epoch),
        &storage::last_bettor_amt::get(&epoch),
    )
    .unwrap();
    let extra_fee_scale = SCALING_FACTOR + FEE_OWNER + FEE_DAO;
    write_result_word(
        &v.mul_div_round_up(&extra_fee_scale, SCALING_FACTOR)
            .unwrap(),
    );
    0
}

fn view_player_count() -> usize {
    write_result_word(&storage::user_lottery_ticket_len::get(&pick_epoch().0));
    0
}

fn view_ticket_count() -> usize {
    write_result_word(&storage::global_tickets::get(&pick_epoch().0));
    0
}

fn view_current_epoch() -> usize {
    write_result_word(&pick_epoch().0);
    0
}

fn view_was_epoch_collected() -> usize {
    write_result_word(&storage::was_distributed::get(&pick_epoch().0));
    0
}

fn state_init(admin: Address, asset: Address) -> usize {
    core::assert!(storage::initialised::get().is_zero(), "already created");
    storage_store(&SLOT_ADMIN, &admin.into());
    storage::asset::set(&asset.into());
    storage::initialised::set(&U::from(true));
    0
}

// Get the current epoch by comparing the timestamp with the deadline. If
// we've exceeded the deadline, we return the next epoch, and a flag to
// indicate it should be set to the result returned from this function.
fn pick_epoch() -> (U, bool) {
    let e = storage::epoch::get();
    let x = storage::ts_deadline::get(&e).into();
    if block_timestamp() > x && x != 0 {
        (e + U::ONE, true)
    } else {
        (e, false)
    }
}

fn state_play(amt: U, recipient: Address, comment: &U, preferred_epoch: &U) -> usize {
    core::assert!(amt.is_some(), "amount is zero");
    core::assert!(recipient != [0u8; 20], "recipient is zero");
    let timestamp = U::from(block_timestamp());
    let (epoch, needs_epoch_setting) = pick_epoch();
    core::assert_eq!(epoch, *preferred_epoch, "epoch not the preferred one");
    if needs_epoch_setting {
        // If we've exceeded the timestamp, we need to set a new epoch.
        storage::epoch::set(&epoch);
        emit!(TOPIC_NEW_EPOCH, epoch);
    }
    if comment.is_some() {
        emit!(TOPIC_COMMENT_POSTED, recipient, comment);
    }
    let addr_asset: Address = storage::asset::get().into();
    // Transfer the asset to us:
    eip20::transfer_from(addr_asset, msg_sender(), contract_address(), &amt).unwrap();
    let owner_fee_paid = amt.mul_div_round_up(&FEE_OWNER, SCALING_FACTOR).unwrap();
    let dao_fee_paid = amt.mul_div_round_up(&FEE_DAO, SCALING_FACTOR).unwrap();
    let amt = amt.checked_sub(&owner_fee_paid).checked_sub(&dao_fee_paid);
    let pool_size = storage::pool_size::get(&epoch);
    let last_bettor_amt = storage::last_bettor_amt::get(&epoch);
    // Get the amount that the user has to beat to play the game next:
    let extra_amt = get_min_deposit(&pool_size, &last_bettor_amt).unwrap();
    core::assert!(
        amt >= extra_amt,
        "amount not enough: {extra_amt} needed, {amt} provided. diff: {}",
        amt.abs_diff(&extra_amt)
    );
    // Figure out how many "lottery tickets" to give the user -- aka, the
    // chance of them winning 20% of the prize without actually being the
    // one to win.
    let lottery_tickets = if U::from(100u32) > storage::early_participants::get(&epoch) {
        // Since we have less than 100 participants, give the user extra tickets!
        amt.mul_div(&U::from(190u32), U::from(100u32)).unwrap().0
    } else {
        amt
    };
    let recipient = U::from(recipient);
    storage::last_bettor_addr::set(&epoch, &recipient);
    storage::last_bettor_amt::set(&epoch, &amt);
    storage::owner_fees_collected::add(&owner_fee_paid);
    storage::dao_fees_collected::add(&dao_fee_paid);
    storage::pool_size::set(&epoch, &(pool_size + amt));
    storage::early_participants::add(&epoch, &U::ONE);
    storage::global_tickets::add(&epoch, &lottery_tickets);
    let packed_pos = storage::user_lottery_pos::get(&epoch, &recipient);
    // We get a new position if the user has never had one, then we add the
    // tickets there.
    let has_played = packed_pos[0] >= 1;
    let mut pos = packed_pos;
    pos[0] = 0;
    if !has_played {
        // If this is the first time that the recipient is playing, we need to track them:
        let ticket_len = storage::user_lottery_ticket_len::get(&epoch);
        storage::user_lottery_addresses::set(&epoch, &ticket_len, &recipient);
        pos = ticket_len;
        // The position in this epoch will never exceed a u64, so this is safe:
        let mut packed_pos = pos;
        // If the value here is anything other than 0, we're going to ignore it.
        packed_pos[0] = if packed_pos[0] == 0 { 1 } else { packed_pos[0] };
        storage::user_lottery_pos::set(&epoch, &recipient, &packed_pos);
        storage::user_lottery_ticket_len::set(&epoch, &(pos + U::ONE));
    }
    storage::user_lottery_tickets::add(&epoch, &pos, &lottery_tickets).unwrap();
    emit!(TOPIC_POINTS_COLLECTED, recipient, lottery_tickets);
    storage::points_collected::add(&recipient, &lottery_tickets);
    emit!(
        TOPIC_DEPOSIT_MADE,
        recipient,
        amt,
        storage::pool_size::get(&epoch)
    );
    storage::ts_deadline::set(&epoch, &(timestamp + EXTRA_TIME));
    let r: [u8; 32 * 2] = concat_arrays!(epoch.0, amt.0);
    write_result_slice(&r);
    0
}

fn state_distribute_rewards(epoch: &U, rng: &U) -> usize {
    core::assert_eq!(ADDR_OPERATOR, msg_sender(), "operator only");
    let deadline = storage::ts_deadline::get(&epoch);
    core::assert!(
        U::from(block_timestamp()) > deadline && deadline.is_some(),
        "not concluded"
    );
    // Take 80% of the pool, and send to the winning depositor:
    let last_bettor_addr: Address = storage::last_bettor_addr::get(&epoch).into();
    let ticket_count = storage::global_tickets::get(&epoch)
        - storage::user_lottery_tickets::get(&epoch, &last_bettor_addr.into());
    let addr_asset: Address = storage::asset::get().into();
    let full_pool = storage::pool_size::get(&epoch);
    if ticket_count.is_zero() {
        // We only had one player! Let's transfer them the full amount, and stop.
        if last_bettor_addr != [0u8; 20] {
            eip20::transfer(addr_asset, last_bettor_addr, &full_pool).unwrap();
            emit!(TOPIC_WINNER_CHOSEN, last_bettor_addr, full_pool, false);
        }
        write_result_word(&full_pool);
        return 0;
    }
    // If we had more than one player, we give the top 80% to the last user:
    let winner_reward = full_pool.mul_div(&U::from(8u32), U::from(10u32)).unwrap().0;
    eip20::transfer(addr_asset, last_bettor_addr, &winner_reward).unwrap();
    emit!(TOPIC_WINNER_CHOSEN, last_bettor_addr, winner_reward, false);
    // Using the random word, we start to pick some random words using
    // keccak. We're only ever going to see 10 winners at max, since we
    // divide the winnings up to at most 10 people. We take the 20%:
    let full_lottery_reward = full_pool.mul_div(&U::from(2u32), U::from(10u32)).unwrap().0;
    let ticket_len: usize = storage::user_lottery_ticket_len::get(&epoch).into();
    let max_winners = min(ticket_len, 10usize);
    let user_lottery_reward = full_lottery_reward / U::from(max_winners);
    let mut winners = [[0u8; 20]; 10];
    let mut i = 0usize;
    while max_winners > i {
        let rng_preimage: [u8; 32 + size_of::<usize>()] = concat_arrays!(rng.0, i.to_be_bytes());
        let rng = keccak256(&rng_preimage);
        // The amount outstanding that we can distribute here. We'll reduce this
        // until it's zero, then do the reward. We reduce this so we reduce the amount
        // of gas that's used here:
        let mut leftover_tickets = rng % (ticket_count / SCALING_FACTOR);
        // We prefer to use the storage, though we could actually just load this
        // in ourselves. We'll repeatedly use the storage cache instead to get
        // the elements we want.
        let player_count: usize = storage::user_lottery_ticket_len::get(&epoch).into();
        // The size of the players won't exceed usize, which should be u32 in our wasm.
        let mut p: usize = ((rng >> 64) % U::from(player_count)).into();
        loop {
            let p_u = U::from(p);
            let user_amt = storage::user_lottery_tickets::get(&epoch, &p_u);
            if user_amt >= leftover_tickets {
                let w: Address = storage::user_lottery_addresses::get(&epoch, &p_u).into();
                // Instead of swap/popping the storage, we do a search over the slice:
                let has_user_won_already = winners.contains(&w);
                if !has_user_won_already {
                    winners[i] = w;
                    // Transfer the winner their amount:
                    eip20::transfer(addr_asset, w, &user_lottery_reward).unwrap();
                    emit!(TOPIC_WINNER_CHOSEN, w, user_lottery_reward, true);
                    break;
                }
            }
            // We do a saturating sub so that if we saw the winner before, we set to
            // 0 safely:
            leftover_tickets = leftover_tickets.saturating_sub(&user_amt);
            p += 1;
            if p >= player_count {
                p = 0;
            }
        }
        i += 1;
    }
    storage::was_distributed::set(&epoch, &U::from(true));
    let r: [u8; 32 * 3] = concat_arrays!([0u8; 32], U::from(64u32).0, [0u8; 32]);
    write_result_slice(&r);
    0
}

fn state_upgrade(new_impl: Address) -> usize {
    core::assert_eq!(storage_load(&SLOT_ADMIN), msg_sender().into());
    storage_store(&SLOT_IMPL, &U::from(new_impl));
    emit!(TOPIC_UPGRADED, new_impl);
    0
}

fn state_change_admin(new_admin: Address) -> usize {
    let last_admin = storage_load(&SLOT_ADMIN);
    core::assert_eq!(last_admin, msg_sender().into());
    storage_store(&SLOT_ADMIN, &U::from(new_admin));
    emit!(TOPIC_ADMIN_CHANGED, last_admin, new_admin);
    0
}

fn state_owner_collect_fees() -> usize {
    let f = storage::owner_fees_collected::get();
    eip20::transfer(storage::asset::get().into(), ADDR_OPERATOR, &f).unwrap();
    storage::owner_fees_collected::clear();
    write_result_word(&f);
    0
}

fn state_dao_collect_fees() -> usize {
    let f = storage::dao_fees_collected::get();
    eip20::transfer(storage::asset::get().into(), ADDR_OPERATOR, &f).unwrap();
    storage::dao_fees_collected::clear();
    write_result_word(&f);
    0
}

pub fn user_entrypoint(args_len: usize) -> usize {
    // Allocate the full amount that we will see possibly:
    let args = &read_args_safe!(args_len, { 32 * 5 + 4 });
    let sel: [u8; 4] = args[..4].try_into().unwrap();
    flush_guard(|| {
        match sel {
            // View functions:
            SEL_DEADLINE => view_deadline(),
            SEL_EPOCH_DEADLINE => {
                let epoch = read_words!(&args[4..], 1);
                view_deadline_epoch(*epoch)
            }
            SEL_POOL_SIZE => view_pool_size(),
            SEL_POOL_ASSET => view_pool_asset(),
            SEL_LAST_BETTOR_AMOUNT => view_last_bettor_amount(),
            SEL_LAST_BETTOR_ADDRESS => view_last_bettor_address(),
            SEL_MIN_DEPOSIT => view_min_deposit(),
            SEL_PLAYER_COUNT => view_player_count(),
            SEL_TICKET_COUNT => view_ticket_count(),
            SEL_CURRENT_EPOCH => view_current_epoch(),
            SEL_WAS_EPOCH_COLLECTED => view_was_epoch_collected(),
            // Side effect generating functions:
            SEL_INIT => {
                let (admin, asset) = read_words!(&args[4..], 2);
                state_init(admin.into(), asset.into())
            }
            SEL_PLAY => reentrancy_guard_sel(&SEL_PLAY, || {
                let (amt, recipient, comment, epoch) = read_words!(&args[4..], 4);
                state_play(*amt, recipient.into(), comment, epoch)
            }),
            SEL_DISTRIBUTE_REWARDS => {
                let (epoch, _recipient, rng) = read_words!(&args[4..], 3);
                state_distribute_rewards(epoch, rng)
            }
            SEL_UPGRADE => {
                let new_impl = read_words!(&args[4..], 1);
                state_upgrade(new_impl.into())
            }
            SEL_CHANGE_ADMIN => {
                let new_admin = read_words!(&args[4..], 1);
                state_change_admin(new_admin.into())
            }
            SEL_OWNER_COLLECT_FEES => state_owner_collect_fees(),
            SEL_DAO_COLLECT_FEES => state_dao_collect_fees(),
            _ => 1,
        }
    })
}

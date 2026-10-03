//! Mutation test of the non-interference test (doc 04 section 7.2 item 7). Each deliberate leak
//! must make `run_pair` fail on at least one seed; with all leaks off nothing fails.
#![cfg(feature = "canary")]

use mtg_cards::testpool;
use mtg_core::canary::*;
use mtg_core::ids::Seat;
use mtg_debug::noninterference::*;
use mtg_view::DeckList;

fn fails(mask: u32, own_library: bool) -> usize {
    set(mask);
    let pool = testpool::build();
    let a = DeckList { main: pool.deck_red_green(), side: vec![] };
    let b = DeckList { main: pool.deck_white_black(), side: vec![] };
    let mut n = 0;
    for seed in 0..120u64 {
        for observer in [Seat(0), Seat(1)] {
            let cfg = NiConfig { observer, own_library, depth: 20 + (seed * 37) % 220, max_steps: 400 };
            if run_pair(&pool.db, [&a, &b], seed, cfg).is_err() {
                n += 1;
            }
        }
    }
    set(0);
    n
}

#[test]
fn every_canary_is_caught_and_clean_run_passes() {
    assert_eq!(fails(0, false), 0, "clean build must pass");
    assert_eq!(fails(0, true), 0, "clean build must pass (own library)");
    for (name, bit, own) in [
        ("viewid_from_cardid", VIEWID_FROM_CARDID, false),
        ("option_order_by_cardid", OPTION_ORDER_BY_CARDID, false),
        ("hash_over_state", HASH_OVER_STATE, false),
        ("error_names_card", ERROR_NAMES_CARD, false),
    ] {
        let n = fails(bit, own);
        println!("canary {name}: caught on {n} of 240 runs");
        assert!(n > 0, "canary {name} was NOT caught: the non-interference test is too weak");
    }
}

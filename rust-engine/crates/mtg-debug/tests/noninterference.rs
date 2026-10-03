use mtg_cards::testpool;
use mtg_debug::noninterference::*;
use mtg_core::ids::Seat;
use mtg_view::DeckList;

fn run_all(own_library: bool, games: u64) -> (u64, u64, Vec<NiFail>) {
    let pool = testpool::build();
    let a = DeckList { main: pool.deck_red_green(), side: vec![] };
    let b = DeckList { main: pool.deck_white_black(), side: vec![] };
    let (c1, d1, f1) = run_decks(&pool, &a, &b, own_library, games);
    let t = DeckList { main: pool.deck_tricks(), side: vec![] };
    let u = DeckList { main: pool.deck_blue(), side: vec![] };
    let (c2, d2, f2) = run_decks(&pool, &t, &u, own_library, games / 2);
    let g = DeckList { main: pool.deck_triggers(), side: vec![] };
    let (c3, d3, f3) = run_decks(&pool, &g, &u, own_library, games / 2);
    (c1 + c2 + c3, d1 + d2 + d3, f1.into_iter().chain(f2).chain(f3).collect())
}

fn run_decks(pool: &testpool::TestPool, a: &DeckList, b: &DeckList, own_library: bool, games: u64) -> (u64, u64, Vec<NiFail>) {
    let (mut compared, mut differing, mut fails) = (0, 0, Vec::new());
    for seed in 0..games {
        for observer in [Seat(0), Seat(1)] {
            let cfg = NiConfig { observer, own_library, depth: 20 + (seed * 37) % 220, max_steps: 400 };
            if hidden_differs(&pool.db, [a, b], seed, cfg) {
                differing += 1;
            }
            match run_pair(&pool.db, [a, b], seed, cfg) {
                Ok(n) => compared += n,
                Err(f) => fails.push(f),
            }
        }
    }
    (compared, differing, fails)
}

#[test]
fn opponent_hidden_state_does_not_interfere() {
    let (compared, differing, fails) = run_all(false, 300);
    assert!(fails.is_empty(), "{} failures, first: seed {} step {}: {}", fails.len(), fails[0].seed, fails[0].step, fails[0].what);
    assert!(differing > 500, "worlds rarely differed ({differing}); the test is vacuous");
    assert!(compared > 20_000, "too few comparisons: {compared}");
}

#[test]
fn own_library_order_does_not_interfere_before_next_draw() {
    let (compared, differing, fails) = run_all(true, 300);
    assert!(fails.is_empty(), "{} failures, first: seed {} step {}: {}", fails.len(), fails[0].seed, fails[0].step, fails[0].what);
    assert!(differing > 500, "worlds rarely differed ({differing})");
    assert!(compared > 2_000, "too few comparisons: {compared}");
}

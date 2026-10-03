//! Golden replays: stored games must replay to the stored hashes bit-for-bit (doc 04 invariant I10).
//! A failure means the RNG, the rules, or the canonical option order changed: a CORE CHANGE.

use mtg_cards::testpool;
use mtg_view::{replay, GameRecord};

#[test]
fn goldens_replay_bit_identically() {
    let pool = testpool::build();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../goldens");
    let mut n = 0;
    for e in std::fs::read_dir(dir).expect("goldens dir") {
        let p = e.unwrap().path();
        if p.extension().map(|x| x == "rec").unwrap_or(false) {
            let rec = GameRecord::from_text(&std::fs::read_to_string(&p).unwrap()).unwrap();
            let g = replay(pool.db.clone(), &rec).unwrap_or_else(|e| panic!("{}: {e:?}", p.display()));
            assert_eq!(g.state_hash(), rec.checkpoints.last().unwrap().1, "{}", p.display());
            n += 1;
        }
    }
    assert!(n >= 20, "expected the golden set, found {n}");
}

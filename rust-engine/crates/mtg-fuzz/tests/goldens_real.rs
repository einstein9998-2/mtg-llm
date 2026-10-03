//! Real-pool golden replays (doc 04 invariant I10) against a pinned snapshot of the card text.
//! A failure means the RNG, the rules machinery or the canonical option order changed: a CORE
//! CHANGE, to be reviewed and then regenerated with the `goldens_real` binary.

use mtg_view::{replay, GameRecord};

#[test]
fn real_pool_goldens_replay_bit_identically() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../goldens-real");
    let snapshot = std::fs::read_to_string(dir.join("legacy.cards.snapshot.ron")).expect("pinned card snapshot");
    let db = mtg_fuzz::build_legacy_from_text(&snapshot);
    let (mut n, mut checkpoints) = (0, 0);
    let mut seen_decks = std::collections::BTreeSet::new();
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map(|x| x == "rec").unwrap_or(false) {
            let rec = GameRecord::from_text(&std::fs::read_to_string(&p).unwrap()).unwrap();
            assert!(rec.checkpoints.len() >= 2, "{}: a record without periodic checkpoints verifies too little", p.display());
            checkpoints += rec.checkpoints.len();
            let g = replay(db.clone(), &rec).unwrap_or_else(|e| panic!("{}: {e:?}", p.display()));
            assert_eq!(g.state_hash(), rec.checkpoints.last().unwrap().1, "{}", p.display());
            seen_decks.insert(rec.decks[0].clone());
            seen_decks.insert(rec.decks[1].clone());
            n += 1;
        }
    }
    assert!(n >= 16, "expected the real-pool golden set, found {n}");
    assert_eq!(seen_decks.len(), 8, "every one of the eight decks appears");
    eprintln!("{n} real-pool goldens, {checkpoints} checkpoints");
}

/// The live card text and the pinned snapshot are allowed to differ (card edits do not invalidate
/// the goldens), but a stale snapshot is worth knowing about.
#[test]
fn snapshot_drift_is_reported() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../goldens-real");
    let snapshot = std::fs::read_to_string(dir.join("legacy.cards.snapshot.ron")).unwrap();
    if snapshot != mtg_cards::legacy::LEGACY_CARDS {
        eprintln!("note: the live card text differs from the pinned snapshot; regenerate goldens-real only with a reviewed core change");
    }
}

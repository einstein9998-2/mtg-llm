//! Random-game fuzz over the eight real Legacy decks (kept short; see the legacyfuzz binary for long runs).
use mtg_cards::testpool::TestPool;
use mtg_fuzz::*;

#[test]
fn legacy_decks_random_games_hold_invariants() {
    let pool = TestPool { db: mtg_cards::legacy::build() };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decks");
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let mut decks = vec![];
    for p in &paths {
        let (d, missing) = load_deck_file(&pool.db, p);
        assert!(missing.is_empty(), "{}: unknown cards {missing:?}", p.display());
        assert_eq!(d.main.len(), 60, "{}", p.display());
        decks.push(d);
    }
    assert_eq!(decks.len(), 8);
    let n = decks.len() as u64;
    let mut stats = FuzzStats { games: 0, decisions: 0, engine_steps: 0, wins: [0; 2], draws: 0, truncated: 0, coverage: Default::default() };
    for i in 0..48u64 {
        let a = (i % n) as usize;
        let b = ((a as u64 + 1 + (i / n) % (n - 1)) % n) as usize;
        let seed = 7000 + i;
        if i % 6 == 0 {
            record_and_replay(&pool, [&decks[a], &decks[b]], seed, (i % 2) as u8).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        }
        if let Err(v) = play_random_game(&pool, [&decks[a], &decks[b]], seed, (i % 2) as u8, 1, if i % 8 == 0 { 60 } else { 0 }, 20000, &mut stats) {
            panic!("game {i} seed {}: {}", v.seed, v.detail);
        }
    }
}

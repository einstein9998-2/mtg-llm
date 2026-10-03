//! Reviewer determinism probe: digest of (state hash, view hashes, options count) per decision.
use mtg_core::decision::Status;
use mtg_core::hash::Fx64;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::{DeckList, Game};
use std::hash::{Hash, Hasher};
type Db = std::sync::Arc<mtg_core::card::CardDb>;
fn decks() -> (Db, Vec<DeckList>) {
    let db = mtg_cards::legacy::build();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decks");
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let d = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    (db, d)
}
#[test]
fn digest() {
    let (db, d) = decks();
    let n: u64 = std::env::var("DET_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(200);
    let from: u64 = std::env::var("DET_FROM").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut all = Fx64::default();
    let mut decisions = 0u64;
    for seed in from..from + n {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed / 8 + seed + 3) % 8) as usize]);
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0xD37);
        let mut h = Fx64::default();
        let mut k = 0u32;
        loop {
            match g.advance() {
                Status::GameOver(r) => { format!("{r:?}").hash(&mut h); break; }
                Status::NeedDecision(_) => {
                    g.state_hash().hash(&mut h);
                    let p = g.pending().unwrap();
                    let (id, seat, len) = (p.id, p.seat, p.options.len());
                    for s in [Seat::P0, Seat::P1] { g.observe(s).view_hash.hash(&mut h); }
                    (id, seat.0, len).hash(&mut h);
                    let i = rng.below(len as u64) as usize;
                    g.apply(id, i).unwrap();
                    k += 1; decisions += 1;
                    if k > 4000 { break; }
                }
            }
        }
        println!("GAME {seed} {:016x}", h.finish());
        h.finish().hash(&mut all);
    }
    println!("DIGEST {:016x} decisions {decisions}", all.finish());
}

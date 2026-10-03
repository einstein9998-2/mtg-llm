//! Reviewer probes for core-frozen-m5 (scratch copy only).
#[path = "ni_copy.rs"]
mod ni;
use mtg_core::ids::Seat;
use mtg_fuzz::load_deck_file;
use mtg_view::DeckList;
use std::sync::atomic::Ordering;
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
fn quarry_ni() {
    let (db, d) = decks();
    let boros = 1usize; // sorted: alurentell, boros-aggro, ...
    let games: u64 = std::env::var("Q_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let (mut pairs, mut compared, mut fails) = (0u64, 0u64, Vec::new());
    let mut stops = std::collections::BTreeMap::new();
    for seed in 0..games {
        let other = (seed as usize * 3 + 2) % 8;
        for (ai, bi) in [(boros, other), (other, boros)] {
            for observer in [Seat::P0, Seat::P1] {
                let cfg = ni::NiConfig { observer, own_library: false, depth: 40 + (seed * 53) % 400, max_steps: 1500 };
                pairs += 1;
                match ni::run_pair_ex(&db, [&d[ai], &d[bi]], seed, cfg) {
                    Ok((n, why)) => { compared += n; *stops.entry(why).or_insert(0u32) += 1; }
                    Err(f) => fails.push(format!("a={ai} b={bi} obs={} seed {} step {}: {}", observer.0, f.seed, f.step, f.what)),
                }
            }
        }
    }
    eprintln!("quarry NI: pairs {pairs} steps {compared} fails {} quarry taps {} quarry taps with a graveyard move {}", fails.len(), ni::QTAPS.load(Ordering::Relaxed), ni::QSAC.load(Ordering::Relaxed));
    eprintln!("stops {stops:#?}");
    if let Some(f) = fails.first() { eprintln!("{f}"); }
    assert!(fails.is_empty());
}

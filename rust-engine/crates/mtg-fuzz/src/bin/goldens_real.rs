//! Regenerates `goldens-real/`: a pinned snapshot of the Legacy card text plus one recorded random
//! game per ordered deck pairing, replayed against the snapshot (never the live file) so that card
//! edits do not invalidate them but any change to the RNG, the rules machinery or the canonical
//! option order does. Only run as part of a reviewed core change.
fn main() {
    let live = mtg_cards::legacy::LEGACY_CARDS;
    std::fs::create_dir_all("goldens-real").unwrap();
    std::fs::write("goldens-real/legacy.cards.snapshot.ron", live).unwrap();
    let db = mtg_fuzz::build_legacy_from_text(live);
    let decks = mtg_fuzz::load_real_decks(&db, std::path::Path::new("decks"));
    let mut n = 0;
    for (i, (na, a)) in decks.iter().enumerate() {
        for k in [3usize, 5] {
            let j = (i + k) % decks.len();
            let (nb, b) = &decks[j];
            let seed = 500 + (i * 10 + k) as u64;
            let rec = mtg_fuzz::record_and_replay_db(&db, [a, b], seed, (seed % 2) as u8).expect("record");
            std::fs::write(format!("goldens-real/r{n:02}-{na}-vs-{nb}.rec"), rec.to_text()).unwrap();
            println!("r{n:02} {na} vs {nb}: {} actions", rec.actions.len());
            n += 1;
        }
    }
}

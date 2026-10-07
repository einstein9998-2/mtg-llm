//! Re-targets a trained net to a card database with more (or reordered) cards, by card name.
//! Usage: netremap <old card snapshot .ron> <old net.bin> <out net.bin>
//! The old snapshot is the card text the net was trained on (e.g. goldens-real/legacy.cards.snapshot.ron
//! for nets from engine v5); the new database is `mtg_cards::legacy::build()`. New cards get zero weights.
use mtg_agent::Net;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let old = std::sync::Arc::new(mtg_dsl::build_db(&[("legacy.cards.ron", &std::fs::read_to_string(&a[1]).unwrap())]).unwrap_or_else(|e| panic!("snapshot: {e}")));
    let new = mtg_cards::legacy::build();
    let net = Net::load(std::path::Path::new(&a[2])).unwrap();
    let map: Vec<usize> = old.defs.iter().map(|d| new.id(&d.name).unwrap_or_else(|| panic!("card {} is gone from the new database", d.name)).0 as usize).collect();
    // Names must identify a def: refuse a database where two defs share a name and were merged.
    let mut seen = std::collections::BTreeSet::new();
    for (i, &n) in map.iter().enumerate() {
        if !seen.insert(n) {
            panic!("old defs {} and another both map to new def {n}", old.defs[i].name);
        }
    }
    let out = net.remap_defs(&map, new.defs.len()).unwrap();
    std::fs::write(&a[3], out.to_bytes()).unwrap();
    let moved = map.iter().enumerate().filter(|(i, &n)| *i != n).count();
    println!("{} old defs -> {} new defs ({} changed index), state_len {} -> {}", old.defs.len(), new.defs.len(), moved, net.state_len, out.state_len);
}

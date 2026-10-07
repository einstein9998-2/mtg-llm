//! Re-targets a trained net to the current card database when the old database is only known by its
//! def names, one per line in def order (`dumpdefs` of the engine the net was trained on).
//! Usage: netremap_names <old defs.txt> <old net.bin> <out net.bin>
//! New cards get zero weights (the net ignores them), exactly as `netremap` does.
use mtg_agent::Net;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let old: Vec<String> = std::fs::read_to_string(&a[1]).unwrap().lines().map(|s| s.to_string()).collect();
    let new = mtg_cards::legacy::build();
    let net = Net::load(std::path::Path::new(&a[2])).unwrap();
    let map: Vec<usize> = old.iter().map(|n| new.id(n).unwrap_or_else(|| panic!("card {n} is gone from the new database")).0 as usize).collect();
    let mut seen = std::collections::BTreeSet::new();
    for (i, &n) in map.iter().enumerate() {
        if !seen.insert(n) {
            panic!("old def {} maps to a def already taken ({n})", old[i]);
        }
    }
    let out = net.remap_defs(&map, new.defs.len()).unwrap();
    std::fs::write(&a[3], out.to_bytes()).unwrap();
    let moved = map.iter().enumerate().filter(|(i, &n)| *i != n).count();
    println!("{} old defs -> {} new defs ({} changed index), state_len {} -> {}", old.len(), new.defs.len(), moved, net.state_len, out.state_len);
}

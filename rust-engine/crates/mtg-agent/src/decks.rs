//! Deck files: "<count> <card name>" per line, a blank line before the sideboard, '#' comments.

use mtg_core::card::CardDb;
use mtg_view::DeckList;

pub fn load_deck(db: &CardDb, path: &std::path::Path) -> DeckList {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (mut main, mut side, mut in_side, mut seen) = (vec![], vec![], false, false);
    for l in text.lines().map(str::trim) {
        if l.starts_with('#') {
            continue;
        }
        if l.is_empty() {
            in_side |= seen;
            continue;
        }
        let (n, name) = l.split_once(' ').expect("count and card name");
        let id = db.id(name.trim()).unwrap_or_else(|| panic!("unknown card {name}"));
        for _ in 0..n.trim_end_matches('x').parse::<usize>().expect("count") {
            if in_side { side.push(id) } else { main.push(id) }
        }
        seen |= !in_side;
    }
    DeckList { main, side }
}

/// Every `*.txt` deck in `dir`, sorted by file name, with its name.
pub fn load_deck_dir(db: &CardDb, dir: &str) -> (Vec<String>, Vec<DeckList>) {
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{dir}: {e}")).filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    (paths.iter().map(|p| p.file_stem().unwrap().to_string_lossy().to_string()).collect(), paths.iter().map(|p| load_deck(db, p)).collect())
}

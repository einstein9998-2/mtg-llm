//! Regenerates `goldens/*.rec`. Only run as part of a reviewed core change (doc 01 section 15).
use mtg_cards::testpool;
use mtg_view::DeckList;

fn main() {
    let pool = testpool::build();
    let a = DeckList { main: pool.deck_red_green(), side: vec![] };
    let b = DeckList { main: pool.deck_white_black(), side: vec![] };
    let c = DeckList { main: pool.deck_blue(), side: vec![] };
    let g = DeckList { main: pool.deck_triggers(), side: vec![] };
    let t = DeckList { main: pool.deck_tricks(), side: vec![] };
    std::fs::create_dir_all("goldens").unwrap();
    for seed in 1..=20u64 {
        let decks = match seed {
            13 => [&t, &c],
            14 => [&a, &t],
            15 => [&t, &b],
            16 => [&t, &t],
            17 => [&g, &a],
            18 => [&g, &g],
            19 => [&b, &g],
            20 => [&t, &g],
            _ => match seed % 3 {
                0 => [&a, &b],
                1 => [&b, &c],
                _ => [&c, &a],
            },
        };
        let rec = mtg_fuzz::record_and_replay(&pool, decks, seed, (seed % 2) as u8).expect("record");
        std::fs::write(format!("goldens/g{seed:02}.rec"), rec.to_text()).unwrap();
        println!("g{seed:02}: {} actions", rec.actions.len());
    }
}

//! Self-play data dump for offline training: one JSON line per non-trivial decision with the
//! acting seat's sparse state encoding, option features, the chosen index and (filled in at game
//! end) the result for that seat. Usage: selfplay <decks dir> <games> <out.jsonl> [random|mc] [mc samples]
use mtg_core::decision::{GameResult, Status};
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::*;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = a.get(1).map(String::as_str).unwrap_or("decks");
    let games: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let out = a.get(3).map(String::as_str).unwrap_or("/tmp/selfplay.jsonl");
    let use_mc = a.get(4).map_or(false, |s| s == "mc");
    let samples: u32 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(4);
    let db = mtg_cards::legacy::build();
    let n_defs = db.defs.len();
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let decks: Vec<DeckList> = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    let mut f = std::io::BufWriter::new(std::fs::File::create(out).unwrap());
    let mut rows = 0u64;
    let mut buf = Vec::new();
    for i in 0..games {
        let (da, dbk) = (&decks[(i % 8) as usize], &decks[((i * 3 + 1 + i / 8) % 8) as usize]);
        let dl = [da, dbk];
        let mut g = Game::new(db.clone(), dl, 7000 + i, GameConfig { first_player: Seat((i % 2) as u8), ..GameConfig::default() });
        g.set_track_view_events(true);
        let model = UniformConsistentModel { decks: [da.main.clone(), dbk.main.clone()] };
        let mut mc = FlatMc::new(&model, samples, 3000, i);
        let mut rnd = RandomPolicy::new(i + 11);
        let mut lines: Vec<(Seat, String)> = Vec::new();
        let result = loop {
            match g.advance() {
                Status::GameOver(r) => break r,
                Status::NeedDecision(_) => {
                    let (id, seat, n) = {
                        let p = g.pending().unwrap();
                        (p.id, p.seat, p.options.len())
                    };
                    let idx = if use_mc { mc.choose(&g.seat_view(seat), n) } else { rnd.choose(&g.seat_view(seat), n) };
                    if n > 1 {
                        let o = g.observe(seat);
                        encode_state(&o, n_defs, &mut buf);
                        let sparse: Vec<String> = buf.iter().enumerate().filter(|(_, v)| **v != 0.0).map(|(i, v)| format!("[{i},{v}]")).collect();
                        let opts: Vec<String> = encode_options(&o).iter().map(|x| format!("[{},{},{},{},{}]", x.kind, x.decision, x.subject_def, x.subject_zone, x.value)).collect();
                        lines.push((seat, format!("{{\"game\":{i},\"seat\":{},\"state\":[{}],\"options\":[{}],\"chosen\":{idx}", seat.idx(), sparse.join(","), opts.join(","))));
                    }
                    g.apply(id, idx).unwrap();
                    if g.raw_state().steps() > 4_000_000 {
                        break GameResult::Draw;
                    }
                }
            }
        };
        for (seat, l) in lines {
            let r = match result {
                GameResult::Win(s) if s == seat => 1.0,
                GameResult::Win(_) => -1.0,
                GameResult::Draw => 0.0,
            };
            writeln!(f, "{l},\"result\":{r}}}").unwrap();
            rows += 1;
        }
    }
    // Card names by definition index (state blocks and option subject_def = index + 1).
    let names: Vec<String> = db.defs.iter().map(|d| format!("{:?}", d.name)).collect();
    std::fs::write(format!("{out}.defs.json"), format!("[{}]", names.join(","))).unwrap();
    println!("{games} games, {rows} rows, n_defs {n_defs}, state_len {} -> {out}", state_len(n_defs));
}

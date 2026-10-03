//! Cost of the search primitives on the real Legacy decks: clone, fork (determinize) and observe.
//! Usage: legacybench [deck dir] [samples]
use mtg_core::decision::Status;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::{DeckList, Game, UniformConsistentModel};

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "decks".into());
    let samples: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(400);
    let db = mtg_cards::legacy::build();
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let decks: Vec<DeckList> = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    let mut mids: Vec<(Game, UniformConsistentModel)> = Vec::new();
    for i in 0..samples as u64 {
        let (a, b) = (&decks[(i % 8) as usize], &decks[((i + 3) % 8) as usize]);
        let mut g = Game::new(db.clone(), [a, b], i, GameConfig::default());
        g.set_track_view_events(false);
        let mut rng = Pcg64::from_seed(i);
        let stop = 40 + (i * 37) % 500;
        let mut n = 0;
        let alive = loop {
            match g.advance() {
                Status::GameOver(_) => break false,
                Status::NeedDecision(_) => {
                    if n >= stop {
                        break true;
                    }
                    let p = g.pending().unwrap();
                    let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                    n += 1;
                }
            }
        };
        if alive {
            mids.push((g, UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] }));
        }
    }
    let time = |f: &mut dyn FnMut(usize)| {
        let reps = 20;
        let t = std::time::Instant::now();
        for r in 0..reps {
            for i in 0..mids.len() {
                f(i + r);
            }
        }
        t.elapsed().as_secs_f64() * 1e6 / (reps * mids.len()) as f64
    };
    let c = time(&mut |i| {
        let g = mids[i % mids.len()].0.clone();
        std::hint::black_box(g);
    });
    let f = time(&mut |i| {
        let (g, m) = &mids[i % mids.len()];
        let seat = g.pending().unwrap().seat;
        std::hint::black_box(g.fork(seat, i as u64, m).unwrap());
    });
    let o = time(&mut |i| {
        let g = &mids[i % mids.len()].0;
        std::hint::black_box(g.observe(g.pending().unwrap().seat));
    });
    println!("{} mid-game states: clone {c:.2} us, fork (clone + determinize) {f:.2} us, observe {o:.2} us", mids.len());
}

//! M0 throughput spike: games/s, decisions/s, clone cost and enumeration cost on the test pool.
//! Run with `cargo run --release -p mtg-fuzz --bin bench`.

use mtg_cards::testpool;
use mtg_core::decision::Status;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_view::*;
use std::time::Instant;

fn main() {
    let pool = testpool::build();
    let a = DeckList { main: pool.deck_red_green(), side: vec![] };
    let b = DeckList { main: pool.deck_white_black(), side: vec![] };

    // 1. Full random games, no logging, no checks.
    let games = 3000u64;
    let t = Instant::now();
    let mut decisions = 0u64;
    let mut max_turn = 0;
    let mut mids: Vec<Game> = Vec::new();
    for seed in 0..games {
        let mut g = Game::new(pool.db.clone(), [&a, &b], seed, GameConfig::default());
        g.set_track_view_events(false);
        let mut rng = Pcg64::from_seed(seed ^ 77);
        let mut n = 0u64;
        loop {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(_) => {
                    let p = g.pending().unwrap();
                    let (id, len) = (p.id, p.options.len());
                    let idx = rng.below(len as u64) as usize;
                    g.apply(id, idx).unwrap();
                    n += 1;
                    if (n == 60 || n == 300) && seed % 50 == 0 {
                        g.advance();
                        mids.push(g.clone());
                    }
                    if n > 100_000 {
                        break;
                    }
                }
            }
        }
        decisions += n;
        max_turn = max_turn.max(g.raw_state().turn_number());
    }
    let dt = t.elapsed().as_secs_f64();
    println!("games: {games} in {dt:.2}s = {:.0} games/s, {:.0} decisions/s, {:.1} decisions/game, max turn {max_turn}", games as f64 / dt, decisions as f64 / dt, decisions as f64 / games as f64);

    // 2. Clone cost on mid-game states (decision 60).
    let reps = 20_000;
    let t = Instant::now();
    let mut keep = 0usize;
    for i in 0..reps {
        let g = mids[i % mids.len()].clone();
        keep += std::hint::black_box(&g).raw_state().turn_number() as usize;
    }
    let ns = t.elapsed().as_nanos() as f64 / reps as f64;
    println!("clone (Game, decisions 60 and 300): {:.2} us ({} samples) [{keep}]", ns / 1000.0, mids.len());

    // 3. Option enumeration cost: advance one step from a cloned state and read pending.
    let t = Instant::now();
    let mut opts = 0usize;
    let reps = 20_000;
    for i in 0..reps {
        let mut g = mids[i % mids.len()].clone();
        if let Some(p) = g.pending().map(|p| (p.id, p.options.len())) {
            g.apply(p.0, 0).unwrap();
            g.advance();
            opts += g.pending().map(|p| p.options.len()).unwrap_or(0);
        }
    }
    let ns = t.elapsed().as_nanos() as f64 / reps as f64;
    println!("clone + apply + advance (to next decision): {:.2} us [{opts}]", ns / 1000.0);
}

use mtg_cards::testpool;
use mtg_fuzz::*;
use mtg_view::DeckList;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let check_every: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let seed0: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let deep_sample: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let deep: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let pool = testpool::build();
    let decks = [
        DeckList { main: pool.deck_red_green(), side: vec![] },
        DeckList { main: pool.deck_white_black(), side: vec![] },
        DeckList { main: pool.deck_blue(), side: vec![] },
        DeckList { main: pool.deck_tricks(), side: vec![] },
        DeckList { main: pool.deck_triggers(), side: vec![] },
    ];
    let n = decks.len() as u64;
    let mut stats = FuzzStats { games: 0, decisions: 0, engine_steps: 0, wins: [0; 2], draws: 0, truncated: 0, coverage: Default::default() };
    let t = std::time::Instant::now();
    for i in 0..games {
        let a = (i % n) as usize;
        let b = ((a as u64 + 1 + (i / n) % (n - 1)) % n) as usize;
        let seed = seed0 + i;
        if deep > 0 && i % deep == 0 {
            if let Err(e) = record_and_replay(&pool, [&decks[a], &decks[b]], seed, (i % 2) as u8) {
                eprintln!("VIOLATION (replay) seed {seed}: {e}");
                std::process::exit(1);
            }
        }
        if let Err(v) = play_random_game(&pool, [&decks[a], &decks[b]], seed, (i % 2) as u8, check_every, deep_sample, 20000, &mut stats) {
            eprintln!("VIOLATION game {i} seed {} after {} decisions: {}", v.seed, v.decisions_so_far, v.detail);
            std::process::exit(1);
        }
    }
    let dt = t.elapsed().as_secs_f64();
    println!(
        "{} games ({} truncated), {} decisions, {} engine steps in {:.2}s: {:.0} games/s, {:.0} decisions/s; wins {:?} draws {}",
        stats.games, stats.truncated, stats.decisions, stats.engine_steps, dt, stats.games as f64 / dt, stats.decisions as f64 / dt, stats.wins, stats.draws
    );
    if check_every > 0 {
        for (k, v) in &stats.coverage {
            println!("  {k}: {v}");
        }
    }
}

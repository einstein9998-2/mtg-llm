//! Random-game fuzz over the eight real Legacy decks in a directory of N-card-name lists.
//! Usage: legacyfuzz <deck dir> [games] [check_every] [seed0] [deep_every] [deep_sample]
use mtg_cards::testpool::TestPool;
use mtg_fuzz::*;
use mtg_view::DeckList;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args.get(1).map(String::as_str).unwrap_or("/mnt/project-files/decks");
    let games: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(200);
    let check_every: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let seed0: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let deep: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let deep_sample: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
    let pool = TestPool { db: mtg_cards::legacy::build() };
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let mut decks = vec![];
    let mut names = vec![];
    for p in &paths {
        let (d, missing) = load_deck_file(&pool.db, p);
        if !missing.is_empty() {
            eprintln!("{}: missing {:?}", p.display(), missing);
        }
        println!("{}: main {} side {}", p.file_name().unwrap().to_string_lossy(), d.main.len(), d.side.len());
        names.push(p.file_stem().unwrap().to_string_lossy().to_string());
        decks.push(d);
    }
    let n = decks.len() as u64;
    let mut stats = FuzzStats { games: 0, decisions: 0, engine_steps: 0, wins: [0; 2], draws: 0, truncated: 0, coverage: Default::default() };
    let t = std::time::Instant::now();
    for i in 0..games {
        let a = (i % n) as usize;
        let b = ((a as u64 + 1 + (i / n) % (n - 1)) % n) as usize;
        let seed = seed0 + i;
        if deep > 0 && i % deep == 0 {
            if let Err(e) = record_and_replay(&pool, [&decks[a], &decks[b]], seed, (i % 2) as u8) {
                eprintln!("VIOLATION (replay) {} v {} seed {seed}: {e}", names[a], names[b]);
                std::process::exit(1);
            }
        }
        if let Err(v) = play_random_game(&pool, [&decks[a], &decks[b]], seed, (i % 2) as u8, check_every, deep_sample, 20000, &mut stats) {
            eprintln!("VIOLATION game {i} {} v {} seed {} after {} decisions: {}", names[a], names[b], v.seed, v.decisions_so_far, v.detail);
            std::process::exit(1);
        }
    }
    let dt = t.elapsed().as_secs_f64();
    println!(
        "{} games ({} truncated), {} decisions, {} steps in {:.2}s: {:.0} games/s; wins {:?} draws {}",
        stats.games, stats.truncated, stats.decisions, stats.engine_steps, dt, stats.games as f64 / dt, stats.wins, stats.draws
    );
    for (k, v) in &stats.coverage {
        if !k.starts_with("card:") {
            println!("  {k}: {v}");
        }
    }
    // Deck cards that never entered the battlefield or were never cast in any game.
    let mut never = vec![];
    for d in &decks {
        for &id in d.main.iter() {
            let name = &pool.db.def(id).name;
            if !stats.coverage.contains_key(&format!("card:{name}")) && !never.contains(name) {
                never.push(name.clone());
            }
        }
    }
    println!("deck cards never cast or entered ({}): {:?}", never.len(), never);
}

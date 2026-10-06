//! Random-game fuzz of the SIDEBOARDED decks: for every plan in a plan directory, plays the boarded
//! 60 against the boarded 60 of the opponent (their plan back, if they have one) with invariants
//! checked as in `legacyfuzz`. Sideboard cards never enter play in main-deck fuzz, so this is the
//! first time most of them run in a full game.
//! Usage: sbfuzz <deck dir> <plan dir> [games per plan] [check_every] [seed0] [deep_every]
use mtg_cards::testpool::TestPool;
use mtg_fuzz::*;
use mtg_match::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args.get(1).map(String::as_str).unwrap_or("decks");
    let plans = args.get(2).map(String::as_str).unwrap_or("sideboard-plans");
    let games: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(200);
    let check_every: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let seed0: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
    let deep: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
    let pool = TestPool { db: mtg_cards::legacy::build() };
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let mut names = vec![];
    let mut decks = vec![];
    for p in &paths {
        let (d, missing) = load_deck_file(&pool.db, p);
        assert!(missing.is_empty(), "{}: missing {missing:?}", p.display());
        names.push(p.file_stem().unwrap().to_string_lossy().to_string());
        decks.push(d);
    }
    let book = PlanBook::load_dir(&pool.db, std::path::Path::new(plans)).unwrap_or_else(|e| panic!("plans: {e}"));
    let idx = |n: &str| names.iter().position(|x| x == n).unwrap_or_else(|| panic!("no deck {n}"));
    let mut stats = FuzzStats { games: 0, decisions: 0, engine_steps: 0, wins: [0; 2], draws: 0, truncated: 0, coverage: Default::default() };
    let mut sb_cards: Vec<String> = Vec::new();
    let t = std::time::Instant::now();
    let mut n_pairs = 0;
    for (own, opp) in book.pairs() {
        if own == opp {
            // The mirror: both seats board with the same plan.
        }
        let (a, b) = (idx(&own), idx(&opp));
        let mine = board(&pool.db, &decks[a], book.plan(&own, &opp).unwrap()).unwrap_or_else(|e| panic!("{own} vs {opp}: {e}"));
        let theirs = match book.plan(&opp, &own) {
            Some(p) => board(&pool.db, &decks[b], p).unwrap_or_else(|e| panic!("{opp} vs {own}: {e}")),
            None => decks[b].clone(),
        };
        // Cards that came in from the sideboard.
        for c in book.plan(&own, &opp).unwrap().inn.iter().chain(book.plan(&opp, &own).map(|p| p.inn.iter()).into_iter().flatten()) {
            let n = pool.db.def(*c).name.clone();
            if !sb_cards.contains(&n) {
                sb_cards.push(n);
            }
        }
        n_pairs += 1;
        for i in 0..games {
            let seed = seed0 + i + 100_000 * n_pairs as u64;
            if deep > 0 && i % deep == 0 {
                if let Err(e) = record_and_replay(&pool, [&mine, &theirs], seed, (i % 2) as u8) {
                    eprintln!("VIOLATION (replay) {own} v {opp} seed {seed}: {e}");
                    std::process::exit(1);
                }
            }
            if let Err(v) = play_random_game(&pool, [&mine, &theirs], seed, (i % 2) as u8, check_every, 0, 20000, &mut stats) {
                eprintln!("VIOLATION game {i} {own} v {opp} seed {} after {} decisions: {}", v.seed, v.decisions_so_far, v.detail);
                std::process::exit(1);
            }
        }
    }
    let dt = t.elapsed().as_secs_f64();
    println!("{n_pairs} boarded matchups x {games} games = {} games ({} truncated), {} decisions in {:.1}s; wins {:?} draws {}", stats.games, stats.truncated, stats.decisions, dt, stats.wins, stats.draws);
    let never: Vec<&String> = sb_cards.iter().filter(|n| !stats.coverage.contains_key(&format!("card:{n}"))).collect();
    println!("sideboard cards brought in: {}; never cast or entered play: {:?}", sb_cards.len(), never);
}

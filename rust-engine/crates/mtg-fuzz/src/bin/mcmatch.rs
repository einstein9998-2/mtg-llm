//! Flat Monte Carlo agent vs a random player on the real decks, to sanity-check forks as a search
//! primitive. Usage: mcmatch [games] [samples] [max_rollout]
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(16);
    let samples: u32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let rollout: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let db = mtg_cards::legacy::build();
    let mut paths: Vec<_> = std::fs::read_dir("decks").unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let decks: Vec<DeckList> = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    let (mut mc_wins, mut rnd_wins, mut draws, mut undecided) = (0, 0, 0, 0);
    let (mut agent_decisions, mut forks_us) = (0u64, 0f64);
    let t0 = std::time::Instant::now();
    for i in 0..games {
        let (da, db_) = (&decks[(i % 8) as usize], &decks[((i * 3 + 1) % 8) as usize]);
        let mc_seat = Seat((i % 2) as u8);
        let decklists = if mc_seat == Seat(0) { [da, db_] } else { [db_, da] };
        let mut g = Game::new(db.clone(), decklists, 100 + i, GameConfig::default());
        g.set_track_view_events(false);
        let model = UniformConsistentModel { decks: [decklists[0].main.clone(), decklists[1].main.clone()] };
        let mut mc = FlatMc::new(&model, samples, rollout, i);
        let mut rnd = RandomPolicy::new(i + 5);
        let mut n = 0;
        let res = loop {
            match g.advance() {
                Status::GameOver(r) => break Some(r),
                Status::NeedDecision(_) => {
                    let (id, seat, len) = {
                        let p = g.pending().unwrap();
                        (p.id, p.seat, p.options.len())
                    };
                    let idx = if seat == mc_seat {
                        agent_decisions += 1;
                        let t = std::time::Instant::now();
                        let c = mc.choose(&g.seat_view(seat), len);
                        forks_us += t.elapsed().as_secs_f64() * 1e6;
                        c
                    } else {
                        rnd.choose(&g.seat_view(seat), len)
                    };
                    g.apply(id, idx).unwrap();
                    n += 1;
                    if n > 6000 {
                        break None;
                    }
                }
            }
        };
        match res {
            Some(mtg_core::decision::GameResult::Win(s)) if s == mc_seat => mc_wins += 1,
            Some(mtg_core::decision::GameResult::Win(_)) => rnd_wins += 1,
            Some(_) => draws += 1,
            None => undecided += 1,
        }
        eprintln!("game {i}: {:?} (mc seat {})", res, mc_seat.idx());
    }
    println!("MC {mc_wins} - random {rnd_wins}, draws {draws}, undecided {undecided} in {:.1}s; {agent_decisions} agent decisions, {:.1} ms each", t0.elapsed().as_secs_f64(), forks_us / agent_decisions.max(1) as f64 / 1000.0);
}

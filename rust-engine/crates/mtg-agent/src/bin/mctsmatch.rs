//! Strength check: determinized MCTS (rollout evaluator) against a random player over mixed decks,
//! seats alternating. Usage: mctsmatch <decks dir> <games> <iterations> [max_rollout] [--net weights.bin]
//! [--opp random|rollout]. With --net the searching player is guided by the net instead of random
//! rollouts; --opp rollout makes the opponent a rollout MCTS with the same iteration count.
use mtg_agent::*;
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_view::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = a.get(1).map(String::as_str).unwrap_or("decks");
    let games: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(24);
    let iters: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(16);
    let max_rollout: u32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(2500);
    let flag = |name: &str| a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned();
    let net = flag("--net").map(|p| std::sync::Arc::new(Net::load(std::path::Path::new(&p)).unwrap()));
    let opp_rollout = flag("--opp").as_deref() == Some("rollout");
    let db = mtg_cards::legacy::build();
    let n_defs = db.defs.len();
    let (_, decks) = load_deck_dir(&db, dir);
    let (mut wins, mut losses, mut draws, mut decisions) = (0, 0, 0, 0u64);
    let t0 = std::time::Instant::now();
    for i in 0..games {
        let (da, dbk) = (&decks[(i % 8) as usize], &decks[((i * 3 + 1 + i / 8) % 8) as usize]);
        let mcts_seat = Seat((i % 2) as u8);
        let mut g = Game::new(db.clone(), [da, dbk], 9000 + i, GameConfig { first_player: Seat(((i / 2) % 2) as u8), ..GameConfig::default() });
        let model = UniformConsistentModel { decks: [da.main.clone(), dbk.main.clone()] };
        let cfg = SearchConfig { iterations: iters, seed: i + 1, ..SearchConfig::default() };
        let ev: Box<dyn Evaluator> = match &net {
            Some(n) => Box::new(NetEvaluator::new(n.clone(), n_defs)),
            None => Box::new(RolloutEvaluator::new(max_rollout, i + 5)),
        };
        let mut mc = MctsPolicy::new(&model, ev, cfg.clone());
        let mut rnd: Box<dyn Policy> = if opp_rollout {
            Box::new(MctsPolicy::new(&model, Box::new(RolloutEvaluator::new(max_rollout, i + 9)), SearchConfig { seed: i + 1000, ..cfg }))
        } else {
            Box::new(RandomPolicy::new(i + 77))
        };
        let mut n = 0u32;
        let result = loop {
            match g.advance() {
                Status::GameOver(r) => break Some(r),
                Status::NeedDecision(seat) => {
                    let sv = g.seat_view(seat);
                    let d = sv.decision().unwrap();
                    let k = d.options.len();
                    let idx = if seat == mcts_seat { decisions += (k > 1) as u64; mc.choose(&sv, k) } else { rnd.choose(&sv, k) };
                    g.apply(d.id, idx).unwrap();
                    n += 1;
                    if n > 6000 {
                        break None;
                    }
                }
            }
        };
        match score(result, mcts_seat) {
            x if x == 1.0 => wins += 1,
            x if x == 0.0 => losses += 1,
            _ => draws += 1,
        }
    }
    println!("MCTS({iters} it{}) vs {} over {games} games: {wins} wins, {losses} losses, {draws} draws; {decisions} searched decisions in {:.1}s", if net.is_some() { ", net" } else { ", rollouts" }, if opp_rollout { "rollout MCTS" } else { "random" }, t0.elapsed().as_secs_f64());
}


//! Self-play data from determinized MCTS: both seats search, every non-trivial decision is logged
//! with the visit distribution (policy target), the root value and, at game end, the result for the
//! acting seat (value target). One JSON object per line.
//!
//! Usage: mctsdata <decks dir> <games> <iterations> <out.jsonl> [--net weights.bin] [--rollout N]
//!        [--seed S] [--threads T] [--explore K] [--noise EPS]
//! Without --net the evaluator is random rollouts; with it, the net's priors and value guide the search.
use mtg_agent::*;
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_view::*;
use serde_json::json;
use std::io::Write;
use std::sync::Arc;

struct Args {
    dir: String,
    games: u64,
    iters: u32,
    out: String,
    net: Option<String>,
    rollout: u32,
    seed: u64,
    threads: usize,
    explore: u32,
    noise: f32,
}

fn parse() -> Args {
    let a: Vec<String> = std::env::args().collect();
    let flag = |name: &str| a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned();
    Args {
        dir: a.get(1).cloned().unwrap_or("decks".into()),
        games: a.get(2).and_then(|s| s.parse().ok()).unwrap_or(8),
        iters: a.get(3).and_then(|s| s.parse().ok()).unwrap_or(16),
        out: a.get(4).cloned().unwrap_or("/tmp/mctsdata.jsonl".into()),
        net: flag("--net"),
        rollout: flag("--rollout").and_then(|s| s.parse().ok()).unwrap_or(2500),
        seed: flag("--seed").and_then(|s| s.parse().ok()).unwrap_or(1),
        threads: flag("--threads").and_then(|s| s.parse().ok()).unwrap_or(1),
        explore: flag("--explore").and_then(|s| s.parse().ok()).unwrap_or(30),
        noise: flag("--noise").and_then(|s| s.parse().ok()).unwrap_or(0.25),
    }
}


fn play(db: &Arc<mtg_core::card::CardDb>, decks: &[DeckList], net: &Option<Arc<Net>>, a: &Args, i: u64) -> (Vec<String>, i8) {
    let n_defs = db.defs.len();
    let (da, dbk) = (&decks[(i % 8) as usize], &decks[((i * 3 + 1 + i / 8) % 8) as usize]);
    let first = Seat((i % 2) as u8);
    let mut g = Game::new(db.clone(), [da, dbk], a.seed * 1_000_003 + i, GameConfig { first_player: first, ..GameConfig::default() });
    let model = UniformConsistentModel { decks: [da.main.clone(), dbk.main.clone()] };
    let mut rng = Pcg64::from_seed(a.seed ^ (i + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut buf = Vec::new();
    let mut rows: Vec<(Seat, serde_json::Value)> = Vec::new();
    let (mut decisions, mut counter) = (0u32, 0u64);
    let result = loop {
        match g.advance() {
            Status::GameOver(r) => break r,
            Status::NeedDecision(seat) => {
                let sv = g.seat_view(seat);
                let o = sv.observe();
                let d = o.decision.clone().unwrap();
                let n = d.options.len();
                let idx = if n == 1 {
                    0
                } else {
                    decisions += 1;
                    counter += 1;
                    let cfg = SearchConfig { iterations: a.iters, seed: rng.next_u64() ^ counter, root_noise: a.noise, ..SearchConfig::default() };
                    let mut ev: Box<dyn Evaluator> = match net {
                        Some(nn) => Box::new(NetEvaluator::new(nn.clone(), n_defs)),
                        None => Box::new(RolloutEvaluator::new(a.rollout, rng.next_u64())),
                    };
                    let r = search(&sv, &model, &mut *ev, &cfg);
                    let idx = if decisions <= a.explore {
                        let p = r.policy(1.0);
                        let mut x = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
                        let mut pick = p.len() - 1;
                        for (k, &w) in p.iter().enumerate() {
                            x -= w as f64;
                            if x <= 0.0 {
                                pick = k;
                                break;
                            }
                        }
                        pick
                    } else {
                        r.best()
                    };
                    encode_state(&o, n_defs, &mut buf);
                    let state: Vec<_> = buf.iter().enumerate().filter(|(_, v)| **v != 0.0).map(|(k, v)| json!([k, v])).collect();
                    let opts: Vec<_> = encode_options(&o).iter().map(|x| json!([x.kind, x.decision, x.subject_def, x.subject_zone, x.value])).collect();
                    rows.push((seat, json!({"game": i, "seat": seat.idx(), "state": state, "options": opts, "policy": r.policy(1.0), "root_value": r.value, "chosen": idx})));
                    idx
                };
                g.apply(d.id, idx).unwrap();
                if decisions > 4000 {
                    break mtg_core::decision::GameResult::Draw;
                }
            }
        }
    };
    let res0 = match result {
        mtg_core::decision::GameResult::Win(s) if s == Seat(0) => 1,
        mtg_core::decision::GameResult::Win(_) => -1,
        _ => 0,
    };
    let lines = rows
        .into_iter()
        .map(|(seat, mut v)| {
            let r = if res0 == 0 { 0 } else if (res0 == 1) == (seat == Seat(0)) { 1 } else { -1 };
            v["result"] = json!(r);
            v.to_string()
        })
        .collect();
    (lines, res0)
}

fn main() {
    let a = Arc::new(parse());
    let db = mtg_cards::legacy::build();
    let (_, decks) = load_deck_dir(&db, &a.dir);
    let net = a.net.as_ref().map(|p| Arc::new(Net::load(std::path::Path::new(p)).unwrap()));
    let t0 = std::time::Instant::now();
    let next = std::sync::atomic::AtomicU64::new(0);
    let results = std::sync::Mutex::new(Vec::<(u64, Vec<String>, i8)>::new());
    std::thread::scope(|s| {
        for _ in 0..a.threads.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if i >= a.games {
                    break;
                }
                // A runaway game (the engine's object arena is exhausted) must not kill a long run.
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| play(&db, &decks, &net, &a, i))) {
                    Ok((lines, r)) => results.lock().unwrap().push((i, lines, r)),
                    Err(_) => eprintln!("game {i} (decks {} vs {}) panicked and was dropped", i % 8, (i * 3 + 1 + i / 8) % 8),
                }
            });
        }
    });
    let mut res = results.into_inner().unwrap();
    res.sort_by_key(|r| r.0);
    let mut f = std::io::BufWriter::new(std::fs::File::create(&a.out).unwrap());
    let (mut rows, mut s0, mut s1, mut dr) = (0usize, 0, 0, 0);
    for (_, lines, r) in &res {
        for l in lines {
            writeln!(f, "{l}").unwrap();
            rows += 1;
        }
        match r { 1 => s0 += 1, -1 => s1 += 1, _ => dr += 1 }
    }
    let names: Vec<String> = db.defs.iter().map(|d| d.name.to_string()).collect();
    std::fs::write(format!("{}.defs.json", a.out), serde_json::to_string(&names).unwrap()).unwrap();
    std::fs::write(format!("{}.meta.json", a.out), json!({"n_defs": db.defs.len(), "state_len": state_len(db.defs.len()), "n_blocks": N_BLOCKS, "n_scalars": SCALAR_NAMES.len()}).to_string()).unwrap();
    println!("{} games, {rows} rows, seat0 wins {s0}, seat1 wins {s1}, draws {dr}, n_defs {}, state_len {} in {:.1}s -> {}", a.games, db.defs.len(), state_len(db.defs.len()), t0.elapsed().as_secs_f64(), a.out);
}

//! Re-runs the bot's search on Brady's labeled positions, with and without the rules guard, and
//! compares each pick with his. Agent-side only (SeatView). Positions are reproduced by replaying the
//! recorded action list.
//! Usage: rescore <decks dir> <net.bin> <labels.jsonl>... [--deep 400]
#[path = "../rules.rs"]
mod rules;

use mtg_agent::*;
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_view::*;
use std::sync::Arc;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let deep: u32 = a.iter().position(|x| x == "--deep").and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(400);
    let net = Arc::new(Net::load(std::path::Path::new(&a[2])).unwrap_or_else(|e| panic!("{e}")));
    let db = mtg_cards::legacy::build();
    let (names, decks) = load_deck_dir(&db, &a[1]);
    let me = names.iter().position(|n| n == "alurentell").unwrap();
    let opp = names.iter().position(|n| n == "ur-cutter").unwrap();
    let n_defs = db.defs.len();
    println!("id\tkind\tbrady\tweight\tbot_orig\tquick\tdeep\tquick+rules\tdeep+rules");
    let mut tally = [0u32; 5];
    let mut tot = 0.0f32;
    for file in a[3..].iter().filter(|x| x.ends_with(".jsonl")) {
        for line in std::fs::read_to_string(file).unwrap().lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            let id = v["id"].as_str().unwrap().to_string();
            let w = v["weight"].as_f64().unwrap() as f32;
            let brady = v["brady_pick"].as_u64().unwrap() as usize;
            let ix: u64 = id.rsplit("-d").next().unwrap().parse().unwrap();
            let (gseed, bot, first) = (v["game_seed"].as_u64().unwrap(), v["bot_seat"].as_u64().unwrap() as u8, v["first"].as_u64().unwrap() as u8);
            let hist: Vec<(u8, usize)> = v["replay"].as_array().unwrap().iter().map(|x| (x[0].as_u64().unwrap() as u8, x[1].as_u64().unwrap() as usize)).collect();
            let (d0, d1) = if bot == 0 { (&decks[me], &decks[opp]) } else { (&decks[opp], &decks[me]) };
            let mut g = Game::new(db.clone(), [d0, d1], gseed, GameConfig { first_player: Seat(first), ..GameConfig::default() });
            for (i, (seat, idx)) in hist.iter().enumerate() {
                let Status::NeedDecision(s) = g.advance() else { panic!("ended early") };
                assert_eq!(s.0, *seat, "{id}: replay diverged at {i}");
                let d = g.seat_view(s).observe().decision.unwrap();
                g.apply(d.id, *idx).unwrap();
            }
            let Status::NeedDecision(s) = g.advance() else { panic!("no decision") };
            let sv = g.seat_view(s);
            let obs = sv.observe();
            let d = obs.decision.clone().unwrap();
            let n = d.options.len();
            let model = UniformConsistentModel { decks: [d0.main.clone(), d1.main.clone()] };
            let mut ev = NetEvaluator::new(net.clone(), n_defs);
            let cfg = SearchConfig { iterations: 64, seed: gseed.wrapping_mul(1315423911).wrapping_add(ix), ..SearchConfig::default() };
            let bad = rules::forbidden(&obs, &d);
            let pick = |r: &SearchResult, mask: bool| -> usize {
                let mut r = SearchResult { visits: r.visits.clone(), q: r.q.clone(), value: r.value, iterations: r.iterations, fork_errors: 0 };
                if mask {
                    for k in 0..n {
                        if bad[k] {
                            r.visits[k] = 0;
                            r.q[k] = -2.0;
                        }
                    }
                }
                r.best()
            };
            let q = search(&sv, &model, &mut ev, &cfg);
            // deep: two seeds, summed visits (same as the flagger)
            let mut dv = vec![0u32; n];
            let mut dq = vec![0f32; n];
            for k in 0..2u64 {
                let c = SearchConfig { iterations: deep, seed: cfg.seed ^ (0xABCDEF + k * 7919), ..SearchConfig::default() };
                let r = search(&sv, &model, &mut ev, &c);
                for x in 0..n {
                    dv[x] += r.visits[x];
                    dq[x] += r.q[x] * r.visits[x] as f32;
                }
            }
            for x in 0..n {
                dq[x] = if dv[x] > 0 { dq[x] / dv[x] as f32 } else { -2.0 };
            }
            let dr = SearchResult { visits: dv, q: dq, value: 0.0, iterations: deep * 2, fork_errors: 0 };
            let (q0, d0p, q1, d1p) = (pick(&q, false), pick(&dr, false), pick(&q, true), pick(&dr, true));
            let orig = v["bot_choice"].as_u64().unwrap() as usize;
            println!("{id}\t{:?}\t{brady}\t{w}\t{orig}\t{q0}\t{d0p}\t{q1}\t{d1p}", d.kind);
            if w > 0.0 {
                tot += w;
                for (t, x) in [orig, q0, d0p, q1, d1p].iter().enumerate() {
                    if *x == brady {
                        tally[t] += 1;
                    }
                }
            }
        }
    }
    println!("weighted positions {tot}; agree with Brady (count): orig {} quick {} deep {} quick+rules {} deep+rules {}", tally[0], tally[1], tally[2], tally[3], tally[4]);
}

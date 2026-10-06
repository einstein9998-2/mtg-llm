//! Best-of-three matches with sideboarding, both seats searched with determinized MCTS.
//!
//! Usage: bo3 <decks dir> <plans dir> <deck A> <deck B> [matches] [iterations]
//!        [--net w.bin] [--rollout N] [--board both|a|b|none] [--opp-view plan|main]
//!        [--threads T] [--seed S] [--csv out.csv] [--trace games.jsonl] [--plans-b <dir>] [--public <dir>]
//! --plans-b gives deck B its own plan directory (a mirror with two different plans).
//! --public gives the standard plan directory both seats are assumed to know (default: each seat's own
//! directory). Always pass it when testing a variant, else the opponent's belief reads the variant.
//!        bo3 <decks dir> <plans dir> --check      (validate every plan, print the swaps)
//!
//! Deck A sits in seat 0. Game one's first player alternates by match; later games the loser plays
//! first. --board says who sideboards from game two on (default both).
use mtg_agent::*;
use mtg_core::ids::Seat;
use mtg_match::*;
use mtg_view::{BeliefModel, Policy};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

struct Fac {
    net: Option<Arc<Net>>,
    n_defs: usize,
    iters: u32,
    rollout: u32,
}

impl PlayerFactory for Fac {
    fn make<'m>(&mut self, seat: Seat, _game_no: u32, model: &'m dyn BeliefModel, seed: u64) -> Box<dyn Policy + 'm> {
        let ev: Box<dyn Evaluator> = match &self.net {
            Some(n) => Box::new(NetEvaluator::new(n.clone(), self.n_defs)),
            None => Box::new(RolloutEvaluator::new(self.rollout, seed ^ seat.0 as u64)),
        };
        Box::new(MctsPolicy::new(model, ev, SearchConfig { iterations: self.iters, seed, ..SearchConfig::default() }))
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let flag = |name: &str| a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned();
    let dir = a.get(1).cloned().unwrap_or("decks".into());
    let plans = a.get(2).cloned().unwrap_or("sideboard-plans".into());
    let db = mtg_cards::legacy::build();
    let (names, decks) = load_deck_dir(&db, &dir);
    let book = PlanBook::load_dir(&db, std::path::Path::new(&plans)).unwrap_or_else(|e| panic!("plans: {e}"));
    let public_book = flag("--public").map(|d| PlanBook::load_dir(&db, std::path::Path::new(&d)).unwrap_or_else(|e| panic!("public: {e}")));
    let book_b = flag("--plans-b").map(|d| PlanBook::load_dir(&db, std::path::Path::new(&d)).unwrap_or_else(|e| panic!("plans-b: {e}")));
    let cfg0 = MatchConfig::default();
    if a.iter().any(|x| x == "--check") {
        let mut bad = 0;
        for (own, opp, side, plan) in book.entries() {
            let Some(i) = names.iter().position(|n| *n == own) else {
                println!("{own} vs {opp}: no deck named {own}");
                bad += 1;
                continue;
            };
            let opp = if side == Side::Any { opp } else { format!("{opp} {}", if side == Side::OnPlay { "on-play" } else { "on-draw" }) };
            match board(&db, &decks[i], plan) {
                Ok(d) => {
                    let nm = |v: &[mtg_core::ids::CardDefId]| {
                        let mut s: Vec<String> = Vec::new();
                        for &c in v {
                            let n = &db.def(c).name;
                            match s.iter_mut().find(|x| x.ends_with(&format!(" {n}"))) {
                                Some(x) => {
                                    let k: usize = x.split(' ').next().unwrap().parse().unwrap();
                                    *x = format!("{} {n}", k + 1);
                                }
                                None => s.push(format!("1 {n}")),
                            }
                        }
                        s.join(", ")
                    };
                    println!("OK  {own} vs {opp}: out [{}] in [{}] (main {}, side {})", nm(&plan.out), nm(&plan.inn), d.main.len(), d.side.len());
                }
                Err(e) => {
                    println!("BAD {own} vs {opp}: {e}");
                    bad += 1;
                }
            }
        }
        println!("{} plans, {bad} invalid", book.entries().len());
        std::process::exit((bad > 0) as i32);
    }
    let find = |n: &str| names.iter().position(|x| x == n).unwrap_or_else(|| panic!("no deck {n} in {dir} (have {names:?})"));
    let (ia, ib) = (find(a.get(3).expect("deck A")), find(a.get(4).expect("deck B")));
    let matches: u64 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(40);
    let iters: u32 = a.get(6).and_then(|s| s.parse().ok()).unwrap_or(16);
    let rollout: u32 = flag("--rollout").and_then(|s| s.parse().ok()).unwrap_or(2500);
    let threads: usize = flag("--threads").and_then(|s| s.parse().ok()).unwrap_or(1);
    let seed0: u64 = flag("--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
    let board_flag = flag("--board").unwrap_or("both".into());
    let board_seats = match board_flag.as_str() {
        "both" => [true, true],
        "a" => [true, false],
        "b" => [false, true],
        "none" => [false, false],
        x => panic!("--board {x}"),
    };
    let opp_view = match flag("--opp-view").as_deref() {
        None | Some("plan") => OppView::Plan,
        Some("main") => OppView::Main,
        Some(x) => panic!("--opp-view {x}"),
    };
    let net = flag("--net").map(|p| Arc::new(Net::load(std::path::Path::new(&p)).unwrap()));
    let n_defs = db.defs.len();
    let (na, nb) = (names[ia].clone(), names[ib].clone());
    let next = AtomicU64::new(0);
    let results: Mutex<Vec<(u64, MatchResult)>> = Mutex::new(Vec::new());
    let t0 = std::time::Instant::now();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| {
                let mut fac = Fac { net: net.clone(), n_defs, iters, rollout };
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= matches {
                        break;
                    }
                    let cfg = MatchConfig { board: board_seats, opp_view, first: Seat((i % 2) as u8), seed: seed0 * 7919 + i, ..cfg0.clone() };
                    let r = play_match(&db, [&na, &nb], [&decks[ia], &decks[ib]], [Some(&book), Some(book_b.as_ref().unwrap_or(&book))], [Some(public_book.as_ref().unwrap_or(&book)), Some(public_book.as_ref().or(book_b.as_ref()).unwrap_or(&book))], &cfg, &mut fac).unwrap_or_else(|e| panic!("{e}"));
                    results.lock().unwrap().push((i, r));
                }
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(i, _)| *i);
    // Match and game tallies from deck A's side.
    let (mut mw, mut ml, mut md) = (0, 0, 0);
    let (mut g1w, mut g1n, mut gbw, mut gbn, mut gb_drawn, mut panics) = (0, 0, 0, 0, 0, 0);
    let (mut gb_a_boarded, mut gb_b_boarded) = (0, 0);
    let mut csv = String::from("match,game,first,a_boarded,b_boarded,winner\n");
    for (i, r) in &results {
        match r.winner() {
            Some(Seat(0)) => mw += 1,
            Some(_) => ml += 1,
            None => md += 1,
        }
        for (k, g) in r.games.iter().enumerate() {
            panics += g.panicked as u32;
            let w = match g.winner {
                Some(Seat(0)) => "A",
                Some(_) => "B",
                None => "draw",
            };
            csv += &format!("{i},{},{},{},{},{w}\n", k + 1, if g.first.0 == 0 { "A" } else { "B" }, g.boarded[0] as u8, g.boarded[1] as u8);
            if k == 0 {
                g1n += 1;
                g1w += (w == "A") as u32;
            } else {
                gb_drawn += g.winner.is_none() as u32;
                gbn += 1;
                gbw += (w == "A") as u32;
                gb_a_boarded += g.boarded[0] as u32;
                gb_b_boarded += g.boarded[1] as u32;
            }
        }
    }
    let pct = |w: u32, n: u32| if n == 0 { 0.0 } else { 100.0 * w as f64 / n as f64 };
    println!(
        "{na} (A) vs {nb} (B), {matches} matches, {iters} it{} , board={board_flag}, opp-view={:?}: matches A {mw} B {ml} undecided {md} ({:.1}% A); game 1: A {g1w}/{g1n} ({:.1}%); games 2+: A {gbw}/{gbn} ({:.1}%), {gb_drawn} drawn, boarded A in {gb_a_boarded} B in {gb_b_boarded}; panics {panics}; {:.0}s",
        if net.is_some() { ", net" } else { ", rollouts" },
        opp_view,
        pct(mw, mw + ml + md),
        pct(g1w, g1n),
        pct(gbw, gbn),
        t0.elapsed().as_secs_f64()
    );
    // One JSON line per game with everything `sbtrace` needs to replay it.
    if let Some(p) = flag("--trace") {
        let nm = |v: &[mtg_core::ids::CardDefId]| -> Vec<String> { v.iter().map(|&c| db.def(c).name.clone()).collect() };
        let mut out = String::new();
        for (i, r) in &results {
            for (k, g) in r.games.iter().enumerate() {
                let line = serde_json::json!({
                    "match": i, "game": k + 1, "first": g.first.0, "seed": g.seed,
                    "winner": g.winner.map(|w| w.0), "boarded": [g.boarded[0], g.boarded[1]],
                    "decks": [{"main": nm(&g.decks[0].main), "side": nm(&g.decks[0].side)}, {"main": nm(&g.decks[1].main), "side": nm(&g.decks[1].side)}],
                    "actions": g.actions,
                });
                out += &line.to_string();
                out.push('\n');
            }
        }
        std::fs::write(p, out).unwrap();
    }
    if let Some(p) = flag("--csv") {
        std::fs::write(p, csv).unwrap();
    }
}

//! Prismatic Ending spots: play postboard games (net + determinized MCTS on both seats), and at every
//! sorcery-speed decision of seat 0 (deck A) where Prismatic Ending can be cast to exile an opposing
//! creature, branch the true game state into arms and roll each out with the net policy (sampled from
//! the priors, both seats):
//!   cast  : cast Ending now on the most dangerous opposing creature it can exile (cheapest X that works)
//!   wait  : Ending may not be cast until the end of the opponent's next turn
//!   never : Ending may not be cast for the rest of the game
//!   free  : no restriction (what the net does by itself)
//! Output: one JSON line per spot with the position features and the win count of each arm.
//!
//! Usage: endingspot <decks dir> <plans dir> <deck A> <deck B> --net w.bin [--games N] [--iters 32]
//!        [--reps R] [--threads T] [--seed S] [--out spots.jsonl] [--temp 1.0] [--baseline N]
//! `--baseline N` also plays N whole games with the net policy only (no search), to compare strength.
use mtg_agent::*;
use mtg_core::card::CardDb;
use mtg_core::decision::{GameResult, Status};
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_core::types::Types;
use mtg_match::*;
use mtg_view::view::{ActionKind, Decision, ViewDecisionKind, ViewTarget};
use mtg_view::{BeliefModel, Game, Policy};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Cast,
    Wait,
    Never,
    Free,
    /// Hold this copy for one more own turn, then cast it at the first sorcery-speed chance (best plan).
    Late1,
    /// Hold for two more own turns, then cast.
    Late2,
}

const ARMS: [Arm; 6] = [Arm::Cast, Arm::Wait, Arm::Never, Arm::Free, Arm::Late1, Arm::Late2];

struct Fac {
    net: Arc<Net>,
    n_defs: usize,
    iters: u32,
}

impl Fac {
    fn make<'m>(&self, model: &'m dyn BeliefModel, seed: u64) -> Box<dyn Policy + 'm> {
        let ev: Box<dyn Evaluator> = Box::new(NetEvaluator::new(self.net.clone(), self.n_defs));
        Box::new(MctsPolicy::new(model, ev, SearchConfig { iterations: self.iters, seed, ..SearchConfig::default() }))
    }
}

fn is_ending_cast(d: &Decision, ending: CardDefId) -> Option<usize> {
    d.options.iter().position(|o| o.kind == ActionKind::CastSpell && o.subject_def == Some(ending))
}

fn mv_of(db: &CardDb, def: CardDefId) -> u32 {
    db.def(def).cost.as_ref().map_or(0, |c| c.mana_value())
}

fn result_seat(r: GameResult) -> Option<u8> {
    match r {
        GameResult::Win(s) => Some(s.0),
        _ => None,
    }
}

/// One way to cast Ending from the spot: the option indices to apply in order, and what it does.
#[derive(Clone, Debug)]
struct Plan {
    acts: Vec<usize>,
    target: String,
    tmv: u32,
    tpow: i32,
    x: u32,
    exiled: bool,
}

/// Passes everything until the stack is empty again at a priority decision (Ending resolves
/// unopposed) or 30 decisions go by.
fn settle(g: &mut Game) {
    for _ in 0..30 {
        match g.advance() {
            Status::GameOver(_) => return,
            Status::NeedDecision(seat) => {
                let sv = g.seat_view(seat);
                let d = sv.decision().unwrap();
                let o = sv.observe();
                if matches!(d.kind, ViewDecisionKind::Priority) && o.stack.is_empty() {
                    return;
                }
                let id = d.id;
                drop(sv);
                g.apply(id, 0).unwrap();
            }
        }
    }
}

/// Enumerates the (X, target) choices of casting Ending at the pending decision and records which of
/// them exile an opposing creature when nobody responds.
fn enumerate_plans(db: &CardDb, g: &Game, ci: usize) -> Vec<Plan> {
    let mut out = Vec::new();
    let mut base = g.clone();
    let id0 = base.seat_view(Seat(0)).decision().unwrap().id;
    base.apply(id0, ci).unwrap();
    rec(db, base, vec![ci], 0, &mut out);
    out
}

fn rec(db: &CardDb, mut g: Game, mut acts: Vec<usize>, x: u32, out: &mut Vec<Plan>) {
    for _ in 0..10 {
        match g.advance() {
            Status::GameOver(_) => return,
            Status::NeedDecision(seat) => {
                let sv = g.seat_view(seat);
                let d = sv.decision().unwrap();
                let o = sv.observe();
                let n = d.options.len();
                let id = d.id;
                if seat.0 == 0 && matches!(d.kind, ViewDecisionKind::Priority) {
                    // the spell is on the stack and we hold priority: let it resolve
                    let item = o.stack.iter().find(|s| !s.is_ability);
                    let tv = item.and_then(|s| {
                        s.targets.iter().find_map(|t| match t {
                            ViewTarget::Object { vid, name } => Some((*vid, name.clone())),
                            _ => None,
                        })
                    });
                    let Some((tvid, tname)) = tv else { return };
                    let Some(p) = o.battlefield.iter().find(|p| p.vid == tvid) else { return };
                    let (tmv, tpow, creature, theirs) = (mv_of(db, p.def), p.power, p.types.contains(Types::CREATURE), !p.controlled_by_me);
                    drop(sv);
                    if creature && theirs {
                        let mut g2 = g.clone();
                        settle(&mut g2);
                        let gone = !g2.seat_view(Seat(0)).observe().battlefield.iter().any(|p| p.vid == tvid);
                        out.push(Plan { acts, target: tname, tmv, tpow, x, exiled: gone });
                    }
                    return;
                }
                if n == 1 {
                    drop(sv);
                    g.apply(id, 0).unwrap();
                    acts.push(0);
                    continue;
                }
                if seat.0 == 0 && matches!(d.kind, ViewDecisionKind::ChooseX | ViewDecisionKind::ChooseTarget { .. }) {
                    let opts = d.options.clone();
                    let is_x = matches!(d.kind, ViewDecisionKind::ChooseX);
                    drop(sv);
                    for (i, a) in opts.iter().enumerate() {
                        let mut g2 = g.clone();
                        g2.apply(id, i).unwrap();
                        let mut a2 = acts.clone();
                        a2.push(i);
                        let xx = if is_x { a.value.unwrap_or_else(|| a.label.trim_start_matches("X = ").parse().unwrap_or(0)) } else { x };
                        rec(db, g2, a2, xx, out);
                    }
                    return;
                }
                // any other decision (payment details, the opponent's): the default option
                drop(sv);
                g.apply(id, 0).unwrap();
                acts.push(0);
            }
        }
    }
}

/// The plan to use: it must exile; most dangerous target, then cheapest X.
fn best_plan(plans: &[Plan]) -> Option<Plan> {
    plans.iter().filter(|p| p.exiled).max_by_key(|p| ((p.tpow as i64) * 100 + (p.tmv as i64) * 10, -(p.x as i64))).cloned()
}

struct Roller<'a> {
    db: &'a CardDb,
    ending: CardDefId,
    evs: [NetEvaluator; 2],
    rng: Pcg64,
    temp: f32,
}

impl<'a> Roller<'a> {
    fn unit(&mut self) -> f32 {
        ((self.rng.next_u64() >> 40) as f32 + 0.5) / (1u64 << 24) as f32
    }

    fn pick(&mut self, g: &Game, seat: Seat, n: usize, banned: &[bool]) -> usize {
        let e = self.evs[seat.idx()].eval(g, seat, n);
        let t = self.temp;
        let w: Vec<f32> = e.priors.iter().enumerate().map(|(i, p)| if banned[i] { 0.0 } else { p.max(1e-6).powf(1.0 / t) }).collect();
        let s: f32 = w.iter().sum();
        if s <= 0.0 {
            return (0..n).find(|&i| !banned[i]).unwrap_or(0);
        }
        let r = self.unit() * s;
        let mut acc = 0.0;
        for (i, x) in w.iter().enumerate() {
            acc += *x;
            if r <= acc {
                return i;
            }
        }
        n - 1
    }

    /// Net-policy playout from `g`'s pending decision (after applying `plan`, if any). Seat 0 may not
    /// cast Ending while `ban(turn)`.
    fn run(&mut self, mut g: Game, plan: Option<&Plan>, ban: &dyn Fn(u16) -> bool, late_from: Option<u16>, max_dec: u32) -> Option<u8> {
        let mut late_pending = late_from.is_some();
        if let Some(p) = plan {
            for &i in &p.acts {
                let seat = match g.advance() {
                    Status::NeedDecision(s) => s,
                    Status::GameOver(r) => return result_seat(r),
                };
                let id = g.seat_view(seat).decision().unwrap().id;
                g.apply(id, i).unwrap();
            }
        }
        for _ in 0..max_dec {
            match g.advance() {
                Status::GameOver(r) => return result_seat(r),
                Status::NeedDecision(seat) => {
                    let sv = g.seat_view(seat);
                    let d = sv.decision().expect("decision pending");
                    let n = d.options.len();
                    let id = d.id;
                    if late_pending && seat.0 == 0 && matches!(d.kind, ViewDecisionKind::Priority) {
                        if let Some(ci) = is_ending_cast(&d, self.ending) {
                            let o = sv.observe();
                            if o.turn >= late_from.unwrap() && o.stack.is_empty() && o.battlefield.iter().any(|p| !p.controlled_by_me && p.types.contains(Types::CREATURE)) {
                                drop(sv);
                                if let Some(p) = best_plan(&enumerate_plans(self.db, &g, ci)) {
                                    late_pending = false;
                                    for &i in &p.acts {
                                        let seat = match g.advance() {
                                            Status::NeedDecision(s) => s,
                                            Status::GameOver(r) => return result_seat(r),
                                        };
                                        let id = g.seat_view(seat).decision().unwrap().id;
                                        g.apply(id, i).unwrap();
                                    }
                                    continue;
                                }
                                // nothing to exile: fall through to a normal choice below
                                let sv = g.seat_view(seat);
                                let d = sv.decision().unwrap();
                                let banned: Vec<bool> = d.options.iter().map(|o| o.kind == ActionKind::CastSpell && o.subject_def == Some(self.ending)).collect();
                                drop(sv);
                                let idx = self.pick(&g, seat, n, &banned);
                                g.apply(id, idx).unwrap();
                                continue;
                            }
                        }
                    }
                    let idx = if n == 1 {
                        drop(sv);
                        0
                    } else {
                        let turn = sv.observe().turn;
                        let ending = self.ending;
                        let banned: Vec<bool> = if seat.0 == 0 && ban(turn) { d.options.iter().map(|o| o.kind == ActionKind::CastSpell && o.subject_def == Some(ending)).collect() } else { vec![false; n] };
                        drop(sv);
                        self.pick(&g, seat, n, &banned)
                    };
                    g.apply(id, idx).unwrap();
                }
            }
        }
        None
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let flag = |name: &str| a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned();
    let dir = a[1].clone();
    let plans = a[2].clone();
    let db = mtg_cards::legacy::build();
    eprintln!("card defs: {}", db.defs.len());
    let (names, decks) = load_deck_dir(&db, &dir);
    let book = PlanBook::load_dir(&db, std::path::Path::new(&plans)).unwrap_or_else(|e| panic!("plans: {e}"));
    let find = |n: &str| names.iter().position(|x| x == n).unwrap_or_else(|| panic!("no deck {n}"));
    let (ia, ib) = (find(&a[3]), find(&a[4]));
    let (na, nb) = (names[ia].clone(), names[ib].clone());
    let games: u64 = flag("--games").and_then(|s| s.parse().ok()).unwrap_or(20);
    let iters: u32 = flag("--iters").and_then(|s| s.parse().ok()).unwrap_or(32);
    let reps: u32 = flag("--reps").and_then(|s| s.parse().ok()).unwrap_or(8);
    let threads: usize = flag("--threads").and_then(|s| s.parse().ok()).unwrap_or(1);
    let seed0: u64 = flag("--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
    let temp: f32 = flag("--temp").and_then(|s| s.parse().ok()).unwrap_or(0.2);
    let max_life: i32 = flag("--max-life").and_then(|s| s.parse().ok()).unwrap_or(i32::MAX);
    let gen_net = flag("--gen").as_deref() == Some("net");
    let baseline: u64 = flag("--baseline").and_then(|s| s.parse().ok()).unwrap_or(0);
    let net = Arc::new(Net::load(std::path::Path::new(&flag("--net").expect("--net"))).unwrap());
    let n_defs = db.defs.len();
    let ending = db.id("Prismatic Ending").expect("Prismatic Ending");
    let out = Mutex::new(Vec::<String>::new());
    let next = AtomicU64::new(0);
    let t0 = std::time::Instant::now();
    // games, A wins, spots, infeasible, rollouts, baseline games, baseline A wins
    let tally = Mutex::new([0u64; 7]);
    let mk_roller = |seed: u64| Roller { db: &db, ending, evs: [NetEvaluator::new(net.clone(), n_defs), NetEvaluator::new(net.clone(), n_defs)], rng: Pcg64::from_seed(seed), temp };
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| {
                let fac = Fac { net: net.clone(), n_defs, iters };
                loop {
                    let gi = next.fetch_add(1, Ordering::SeqCst);
                    if gi >= games + baseline {
                        break;
                    }
                    let is_base = gi >= games;
                    let first = Seat((gi % 2) as u8);
                    let mcfg = MatchConfig { board: [true, true], first, seed: seed0 * 7919 + gi, max_games: 1, board_from: 0, ..MatchConfig::default() };
                    let (gdecks, _) = game_decks(&db, [&na, &nb], [&decks[ia], &decks[ib]], [Some(&book), Some(&book)], &mcfg, 0, first).unwrap();
                    let seed = mcfg.seed.wrapping_mul(1_000_003);
                    let mut g = Game::new(db.clone(), [&gdecks[0], &gdecks[1]], seed, GameConfig { first_player: first, ..GameConfig::default() });
                    if is_base {
                        let w = mk_roller(seed0 ^ gi).run(g, None, &|_| false, None, 6000);
                        let mut t = tally.lock().unwrap();
                        t[5] += 1;
                        t[6] += (w == Some(0)) as u64;
                        continue;
                    }
                    let models = beliefs([&decks[ia], &decks[ib]], &gdecks, &gdecks, OppView::Plan);
                    let mut pols = [fac.make(&models[0], seed ^ 0x51), fac.make(&models[1], seed ^ 0xA7)];
                    let mut genr = mk_roller(seed0 ^ gi ^ 0x7777);
                    let mut seen: Vec<u16> = Vec::new();
                    let mut spots: Vec<String> = Vec::new();
                    let (mut nspot, mut ninf, mut nroll) = (0u64, 0u64, 0u64);
                    let mut winner: Option<u8> = None;
                    let mut decs = 0u32;
                    loop {
                        decs += 1;
                        if decs > 6000 {
                            break;
                        }
                        match g.advance() {
                            Status::GameOver(r) => {
                                winner = result_seat(r);
                                break;
                            }
                            Status::NeedDecision(seat) => {
                                let sv = g.seat_view(seat);
                                let d = sv.decision().expect("decision");
                                let n = d.options.len();
                                let id = d.id;
                                if n == 1 {
                                    drop(sv);
                                    g.apply(id, 0).unwrap();
                                    continue;
                                }
                                let mut spot_json: Option<serde_json::Value> = None;
                                if seat.0 == 0 {
                                    if let Some(ci) = is_ending_cast(&d, ending) {
                                        let o = sv.observe();
                                        let has_opp_creature = o.battlefield.iter().any(|p| !p.controlled_by_me && p.types.contains(Types::CREATURE));
                                        if matches!(d.kind, ViewDecisionKind::Priority) && o.stack.is_empty() && has_opp_creature && o.me.life <= max_life && !seen.contains(&o.turn) {
                                            seen.push(o.turn);
                                            let all_plans = enumerate_plans(&db, &g, ci);
                                            let bp = best_plan(&all_plans);
                                            let opp_c: Vec<serde_json::Value> = o.battlefield.iter().filter(|p| !p.controlled_by_me && p.types.contains(Types::CREATURE)).map(|p| serde_json::json!({"name": p.name, "p": p.power, "t": p.toughness, "mv": mv_of(&db, p.def), "tapped": p.tapped})).collect();
                                            let opp_nonc: Vec<String> = o.battlefield.iter().filter(|p| !p.controlled_by_me && !p.types.contains(Types::CREATURE)).map(|p| p.name.clone()).collect();
                                            let my_hand: Vec<String> = o.me.hand.iter().map(|c| c.name.clone()).collect();
                                            let bf_me: Vec<String> = o.battlefield.iter().filter(|p| p.controlled_by_me).map(|p| p.name.clone()).collect();
                                            let cast_opts: Vec<String> = d.options.iter().filter(|x| x.kind == ActionKind::CastSpell).map(|x| x.label.clone()).collect();
                                            let after_opts: Vec<String> = match &bp {
                                                Some(p) => {
                                                    let mut g2 = g.clone();
                                                    for &i in &p.acts {
                                                        let seat = match g2.advance() {
                                                            Status::NeedDecision(s) => s,
                                                            Status::GameOver(_) => break,
                                                        };
                                                        let id = g2.seat_view(seat).decision().unwrap().id;
                                                        g2.apply(id, i).unwrap();
                                                    }
                                                    settle(&mut g2);
                                                    match g2.advance() {
                                                        Status::NeedDecision(Seat(0)) => g2.seat_view(Seat(0)).decision().map(|d| d.options.iter().filter(|x| x.kind == ActionKind::CastSpell).map(|x| x.label.clone()).collect()).unwrap_or_default(),
                                                        _ => Vec::new(),
                                                    }
                                                }
                                                None => Vec::new(),
                                            };
                                            let mut j = serde_json::json!({
                                                "cast_opts": cast_opts, "after_opts": after_opts,
                                                "matchup": nb, "game": gi, "first": first.0, "turn": o.turn, "step": format!("{:?}", o.step),
                                                "life": o.me.life, "olife": o.opp.life, "hand": my_hand, "bf": bf_me, "ocreatures": opp_c, "opermanents": opp_nonc,
                                                "ohand": o.opp.hand_count, "lib": o.me.library_count, "gy": o.me.graveyard.len(),
                                                "n_plans": all_plans.len(), "n_exiling": all_plans.iter().filter(|p| p.exiled).count(),
                                            });
                                            match &bp {
                                                None => {
                                                    ninf += 1;
                                                    j["feasible"] = serde_json::json!(false);
                                                }
                                                Some(p) => {
                                                    nspot += 1;
                                                    j["feasible"] = serde_json::json!(true);
                                                    j["target"] = serde_json::json!({"name": p.target, "mv": p.tmv, "p": p.tpow, "x": p.x});
                                                    let mut res = serde_json::Map::new();
                                                    let tt = o.turn;
                                                    for arm in ARMS {
                                                        let (mut wins, mut decided) = (0u32, 0u32);
                                                        for r in 0..reps {
                                                            let mut rl = mk_roller(seed0 ^ (gi << 24) ^ (nspot << 12) ^ r as u64 ^ 0xABCD);
                                                            let w = match arm {
                                                                Arm::Cast => rl.run(g.clone(), Some(p), &|_| false, None, 6000),
                                                                Arm::Wait => rl.run(g.clone(), None, &|t| t <= tt + 1, None, 6000),
                                                                Arm::Never => rl.run(g.clone(), None, &|_| true, None, 6000),
                                                                Arm::Free => rl.run(g.clone(), None, &|_| false, None, 6000),
                                                                Arm::Late1 => rl.run(g.clone(), None, &|t| t < tt + 2, Some(tt + 2), 6000),
                                                                Arm::Late2 => rl.run(g.clone(), None, &|t| t < tt + 4, Some(tt + 4), 6000),
                                                            };
                                                            nroll += 1;
                                                            if let Some(w) = w {
                                                                decided += 1;
                                                                wins += (w == 0) as u32;
                                                            }
                                                        }
                                                        res.insert(format!("{arm:?}").to_lowercase(), serde_json::json!({"wins": wins, "n": decided}));
                                                    }
                                                    j["arms"] = serde_json::Value::Object(res);
                                                }
                                            }
                                            spot_json = Some(j);
                                        }
                                    }
                                }
                                let i = if gen_net { genr.pick(&g, seat, n, &vec![false; n]) } else if seat.0 == 0 { pols[0].choose(&sv, n) } else { pols[1].choose(&sv, n) };
                                let mcts_cast = seat.0 == 0 && is_ending_cast(&d, ending) == Some(i);
                                drop(sv);
                                if let Some(mut j) = spot_json {
                                    j["mcts_cast"] = serde_json::json!(mcts_cast);
                                    spots.push(j.to_string());
                                }
                                g.apply(id, i).unwrap();
                            }
                        }
                    }
                    let mut t = tally.lock().unwrap();
                    t[0] += 1;
                    t[1] += (winner == Some(0)) as u64;
                    t[2] += nspot;
                    t[3] += ninf;
                    t[4] += nroll;
                    drop(t);
                    out.lock().unwrap().extend(spots);
                }
            });
        }
    });
    let t = tally.into_inner().unwrap();
    eprintln!(
        "{} vs {}: MCTS games {}, A won {} ({:.1}%); {} feasible spots, {} infeasible, {} rollouts; net-only baseline games {} A won {} ({:.1}%); {:.0}s",
        na,
        nb,
        t[0],
        t[1],
        100.0 * t[1] as f64 / t[0].max(1) as f64,
        t[2],
        t[3],
        t[4],
        t[5],
        t[6],
        100.0 * t[6] as f64 / t[5].max(1) as f64,
        t0.elapsed().as_secs_f64()
    );
    let lines = out.into_inner().unwrap();
    match flag("--out") {
        Some(p) => std::fs::write(p, lines.join("\n") + "\n").unwrap(),
        None => println!("{}", lines.join("\n")),
    }
}

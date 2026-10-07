//! Tough-spot flagger: plays the Alurentell bot (net-guided determinized MCTS) against a searching
//! opponent and records the decisions where the bot's choice is hard, so Brady can give input.
//!
//! Usage: tough <decks dir> --net w.bin --out DIR [--me alurentell] [--opp ur-cutter] [--opp-net w2.bin]
//!        [--games 20] [--iters 64] [--deep 400] [--opp-iters 32] [--seed 1] [--threads 4]
//!
//! Per bot decision (non-trivial, more than "pass or tap mana"):
//!   1. net priors and value on the bot's own observation;
//!   2. normal search (`--iters`), its best action is what the bot plays;
//!   3. if the position looks contested, two deeper searches (`--deep` iterations each, different
//!      seeds) give stable visit shares and values per option;
//!   4. a position is FLAGGED when the deep search finds two near-equal top options ("close") and/or the
//!      net's favourite is not the search's choice ("disagree").
//! The bot only ever acts on its `SeatView`; the real game is held by this loop to advance it, and the
//! output holds nothing but what the bot saw (rendered Observation) plus the bot's own numbers.
mod render;

use mtg_agent::*;
use mtg_core::decision::{GameResult, Status};
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_view::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Params {
    iters: u32,
    deep: u32,
    opp_iters: u32,
    seed: u64,
}

struct Flagged {
    json: serde_json::Value,
    score: f64,
}

#[derive(Default)]
struct GameStats {
    result: i32, // +1 bot win, -1 bot loss, 0 draw/truncated
    decisions: u32,
    contested: u32,
    flagged: u32,
    turns: u16,
}

fn boring(d: &Decision) -> bool {
    d.options.iter().all(|a| matches!(a.kind, ActionKind::Pass | ActionKind::ActivateMana))
}

fn tv(p: &[f32], q: &[f32]) -> f32 {
    p.iter().zip(q).map(|(a, b)| (a - b).abs()).sum::<f32>() / 2.0
}

fn shares(visits: &[u32]) -> Vec<f32> {
    let t: u32 = visits.iter().sum::<u32>().max(1);
    visits.iter().map(|&v| v as f32 / t as f32).collect()
}

fn play_game(
    db: &std::sync::Arc<mtg_core::card::CardDb>,
    decks: [&DeckList; 2],
    bot_seat: Seat,
    first: Seat,
    gseed: u64,
    me_net: &Arc<Net>,
    opp_net: &Arc<Net>,
    p: &Params,
    game_no: usize,
    out: &mut Vec<Flagged>,
) -> GameStats {
    let n_defs = db.defs.len();
    let mut g = Game::new(db.clone(), [decks[0], decks[1]], gseed, GameConfig { first_player: first, ..GameConfig::default() });
    let model = UniformConsistentModel { decks: [decks[0].main.clone(), decks[1].main.clone()] };
    let mut me_ev = NetEvaluator::new(me_net.clone(), n_defs);
    let mut peek = NetEvaluator::new(me_net.clone(), n_defs);
    let mut opp = MctsPolicy::new(
        &model,
        Box::new(NetEvaluator::new(opp_net.clone(), n_defs)),
        SearchConfig { iterations: p.opp_iters, seed: gseed + 11, ..SearchConfig::default() },
    );
    let mut st = GameStats::default();
    let mut pending: Vec<ViewEvent> = Vec::new();
    let mut history: Vec<(u8, usize)> = Vec::new();
    let mut ordinal = 0u32;
    let mut mine: Vec<Flagged> = Vec::new();
    let mut n_dec = 0u32;
    let mut buf: Vec<f32> = Vec::new();
    // Readable log of what the bot has seen so far this game (key events only).
    let mut game_log: Vec<String> = Vec::new();
    let result = loop {
        if n_dec > 6000 {
            break None;
        }
        match g.advance() {
            Status::GameOver(r) => break Some(r),
            Status::NeedDecision(seat) => {
                n_dec += 1;
                let sv = g.seat_view(seat);
                let obs = sv.observe();
                let d = obs.decision.clone().unwrap();
                st.turns = st.turns.max(obs.turn);
                if seat != bot_seat {
                    let idx = opp.choose(&sv, d.options.len());
                    history.push((seat.0, idx));
                    g.apply(d.id, idx).unwrap();
                    continue;
                }
                pending.extend(obs.events.iter().cloned());
                for e in obs.events.iter() {
                    let keep = matches!(e, ViewEvent::TurnBegan { .. } | ViewEvent::SpellCast { .. } | ViewEvent::LandPlayed { .. } | ViewEvent::AbilityActivated { .. } | ViewEvent::SpellCountered { .. } | ViewEvent::SpellFizzled { .. } | ViewEvent::LifeChange { .. } | ViewEvent::Damage { .. } | ViewEvent::TokenCreated { .. } | ViewEvent::MulliganTaken { .. } | ViewEvent::HandKept { .. } | ViewEvent::DrewCard { .. })
                        || matches!(e, ViewEvent::Moved { from: mtg_core::types::ZoneKind::Hand, to: mtg_core::types::ZoneKind::Library, owner_is_me: true, .. });
                    if let (true, Some(s)) = (keep, render::event_str(e)) {
                        if game_log.last() != Some(&s) {
                            game_log.push(s);
                        }
                    }
                }
                let n = d.options.len();
                if d.trivial || n == 1 {
                    history.push((seat.0, 0));
                    g.apply(d.id, 0).unwrap();
                    continue;
                }
                ordinal += 1;
                let ix = ordinal;
                // 1. net on the real observation
                let net_eval = peek.eval(&g, seat, n);
                // 2. normal search = what the bot plays
                let cfg = SearchConfig { iterations: p.iters, seed: gseed.wrapping_mul(1315423911).wrapping_add(ix as u64), ..SearchConfig::default() };
                let r1 = search(&sv, &model, &mut me_ev, &cfg);
                let choice = r1.best();
                let events_shown = std::mem::take(&mut pending);
                if !boring(&d) {
                    st.decisions += 1;
                    let s1 = shares(&r1.visits);
                    let mut order: Vec<usize> = (0..n).collect();
                    order.sort_by(|&a, &b| r1.visits[b].cmp(&r1.visits[a]));
                    let second = s1[order[1]];
                    let netargmax = (0..n).max_by(|&a, &b| net_eval.priors[a].partial_cmp(&net_eval.priors[b]).unwrap()).unwrap();
                    let is_mull = matches!(d.kind, ViewDecisionKind::Mulligan);
                    let contested = is_mull || r1.value.abs() < 0.92 && (second >= 0.12 || (netargmax != choice && tv(&net_eval.priors, &s1) >= 0.35));
                    if contested {
                        st.contested += 1;
                        // 3. deep search, two seeds
                        let mut visits = vec![0u32; n];
                        let mut wsum = vec![0f32; n];
                        let mut best_by_run = vec![];
                        for k in 0..2u64 {
                            let c = SearchConfig { iterations: p.deep, seed: cfg.seed ^ (0xABCDEF + k * 7919), ..SearchConfig::default() };
                            let r = search(&sv, &model, &mut me_ev, &c);
                            best_by_run.push(r.best());
                            for a in 0..n {
                                visits[a] += r.visits[a];
                                wsum[a] += r.q[a] * r.visits[a] as f32;
                            }
                        }
                        let q: Vec<f32> = (0..n).map(|a| if visits[a] > 0 { wsum[a] / visits[a] as f32 } else { f32::NAN }).collect();
                        let sh = shares(&visits);
                        let mut ord: Vec<usize> = (0..n).collect();
                        ord.sort_by(|&a, &b| visits[b].cmp(&visits[a]));
                        let (a1, a2) = (ord[0], ord[1]);
                        let gap = if q[a2].is_nan() { 1.0 } else { (q[a1] - q[a2]).abs() };
                        let runs_disagree = best_by_run[0] != best_by_run[1];
                        let close = sh[a2] >= 0.15 && (gap <= 0.05 || runs_disagree);
                        let net_top = (0..n).max_by(|&a, &b| net_eval.priors[a].partial_cmp(&net_eval.priors[b]).unwrap()).unwrap();
                        let dist = tv(&net_eval.priors, &sh);
                        let disagree = net_top != a1 && dist >= 0.40;
                        let tot: f32 = visits.iter().sum::<u32>().max(1) as f32;
                        let deep_value = wsum.iter().sum::<f32>() / tot;
                        let qs: Vec<f32> = q.iter().cloned().filter(|x| !x.is_nan()).collect();
                        let spread = qs.iter().cloned().fold(f32::NEG_INFINITY, f32::max) - qs.iter().cloned().fold(f32::INFINITY, f32::min);
                        let mull_check = is_mull && sh[a2] >= 0.10;
                        if close || disagree || mull_check {
                            st.flagged += 1;
                            let mut reasons = vec![];
                            if close {
                                reasons.push("close");
                            }
                            if disagree {
                                reasons.push("net-vs-search");
                            }
                            if mull_check {
                                reasons.push("mulligan-check");
                            }
                            if choice != a1 && sh[choice] < 0.15 {
                                reasons.push("quick-vs-deep");
                            }
                            // Interest: balanced top two, high stakes, strong disagreement.
                            let score = (sh[a1].min(sh[a2]) * 2.0) as f64 * (1.0 + spread as f64) + if disagree { dist as f64 } else { 0.0 };
                            encode_state(&obs, n_defs, &mut buf);
                            let sparse: Vec<(u32, f32)> = buf.iter().enumerate().filter(|(_, v)| **v != 0.0).map(|(i, v)| (i as u32, *v)).collect();
                            let feats: Vec<[u32; 5]> = encode_options(&obs).iter().map(|o| [o.kind as u32, o.decision as u32, o.subject_def as u32, o.subject_zone as u32, o.value]).collect();
                            let mut shown = obs.clone();
                            shown.events = events_shown;
                            let text = render::render(&shown, 0);
                            let opts: Vec<serde_json::Value> = (0..n)
                                .map(|i| {
                                    serde_json::json!({
                                        "idx": d.options[i].idx, "kind": format!("{:?}", d.options[i].kind), "label": d.options[i].label,
                                        "net_prior": net_eval.priors[i], "search_share": sh[i],
                                        "win_est": if q[i].is_nan() { serde_json::Value::Null } else { serde_json::json!((q[i] as f64 + 1.0) / 2.0) },
                                        "quick_share": s1[i],
                                    })
                                })
                                .collect();
                            let json = serde_json::json!({
                                "id": format!("g{game_no}-d{ix}"), "game": game_no, "game_seed": gseed, "bot_seat": bot_seat.0, "first": first.0,
                                "decision_ordinal": ix, "turn": obs.turn, "step": format!("{:?}", obs.step), "decision_kind": format!("{:?}", d.kind),
                                "context": d.context, "reasons": reasons, "score": score, "spread": spread, "tv_net_vs_search": dist,
                                "runs_disagree": runs_disagree, "deep_win_est": (deep_value as f64 + 1.0) / 2.0, "net_value": (net_eval.value as f64 + 1.0) / 2.0,
                                "bot_choice": choice, "deep_best": a1, "net_top": net_top,
                                "view": text, "game_log": game_log.iter().rev().take(60).rev().cloned().collect::<Vec<_>>(), "options": opts,
                                "train": { "n_defs": n_defs, "state_sparse": sparse, "option_feats": feats },
                                "replay": history.clone(),
                            });
                            mine.push(Flagged { json, score });
                        }
                    }
                }
                history.push((seat.0, choice));
                g.apply(d.id, choice).unwrap();
            }
        }
    };
    st.result = match result {
        Some(GameResult::Win(s)) => if s == bot_seat { 1 } else { -1 },
        _ => 0,
    };
    for f in &mut mine {
        f.json["game_result_for_bot"] = serde_json::json!(st.result);
        f.json["game_turns"] = serde_json::json!(st.turns);
    }
    out.extend(mine);
    st
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let flag = |name: &str| a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned();
    let dir = a.get(1).expect("decks dir").clone();
    let me_name = flag("--me").unwrap_or("alurentell".into());
    let opp_name = flag("--opp").unwrap_or("ur-cutter".into());
    let load = |p: String| Arc::new(Net::load(std::path::Path::new(&p)).unwrap_or_else(|e| panic!("{e}")));
    let me_net = load(flag("--net").expect("--net"));
    let opp_net = flag("--opp-net").map(load).unwrap_or_else(|| me_net.clone());
    let games: usize = flag("--games").and_then(|s| s.parse().ok()).unwrap_or(20);
    let threads: usize = flag("--threads").and_then(|s| s.parse().ok()).unwrap_or(4);
    let base: u64 = flag("--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
    let params = Params {
        iters: flag("--iters").and_then(|s| s.parse().ok()).unwrap_or(64),
        deep: flag("--deep").and_then(|s| s.parse().ok()).unwrap_or(400),
        opp_iters: flag("--opp-iters").and_then(|s| s.parse().ok()).unwrap_or(32),
        seed: base,
    };
    let out_dir = std::path::PathBuf::from(flag("--out").expect("--out"));
    std::fs::create_dir_all(&out_dir).unwrap();
    let db = mtg_cards::legacy::build();
    let (names, decks) = load_deck_dir(&db, &dir);
    let find = |n: &str| names.iter().position(|x| x == n).unwrap_or_else(|| panic!("deck {n} not in {names:?}"));
    let (me_i, opp_i) = (find(&me_name), find(&opp_name));
    let next = AtomicUsize::new(0);
    let all: Mutex<Vec<Flagged>> = Mutex::new(Vec::new());
    let stats: Mutex<Vec<(usize, GameStats)>> = Mutex::new(Vec::new());
    let t0 = std::time::Instant::now();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= games {
                    break;
                }
                let bot_seat = Seat((i % 2) as u8);
                let first = Seat(((i / 2) % 2) as u8);
                let (d0, d1) = if bot_seat == Seat(0) { (&decks[me_i], &decks[opp_i]) } else { (&decks[opp_i], &decks[me_i]) };
                let mut out = Vec::new();
                let st = play_game(&db, [d0, d1], bot_seat, first, params.seed * 100_000 + i as u64, &me_net, &opp_net, &params, i, &mut out);
                eprintln!("game {i}: bot {} in {} turns; {} decisions, {} contested, {} flagged ({:.0}s)", match st.result { 1 => "won", -1 => "lost", _ => "drew" }, st.turns, st.decisions, st.contested, st.flagged, t0.elapsed().as_secs_f64());
                all.lock().unwrap().extend(out);
                stats.lock().unwrap().push((i, st));
            });
        }
    });
    let mut all = all.into_inner().unwrap();
    all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let mut f = std::io::BufWriter::new(std::fs::File::create(out_dir.join("positions.jsonl")).unwrap());
    use std::io::Write;
    for p in &all {
        writeln!(f, "{}", p.json).unwrap();
    }
    let mut stats = stats.into_inner().unwrap();
    stats.sort_by_key(|s| s.0);
    let mut tsv = String::from("game\tbot_seat\tresult\tturns\tdecisions\tcontested\tflagged\n");
    for (i, s) in &stats {
        tsv += &format!("{i}\t{}\t{}\t{}\t{}\t{}\t{}\n", i % 2, s.result, s.turns, s.decisions, s.contested, s.flagged);
    }
    std::fs::write(out_dir.join("games.tsv"), tsv).unwrap();
    let (d, c, fl): (u32, u32, u32) = stats.iter().fold((0, 0, 0), |a, (_, s)| (a.0 + s.decisions, a.1 + s.contested, a.2 + s.flagged));
    let w = stats.iter().filter(|s| s.1.result == 1).count();
    println!("{games} games, bot (Alurentell) won {w}; {d} real decisions, {c} contested, {fl} flagged; {:.0}s", t0.elapsed().as_secs_f64());
}

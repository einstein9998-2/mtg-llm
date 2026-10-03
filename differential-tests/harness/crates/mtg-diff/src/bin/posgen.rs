//! Random play on the Rust engine, sampling empty-stack priority windows as differential positions.
//! Usage: posgen <deck dir> <games> <seed0> <per-game cap> [combat] > positions.jsonl
use mtg_core::decision::*;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_diff::pos::export;
use mtg_view::*;
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = a.get(1).cloned().unwrap_or("/mnt/project-files/decks".into());
    let games: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(50);
    let seed0: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let cap: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(5);
    // `combat`: emit only combat runs, from most main-phase windows (the per-game cap then counts combat runs)
    let combat_only = a.get(5).map_or(false, |m| m == "combat");
    // `response`: emit only stack responses (a spell, then the opponent answers it), from most windows where a spell is the chosen action
    let resp_only = a.get(5).map_or(false, |m| m == "response");
    let db = mtg_cards::legacy::build();
    let mut paths: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let mut decks = vec![];
    let mut names = vec![];
    for p in &paths {
        let (d, _) = mtg_fuzz::load_deck_file(&db, p);
        names.push(p.file_stem().unwrap().to_string_lossy().to_string());
        decks.push(d);
    }
    let pool = db;
    let n = decks.len() as u64;
    let mut skipped: BTreeMap<String, u64> = BTreeMap::new();
    let mut emitted = 0u64;
    let mut combats = 0u64;
    let mut responses = 0u64;
    for i in 0..games {
        let x = (i % n) as usize;
        let y = ((x as u64 + 1 + (i / n) % (n - 1)) % n) as usize;
        let seed = seed0 + i;
        let first = (i % 2) as u8;
        let cfg = GameConfig { first_player: Seat(first), ..GameConfig::default() };
        let mut g = Game::new(pool.clone(), [&decks[x], &decks[y]], seed, cfg);
        g.set_track_view_events(false);
        g.set_keep_events(true);
        let mut activated: std::collections::BTreeSet<String> = Default::default();
        let mut act_turn = 0u16;
        let mut rng = Pcg64::from_seed(seed ^ 0x51ED_C0DE);
        let mut got = 0usize;
        let mut trace: Vec<u32> = vec![];
        for step in 0..4000u32 {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(_) => {}
            }
            let td = g.raw_state().turn_data();
            if td.turn != act_turn {
                act_turn = td.turn;
                activated.clear();
            }
            for e in g.take_events() {
                if let mtg_core::event::Event::AbilityActivated { source, .. } = e {
                    activated.insert(mtg_diff::util::def_name(&pool, g.raw_state(), source));
                }
            }
            let p = g.pending().unwrap().clone();
            if matches!(p.kind, DecisionKind::Priority) && got < cap && rng.below(100) < if combat_only || resp_only { 70 } else { 20 } {
                match export(&g, &activated) {
                    Ok(pos) => {
                        if !combat_only && !resp_only {
                            got += 1;
                        }
                        emitted += 1;
                        let id = format!("{}-{}-s{}-d{}", names[x], names[y], seed, step);
                        // the action the step differential plays: a uniformly chosen non-pass option whose Forge counterpart is unambiguous
                        let cand: Vec<usize> = (0..p.options.len()).filter(|&i| pos.option_keys[i].as_ref().map_or(false, |k| !k.starts_with("playback|") && !k.starts_with("cast|Exile|") && !k.starts_with("cast|Graveyard|Nethergoyf|") && !(k.starts_with("act|") && !k.ends_with("|1")))).collect();
                        let (action, action_idx) = if cand.is_empty() { (serde_json::Value::Null, serde_json::Value::Null) } else {
                            let i = cand[rng.below(cand.len() as u64) as usize];
                            (serde_json::json!(pos.option_keys[i]), serde_json::json!(i))
                        };
                        let j = serde_json::json!({
                            "id": id, "seed": seed, "decks": [names[x], names[y]], "turn": pos.turn, "step": pos.step, "active": pos.active, "window": pos.window, "priority": pos.priority,
                            "state": pos.state, "rust_actions": pos.actions, "summary": pos.summary, "activated": pos.activated, "first": first, "trace": trace, "action": action, "action_idx": action_idx,
                        });
                        if !combat_only && !resp_only {
                            println!("{j}");
                        }
                        // a stack response: play a spell, then the opponent answers it with a random available non-pass action
                        if !combat_only && action.as_str().map_or(false, |k| k.starts_with("cast|")) && rng.below(100) < if resp_only { 100 } else { 60 } {
                            let mut c = g.clone();
                            if mtg_diff::step::advance_to_response(&mut c, action_idx.as_u64().unwrap() as usize).is_some() {
                                if let Ok((_, keys)) = mtg_diff::pos::window_keys(&c) {
                                    let cand: Vec<&String> = keys.iter().flatten().filter(|k| !k.starts_with("playback|") && !k.starts_with("play|") && !k.starts_with("cast|Exile|") && !(k.starts_with("act|") && !k.ends_with("|1"))).collect();
                                    if !cand.is_empty() {
                                        let k = cand[rng.below(cand.len() as u64) as usize].clone();
                                        let mut jr = j.clone();
                                        jr["id"] = serde_json::json!(format!("{id}-resp"));
                                        jr["response"] = serde_json::json!(k);
                                        println!("{jr}");
                                        responses += 1;
                                        if resp_only {
                                            got += 1;
                                        }
                                    }
                                }
                            }
                        }
                        // a combat run: from the active player's first main-phase window, pass into combat and attack with a random subset
                        if !resp_only && pos.step == "MAIN1" && pos.window == 0 && pos.priority == pos.active {
                            let mut c = g.clone();
                            let pass = p.options.iter().position(|o| matches!(o, Opt::Pass)).unwrap();
                            c.apply(p.id, pass).unwrap();
                            let mut atk: Vec<String> = vec![];
                            for _ in 0..200 {
                                if matches!(c.advance(), Status::GameOver(_)) {
                                    break;
                                }
                                c.take_events();
                                let q = c.pending().unwrap().clone();
                                let st = c.raw_state();
                                match q.kind {
                                    DecisionKind::Priority => {
                                        if st.turn_data().step != mtg_core::types::Step::Main1 && st.turn_data().step != mtg_core::types::Step::BeginCombat {
                                            break;
                                        }
                                        let i = q.options.iter().position(|o| matches!(o, Opt::Pass)).unwrap_or(0);
                                        c.apply(q.id, i).unwrap();
                                    }
                                    DecisionKind::DeclareAttacker { creature } => {
                                        let yes = rng.below(100) < 55;
                                        let at = q.options.iter().position(|o| matches!(o, Opt::Attack(mtg_core::decision::AttackTarget::Player(_))));
                                        let no = q.options.iter().position(|o| matches!(o, Opt::NoAttack));
                                        let i = match (yes, at, no) {
                                            (true, Some(a), _) | (false, Some(a), None) => a,
                                            (_, _, Some(n)) => n,
                                            _ => 0,
                                        };
                                        if matches!(q.options[i], Opt::Attack(_)) {
                                            atk.push(mtg_diff::util::creature_label(&pool, st, creature));
                                        }
                                        c.apply(q.id, i).unwrap();
                                    }
                                    _ => break,
                                }
                            }
                            if !atk.is_empty() {
                                atk.sort();
                                let mut jc = j.clone();
                                jc["id"] = serde_json::json!(format!("{id}-combat"));
                                jc["action"] = serde_json::json!(format!("combat|{}", atk.join(";")));
                                jc["action_idx"] = serde_json::json!(pass);
                                println!("{jc}");
                                combats += 1;
                                if combat_only {
                                    got += 1;
                                }
                            }
                        }
                    }
                    Err(e) => *skipped.entry(e).or_insert(0) += 1,
                }
            }
            // biased random policy: mostly act at priority, otherwise uniform
            let idx = if matches!(p.kind, DecisionKind::Priority) && p.options.len() > 1 && rng.below(100) < 88 {
                1 + rng.below(p.options.len() as u64 - 1) as usize
            } else {
                rng.below(p.options.len() as u64) as usize
            };
            // option 0 at priority is Pass; guard against the engine ordering differently
            let idx = if matches!(p.kind, DecisionKind::Priority) && !matches!(p.options[0], Opt::Pass) { rng.below(p.options.len() as u64) as usize } else { idx };
            g.apply(p.id, idx).unwrap();
            trace.push(idx as u32);
        }
    }
    eprintln!("emitted {emitted} (combat runs {combats}, responses {responses}); skipped {skipped:?}");
}

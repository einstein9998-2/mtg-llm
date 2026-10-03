//! Start positions for lockstep lines.
//!   linegen scn <file.scn|dir>... --seeds N [--max M]      one line per scenario state and policy seed (states are Forge GameState text)
//!   linegen random <deck dir> <games> <seed0> <per-game cap> [--max M]   priority windows of random Rust games (replayed from their trace)
//! Positions go to stdout as JSON lines; the Forge probe plays each (`mode = line`) and `linediff` replays it on the engine.
use mtg_core::decision::*;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_diff::pos::export;
use mtg_view::*;

fn flag(a: &[String], k: &str, d: u64) -> u64 {
    a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d)
}

fn hand_names(state: &[String]) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    for l in state {
        if let Some((k, val)) = l.split_once('=') {
            if k.ends_with("hand") {
                for n in val.split(';').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                    if !v.contains(&n.to_string()) {
                        v.push(n.to_string());
                    }
                }
            }
        }
    }
    v
}

/// The scenario harness's state completion: p1/p2 prefixes, 20 life, ten Islands, turn 3, main phase 1, no summoning sickness.
fn normalize(state: &[String], swap: bool) -> Vec<String> {
    let (k1, k2) = if swap { ("ai", "human") } else { ("human", "ai") };
    let mut lines: Vec<String> = vec![];
    for s in state {
        let mut t = s.replace("activeplayer=p1", &format!("activeplayer={k1}")).replace("activeplayer=p2", &format!("activeplayer={k2}"));
        if let Some(r) = t.strip_prefix("human") {
            t = format!("@H{r}");
        } else if let Some(r) = t.strip_prefix("ai") {
            t = format!("@A{r}");
        }
        if let Some(r) = t.strip_prefix("p1") {
            t = format!("{k1}{r}");
        } else if let Some(r) = t.strip_prefix("p2") {
            t = format!("{k2}{r}");
        }
        if let Some(r) = t.strip_prefix("@H") {
            t = format!("{k1}{r}");
        } else if let Some(r) = t.strip_prefix("@A") {
            t = format!("{k2}{r}");
        }
        lines.push(t);
    }
    // Forge keeps only the last of a repeated key; the engine's scenario loader merges them. Merge zone lists here so both see the same state.
    let mut merged: Vec<String> = vec![];
    for l in lines {
        let key = l.split('=').next().unwrap_or("").to_string();
        let zone = ["hand", "battlefield", "graveyard", "exile", "library"].iter().any(|z| key.ends_with(z));
        match merged.iter_mut().find(|m| zone && m.split('=').next() == Some(key.as_str())) {
            Some(m) => {
                let add = l[key.len() + 1..].to_string();
                if m.ends_with('=') {
                    m.push_str(&add);
                } else if !add.is_empty() {
                    m.push(';');
                    m.push_str(&add);
                }
            }
            None => merged.push(l),
        }
    }
    let mut lines = merged;
    if !lines.iter().any(|l| l.starts_with("removesummoningsickness")) {
        lines.push("removesummoningsickness=true".into());
    }
    for side in ["human", "ai"] {
        if !lines.iter().any(|l| l.starts_with(&format!("{side}life="))) {
            lines.push(format!("{side}life=20"));
        }
        if !lines.iter().any(|l| l.starts_with(&format!("{side}library="))) {
            lines.push(format!("{side}library={}", vec!["Island"; 10].join(";")));
        }
    }
    if !lines.iter().any(|l| l.starts_with("turn=")) {
        lines.push("turn=3".into());
    }
    if !lines.iter().any(|l| l.starts_with("activeplayer=")) {
        lines.push(format!("activeplayer={k1}"));
    }
    if !lines.iter().any(|l| l.starts_with("activephase=")) {
        lines.push("activephase=MAIN1".into());
    }
    lines
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let max = flag(&a, "--max", 12);
    match a.first().map(|s| s.as_str()) {
        Some("scn") => {
            let seeds = flag(&a, "--seeds", 8);
            let mut files = vec![];
            let mut i = 1;
            while i < a.len() {
                if a[i].starts_with("--") {
                    i += 2;
                    continue;
                }
                let p = std::path::Path::new(&a[i]);
                if p.is_dir() {
                    let mut fs: Vec<_> = std::fs::read_dir(p).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "scn")).collect();
                    fs.sort();
                    files.extend(fs);
                } else {
                    files.push(p.to_path_buf());
                }
                i += 1;
            }
            for f in files {
                for sc in mtg_diff::scn::parse_file(&f) {
                    if sc.state.is_empty() {
                        continue;
                    }
                    let state = normalize(&sc.state, sc.swap);
                    let focus = hand_names(&state);
                    for s in 0..seeds {
                        let j = serde_json::json!({
                            "id": format!("scn:{}:{}:l{}", sc.file, sc.name, s), "seed": 7, "mode": "line", "line_seed": 1000 + s, "line_max": max, "focus": if s % 4 == 3 { vec![] } else { focus.clone() },
                            "state": state, "window": 0, "from_state": true,
                        });
                        println!("{j}");
                    }
                }
            }
        }
        Some("random") => {
            let dir = a.get(1).cloned().unwrap();
            let games: u64 = a[2].parse().unwrap();
            let seed0: u64 = a[3].parse().unwrap();
            let cap: usize = a[4].parse().unwrap();
            let db = mtg_cards::legacy::build();
            let mut paths: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
            paths.sort();
            let (mut decks, mut names) = (vec![], vec![]);
            for p in &paths {
                let (d, _) = mtg_fuzz::load_deck_file(&db, p);
                names.push(p.file_stem().unwrap().to_string_lossy().to_string());
                decks.push(d);
            }
            let n = decks.len() as u64;
            let mut emitted = 0u64;
            for i in 0..games {
                let x = (i % n) as usize;
                let y = ((x as u64 + 1 + (i / n) % (n - 1)) % n) as usize;
                let seed = seed0 + i;
                let first = (i % 2) as u8;
                let cfg = GameConfig { first_player: Seat(first), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[x], &decks[y]], seed, cfg);
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
                            activated.insert(mtg_diff::util::def_name(&db, g.raw_state(), source));
                        }
                    }
                    let p = g.pending().unwrap().clone();
                    // lines start at an empty-stack main-phase window of the active player, where there is something to do
                    if matches!(p.kind, DecisionKind::Priority) && got < cap && g.raw_state().stack().is_empty() && p.options.len() > 2 && rng.below(100) < 25 {
                        if let Ok(pos) = export(&g, &activated) {
                            if (pos.step == "MAIN1" || pos.step == "MAIN2") && pos.window == 0 {
                                got += 1;
                                emitted += 1;
                                let hand: Vec<String> = hand_names(&pos.state);
                                let ls = rng.below(1_000_000);
                                let j = serde_json::json!({
                                    "id": format!("rnd:{}-{}-s{}-d{}", names[x], names[y], seed, step), "seed": seed, "decks": [names[x], names[y]], "first": first, "trace": trace, "mode": "line",
                                    "line_seed": ls, "line_max": max, "focus": if ls % 2 == 0 { hand } else { vec![] }, "state": pos.state, "window": pos.window, "activated": pos.activated,
                                });
                                println!("{j}");
                            }
                        }
                    }
                    let idx = if matches!(p.kind, DecisionKind::Priority) && p.options.len() > 1 && rng.below(100) < 88 { 1 + rng.below(p.options.len() as u64 - 1) as usize } else { rng.below(p.options.len() as u64) as usize };
                    let idx = if matches!(p.kind, DecisionKind::Priority) && !matches!(p.options[0], Opt::Pass) { rng.below(p.options.len() as u64) as usize } else { idx };
                    g.apply(p.id, idx).unwrap();
                    trace.push(idx as u32);
                }
            }
            eprintln!("emitted {emitted}");
        }
        _ => eprintln!("usage: linegen scn <file|dir>... --seeds N | linegen random <decks> <games> <seed0> <cap> [--max M]"),
    }
}

//! Batched environment server: JSON lines on stdin, JSON lines on stdout (protocol in
//! `crates/mtg-agent/python/mtg_env.py` and `PHASE3-HANDOFF.md`). Usage: envserver <decks dir>
use mtg_agent::*;
use mtg_view::{SCALAR_NAMES, N_BLOCKS};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn out_json(outs: &[Out]) -> Value {
    let mut obs = Vec::new();
    let mut done = Vec::new();
    for o in outs {
        match o {
            Out::Obs(o) => obs.push(json!({
                "g": o.game, "seat": o.seat.idx(), "n": o.options.len(),
                "state": o.state.iter().map(|(i, v)| json!([i, v])).collect::<Vec<_>>(),
                "options": o.options.iter().map(|x| json!([x.kind, x.decision, x.subject_def, x.subject_zone, x.value])).collect::<Vec<_>>(),
            })),
            Out::Done(d) => done.push(json!({"g": d.game, "result": d.result, "decks": [d.decks.0, d.decks.1], "first": d.first, "decisions": d.decisions, "truncated": d.truncated})),
        }
    }
    json!({"ok": true, "obs": obs, "done": done})
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "decks".into());
    let db = mtg_cards::legacy::build();
    let (names, decks) = load_deck_dir(&db, &dir);
    let mut env: Option<BatchEnv> = None;
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("stdin");
        if line.trim().is_empty() {
            continue;
        }
        let reply = (|| -> Result<Value, String> {
            let v: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            match v["cmd"].as_str().unwrap_or("") {
                "meta" => Ok(json!({
                    "ok": true, "decks": names, "n_defs": db.defs.len(), "n_blocks": N_BLOCKS,
                    "n_scalars": SCALAR_NAMES.len(), "scalar_names": SCALAR_NAMES.to_vec(),
                    "state_len": mtg_view::state_len(db.defs.len()),
                    "defs": db.defs.iter().map(|d| d.name.to_string()).collect::<Vec<_>>(),
                })),
                "start" => {
                    let mut cfg = EnvConfig::default();
                    if let Some(n) = v["n_games"].as_u64() {
                        cfg.n_games = n as usize;
                    }
                    if let Some(s) = v["seed"].as_u64() {
                        cfg.seed0 = s;
                    }
                    if let Some(b) = v["autoreset"].as_bool() {
                        cfg.autoreset = b;
                    }
                    if let Some(m) = v["max_decisions"].as_u64() {
                        cfg.max_decisions = m as u32;
                    }
                    if let Some(p) = v["pairs"].as_array() {
                        for x in p {
                            let (a, b) = (x[0].as_u64().ok_or("bad pair")? as usize, x[1].as_u64().ok_or("bad pair")? as usize);
                            if a >= decks.len() || b >= decks.len() {
                                return Err("deck index out of range".into());
                            }
                            cfg.pairs.push((a, b));
                        }
                    }
                    let mut e = BatchEnv::new(db.clone(), decks.clone(), cfg);
                    let outs = e.start();
                    env = Some(e);
                    Ok(out_json(&outs))
                }
                "step" => {
                    let e = env.as_mut().ok_or("send start first")?;
                    let mut acts = Vec::new();
                    for a in v["actions"].as_array().ok_or("actions must be a list")? {
                        acts.push((a[0].as_u64().ok_or("bad action")? as usize, a[1].as_u64().ok_or("bad action")? as usize));
                    }
                    Ok(out_json(&e.step(&acts)?))
                }
                "quit" => std::process::exit(0),
                c => Err(format!("unknown cmd {c:?}")),
            }
        })();
        let reply = reply.unwrap_or_else(|e| json!({"ok": false, "error": e}));
        writeln!(stdout, "{reply}").unwrap();
        stdout.flush().unwrap();
    }
}

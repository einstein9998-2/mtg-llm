//! Step differential: play each position's action in Rust and Forge, compare resulting canonical summaries.
//! Usage: stepdiff <positions.jsonl> <forge-step.jsonl> <deck dir> [out.jsonl]
use mtg_diff::pos::summarize;
use mtg_diff::step::*;
use mtg_diff::step::StepResult;
use std::collections::BTreeMap;

fn norm(s: &str) -> BTreeMap<String, String> {
    // "pre life=.. hand=.. bf=.. gy=.. ex=.. lib=N"
    let mut m = BTreeMap::new();
    let marks = [" life=", " hand=", " bf=", " gy=", " ex=", " lib="];
    let mut pos: Vec<(usize, &str)> = marks.iter().filter_map(|k| s.find(k).map(|i| (i, *k))).collect();
    pos.sort();
    for (n, (i, k)) in pos.iter().enumerate() {
        let end = pos.get(n + 1).map_or(s.len(), |x| x.0);
        m.insert(k.trim().trim_end_matches('=').to_string(), s[i + k.len()..end].to_string());
    }
    let bf = m.get("bf").cloned().unwrap_or_default();
    let mut toks: Vec<String> = vec![];
    let mut tapped_lands = 0;
    for t in bf.split(';').filter(|t| !t.is_empty()) {
        let mut t = t.replace("|SummonSick", "");
        if t.contains("^L") && t.contains("|Tapped") {
            tapped_lands += 1;
            t = t.replace("|Tapped", "");
        }
        if t.contains("^T") {
            // token names differ in decoration between the engines: compare by the base name only
            let base = t.split("^T").next().unwrap_or("").split(" Token").next().unwrap_or("").trim().to_lowercase();
            t = format!("token:{}{}", base, t.split("^T").nth(1).unwrap_or(""));
        }
        toks.push(t);
    }
    toks.sort();
    m.insert("bf".into(), toks.join(";"));
    m.insert("tapped_lands".into(), tapped_lands.to_string());
    let mut hand: Vec<&str> = m["hand"].split(';').filter(|x| !x.is_empty()).collect();
    hand.sort();
    m.insert("hand".into(), hand.join(";"));
    m
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let pos_path = &a[1];
    let forge_path = &a[2];
    let dir = a.get(3).cloned().unwrap_or("/mnt/project-files/decks".into());
    let out_path = a.get(4).cloned();
    let db = mtg_cards::legacy::build();
    let mut decks = std::collections::BTreeMap::new();
    for e in std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
        let p = e.path();
        if p.extension().map_or(false, |x| x == "txt") {
            let (d, _) = mtg_fuzz::load_deck_file(&db, &p);
            decks.insert(p.file_stem().unwrap().to_string_lossy().to_string(), d);
        }
    }
    let forge: std::collections::HashMap<String, serde_json::Value> = std::fs::read_to_string(forge_path).unwrap().lines().filter(|l| !l.trim().is_empty()).map(|l| {
        let v: serde_json::Value = serde_json::from_str(l).unwrap();
        (v["id"].as_str().unwrap().to_string(), v)
    }).collect();
    let mut out = vec![];
    let (mut same, mut diff, mut skipped, mut ferr, mut failed, mut pay_noise) = (0, 0, 0, 0, 0, 0);
    let mut by_card: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for l in std::fs::read_to_string(pos_path).unwrap().lines() {
        let p: serde_json::Value = match serde_json::from_str(l) { Ok(v) => v, Err(_) => continue };
        if p["action"].is_null() {
            continue;
        }
        let id = p["id"].as_str().unwrap().to_string();
        let Some(f) = forge.get(&id) else { skipped += 1; continue };
        if let Some(e) = f.get("error") {
            ferr += 1;
            out.push(serde_json::json!({"id": id, "status": "forge_error", "error": e, "action": p["action"]}));
            continue;
        }
        if f["notes"].as_array().map_or(false, |n| n.iter().any(|x| x.as_str().map_or(false, |t| t.contains("play FAILED")))) {
            out.push(serde_json::json!({"id": id, "status": "forge_play_failed", "action": p["action"], "forge_notes": f["notes"]}));
            failed += 1;
            continue;
        }
        let ds: Vec<String> = p["decks"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let trace: Vec<u32> = p["trace"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u32).collect();
        let mut g = match rebuild(db_arc(&db), [&decks[&ds[0]], &decks[&ds[1]]], p["seed"].as_u64().unwrap(), p["first"].as_u64().unwrap() as u8, &trace) {
            Ok(g) => g,
            Err(e) => { out.push(serde_json::json!({"id": id, "status": "rebuild_error", "error": e})); continue }
        };
        let log: Vec<String> = f["log"].as_array().map(|v| v.iter().map(|x| x.as_str().unwrap().to_string()).collect()).unwrap_or_default();
        let dmg_sources = {
            let st = g.raw_state();
            let db = g.db();
            st.battlefield().iter().any(|&r| matches!(db.def(st.def_of(r)).name.as_str(), "Ancient Tomb" | "City of Traitors"))
        };
        let fs: Vec<String> = f["forge_summary"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let act = p["action"].as_str().unwrap();
        let combat = Some(act).filter(|a| a.starts_with("combat|"));
        let response = p["response"].as_str();
        let (res, rs, mut diffs, pn) = play_compare(&mut g, p["action_idx"].as_u64().unwrap() as usize, &log, combat, response, &fs, dmg_sources, act);
        pay_noise += pn;
        // a mismatch against the replayed game may come from history Forge cannot be given (durable effects, delayed triggers):
        // play the same action from a game built out of the injected state text instead
        let mut injected_note = serde_json::Value::Null;
        let mut history_artifact = false;
        if !diffs.is_empty() && res.unmatched.is_empty() && res.leftover.is_empty() {
            let state: Vec<String> = p["state"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
            match reinject(&db, &state, p["window"].as_u64().unwrap_or(0) as u8, act, p["activated"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default()) {
                Ok((mut g2, idx2)) => {
                    let (res2, _rs2, diffs2, _) = play_compare(&mut g2, idx2, &log, combat, response, &fs, dmg_sources, act);
                    if res2.unmatched.is_empty() && res2.leftover.is_empty() && diffs2.is_empty() {
                        history_artifact = true;
                    }
                    injected_note = serde_json::json!({"diffs": diffs2, "unmatched": res2.unmatched});
                }
                Err(e) => injected_note = serde_json::json!({"error": e}),
            }
        }
        let key = p["action"].as_str().unwrap().to_string();
        let card = key.split('|').nth(2).unwrap_or("").to_string();
        let e = by_card.entry(format!("{}|{}", key.split('|').next().unwrap(), card)).or_insert((0, 0));
        if diffs.is_empty() || history_artifact {
            same += 1;
            e.0 += 1;
        } else {
            diff += 1;
            e.1 += 1;
        }
        out.push(serde_json::json!({
            "id": id, "status": if diffs.is_empty() { "match" } else if !(res.unmatched.is_empty() && res.leftover.is_empty()) { "unaligned" } else if history_artifact { "history_artifact" } else { "mismatch" }, "injected": injected_note, "action": key, "diffs": diffs,
            "unmatched": res.unmatched, "leftover": res.leftover, "aligned": res.unmatched.is_empty() && res.leftover.is_empty(), "shuffled": res.shuffled, "rust_trace": res.trace, "forge_notes": f["notes"], "forge_log": log,
            "rust_summary": rs, "forge_summary": fs, "resp_rust": res.window_actions, "resp_forge": f["forge_actions"],
        }));
    }
    println!("same {same}  mismatch {diff}  forge-errors {ferr}  forge-play-failed {failed}  payment-noise-suppressed {pay_noise}  no-forge-result {skipped}");
    let mut v: Vec<_> = by_card.iter().filter(|(_, c)| c.1 > 0).collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(c.1));
    println!("mismatches by action:");
    for (k, c) in v.iter().take(60) {
        println!("  {:4} / {:4}  {k}", c.1, c.0 + c.1);
    }
    if let Some(p) = out_path {
        std::fs::write(p, out.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n") + "\n").unwrap();
    }
}

/// Plays the action on `g` following the Forge log and compares canonical summaries.
fn play_compare(g: &mut mtg_view::Game, action_idx: usize, log: &[String], combat: Option<&str>, response: Option<&str>, fs: &[String], dmg_sources: bool, act: &str) -> (StepResult, Vec<String>, Vec<String>, u32) {
    let res = run_step(g, action_idx, log, combat, response);
    let rs = summarize(g);
    let mut diffs = vec![];
    let mut pay_noise = 0;
    for (r, fo) in rs.iter().zip(fs.iter()) {
        let (nr, nf) = (norm(r), norm(fo));
        for (k, v) in &nr {
            // Rust and Forge may tap different lands for the same cost; a damaging or sacrificial source (Ancient Tomb, City of Traitors)
            // then changes life, tapped-land counts and the battlefield
            if dmg_sources && matches!(k.as_str(), "life" | "tapped_lands") && !act.starts_with("play|") && !act.starts_with("combat|") {
                if nf.get(k) != Some(v) {
                    pay_noise += 1;
                }
                continue;
            }
            // after a shuffle the two libraries are in different (random) orders: card identities that came from the library cannot be compared,
            // so compare zone sizes instead of contents
            let weak = res.shuffled && matches!(k.as_str(), "hand" | "gy" | "ex" | "bf");
            let (a, b) = (v.clone(), nf.get(k).cloned().unwrap_or_default());
            let (a, b) = if weak { (a.split(';').filter(|x| !x.is_empty()).count().to_string(), b.split(';').filter(|x| !x.is_empty()).count().to_string()) } else { (a, b) };
            if a != b {
                diffs.push(format!("{} {k}{}: rust [{}] forge [{}]", &r[..r.find(' ').unwrap()], if weak { " (size)" } else { "" }, a, b));
            }
        }
    }
    (res, rs, diffs, pay_noise)
}

/// A Rust game built from the injected state lines, at the position's priority window, with the index of the action's option.
fn reinject(db: &std::sync::Arc<mtg_core::card::CardDb>, state: &[String], window: u8, act: &str, activated: std::collections::BTreeSet<String>) -> Result<(mtg_view::Game, usize), String> {
    use mtg_core::decision::*;
    let mut g = mtg_diff::scn::game_from_state(db, state)?;
    if window == 1 {
        let p = g.pending().ok_or("no pending")?.clone();
        let pass = p.options.iter().position(|o| matches!(o, Opt::Pass)).ok_or("no pass")?;
        g.apply(p.id, pass).map_err(|e| format!("{e:?}"))?;
        g.advance();
    }
    let p = g.pending().ok_or("no pending")?.clone();
    if !matches!(p.kind, DecisionKind::Priority) {
        return Err(format!("injected game starts at {:?}", p.kind));
    }
    if act.starts_with("combat|") {
        let pass = p.options.iter().position(|o| matches!(o, Opt::Pass)).ok_or("no pass")?;
        return Ok((g, pass));
    }
    let pos = mtg_diff::pos::export(&g, &activated)?;
    let i = pos.option_keys.iter().position(|k| k.as_deref() == Some(act)).ok_or_else(|| format!("action {act} not offered by the injected game"))?;
    Ok((g, i))
}

fn db_arc(db: &std::sync::Arc<mtg_core::card::CardDb>) -> std::sync::Arc<mtg_core::card::CardDb> {
    db.clone()
}

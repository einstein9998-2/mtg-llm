//! Lockstep line differential driver. Usage: linediff <positions.jsonl> <forge-line.jsonl> <deck dir> <out.jsonl>
use mtg_diff::line::*;
use mtg_diff::step::rebuild;
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (pos_path, forge_path, dir, out_path) = (&a[1], &a[2], a[3].clone(), a[4].clone());
    let db = mtg_cards::legacy::build();
    let mut decks = BTreeMap::new();
    for e in std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
        let p = e.path();
        if p.extension().map_or(false, |x| x == "txt") {
            let (d, _) = mtg_fuzz::load_deck_file(&db, &p);
            decks.insert(p.file_stem().unwrap().to_string_lossy().to_string(), d);
        }
    }
    let forge: std::collections::HashMap<String, serde_json::Value> = std::fs::read_to_string(forge_path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            (v["id"].as_str().unwrap().to_string(), v)
        })
        .collect();
    let mut out = vec![];
    let mut by_status: BTreeMap<String, u32> = BTreeMap::new();
    let (mut total_windows, mut total_played, mut menu_diff_windows) = (0usize, 0usize, 0usize);
    for l in std::fs::read_to_string(pos_path).unwrap().lines() {
        let Ok(p) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        let id = p["id"].as_str().unwrap().to_string();
        let Some(f) = forge.get(&id) else {
            *by_status.entry("no_forge_result".into()).or_default() += 1;
            continue;
        };
        let mut rec = serde_json::json!({"id": id});
        if let Some(e) = f.get("error") {
            *by_status.entry("forge_error".into()).or_default() += 1;
            rec["status"] = "forge_error".into();
            rec["error"] = e.clone();
            out.push(rec);
            continue;
        }
        let windows = parse_windows(&f["windows"]);
        let flog: Vec<String> = f["log"].as_array().map(|v| v.iter().map(|x| x.as_str().unwrap().to_string()).collect()).unwrap_or_default();
        let over = f["over"].as_bool().unwrap_or(false);
        let winner = f["winner"].as_str().unwrap_or("?").to_string();
        let fin: Vec<String> = f["final_summary"].as_array().map(|v| v.iter().map(|x| x.as_str().unwrap().to_string()).collect()).unwrap_or_default();
        let state: Vec<String> = p["state"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let make = || -> Result<mtg_view::Game, String> {
            if p["from_state"].as_bool().unwrap_or(false) {
                mtg_diff::scn::game_from_state(&db, &state)
            } else {
                let ds: Vec<String> = p["decks"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
                let trace: Vec<u32> = p["trace"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u32).collect();
                rebuild(db.clone(), [&decks[&ds[0]], &decks[&ds[1]]], p["seed"].as_u64().unwrap(), p["first"].as_u64().unwrap() as u8, &trace)
            }
        };
        {
            let mut l = mtg_diff::step::LEGEND.lock().unwrap();
            l.picks = vec![];
            l.counter = 0;
            l.options.clear();
        }
        let mut g = match make() {
            Ok(g) => g,
            Err(e) => {
                rec["status"] = "rebuild_error".into();
                rec["error"] = e.into();
                *by_status.entry("rebuild_error".into()).or_default() += 1;
                out.push(rec);
                continue;
            }
        };
        let mut r = run_line(&mut g, &windows, &flog, over, &fin, &winner);
        // identically named legendary copies: Forge's log cannot say which one it kept; if the line fails, retry with the other picks
        let opts = mtg_diff::step::LEGEND.lock().unwrap().options.clone();
        if matches!(r.ending, Ending::Diverged | Ending::Unaligned) && !opts.is_empty() {
            let radices: Vec<usize> = opts.iter().take(4).map(|&n| n.clamp(1, 3)).collect();
            let total: usize = radices.iter().product();
            for code in 1..total.min(24) {
                let mut c = code;
                let picks: Vec<usize> = radices.iter().map(|&rd| { let d = c % rd; c /= rd; d }).collect();
                mtg_diff::step::LEGEND.lock().unwrap().picks = picks;
                let Ok(mut g2) = make() else { break };
                let r2 = run_line(&mut g2, &windows, &flog, over, &fin, &winner);
                if matches!(r2.ending, Ending::Complete | Ending::GameOver) {
                    r = r2;
                    break;
                }
            }
            mtg_diff::step::LEGEND.lock().unwrap().picks = vec![];
        }
        // a divergence on a game replayed from its trace may come from history Forge cannot be given: replay the line from the injected text
        let mut history_artifact = false;
        let mut injected = serde_json::Value::Null;
        if r.ending == Ending::Diverged && !p["from_state"].as_bool().unwrap_or(false) && p["window"].as_u64().unwrap_or(0) == 0 {
            if let Ok(mut g2) = mtg_diff::scn::game_from_state(&db, &state) {
                let r2 = run_line(&mut g2, &windows, &flog, over, &fin, &winner);
                injected = serde_json::json!({"ending": format!("{:?}", r2.ending), "diffs": r2.diffs.iter().map(|d| serde_json::json!({"window": d.window, "seat": d.seat, "kind": d.kind, "detail": d.detail})).collect::<Vec<_>>()});
                if matches!(r2.ending, Ending::Complete | Ending::GameOver) {
                    history_artifact = true;
                    r.diffs.retain(|d| d.kind == "menu");
                    r.ending = r2.ending;
                }
            }
        }
        let status = match (&r.ending, history_artifact) {
            (_, true) => "history_artifact",
            (Ending::Complete, _) => "complete",
            (Ending::GameOver, _) => "game_over",
            (Ending::Diverged, _) => "diverged",
            (Ending::Unaligned, _) => "unaligned",
            (Ending::ForgeFailed, _) => "forge_play_failed",
        };
        *by_status.entry(status.into()).or_default() += 1;
        total_windows += r.windows;
        total_played += r.played.len();
        menu_diff_windows += r.diffs.iter().filter(|d| d.kind == "menu").count();
        rec["status"] = status.into();
        rec["windows"] = r.windows.into();
        rec["played"] = serde_json::json!(r.played);
        rec["diffs"] = serde_json::json!(r.diffs.iter().map(|d| serde_json::json!({"window": d.window, "seat": d.seat, "kind": d.kind, "detail": d.detail, "stack": windows.get(d.window).map(|w| w.stack), "summary": if d.kind == "menu" { windows.get(d.window).map(|w| w.summary.clone()) } else { None }})).collect::<Vec<_>>());
        rec["injected"] = injected;
        rec["rust_trace"] = serde_json::json!(r.rust_trace);
        rec["forge_notes"] = f["notes"].clone();
        // keep enough to debug a divergence without the whole probe record: the Forge log of the diverging window's segment
        if let Some(d) = r.diffs.iter().find(|d| d.kind != "menu") {
            let w = d.window.min(windows.len().saturating_sub(1));
            let start = windows.get(w).map_or(0, |x| x.log_at);
            let end = windows.get(w + 1).map_or(flog.len(), |x| x.log_at);
            rec["forge_log_at_divergence"] = serde_json::json!(flog[start.min(flog.len())..end.min(flog.len())].to_vec());
            rec["forge_summary_at_divergence"] = serde_json::json!(windows.get(w).map(|x| x.summary.clone()));
        }
        out.push(rec);
    }
    println!("lines {}  windows {}  actions played {}  windows with menu differences {}", out.len(), total_windows, total_played, menu_diff_windows);
    for (k, v) in &by_status {
        println!("  {k:20} {v}");
    }
    std::fs::write(out_path, out.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n") + "\n").unwrap();
}

//! Lockstep line differential, Rust side. The Forge probe plays a *line* (several consecutive priority actions of both seats, chosen by a
//! shared deterministic policy, with the stock AI deciding everything inside effects) and records, at every priority window, the seat,
//! stack size, canonical menu, chosen action, canonical state summary and where its decision log starts. The follower replays the line on the
//! Rust engine: at each window it compares seat, stack, menu and state, plays Forge's chosen action by canonical key, and answers the
//! decisions of that step from the log. The first difference ends the line.

use crate::pos::{summarize, window_keys};
use crate::step::run_step_ex;
use mtg_core::decision::*;
use mtg_view::Game;
use crate::util::ability_text;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Window {
    pub seat: usize,
    pub stack: usize,
    pub keys: Vec<String>,
    pub summary: Vec<String>,
    pub chosen: String,
    pub log_at: usize,
    pub failed: bool,
    /// each player's library, top first (card names)
    pub lib: [Vec<String>; 2],
}

pub fn parse_windows(v: &serde_json::Value) -> Vec<Window> {
    v.as_array()
        .map(|a| {
            a.iter()
                .map(|w| Window {
                    seat: if w["seat"].as_str() == Some("P1") { 0 } else { 1 },
                    stack: w["stack"].as_u64().unwrap_or(0) as usize,
                    keys: w["keys"].as_array().map(|k| k.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
                    summary: w["summary"].as_array().map(|k| k.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
                    chosen: w["chosen"].as_str().unwrap_or("").to_string(),
                    log_at: w["log_at"].as_u64().unwrap_or(0) as usize,
                    failed: w["failed"].as_bool().unwrap_or(false),
                    lib: {
                        let l = |i: usize| w["lib"][i].as_str().map(|t| t.split(';').filter(|x| !x.is_empty()).map(|x| x.to_string()).collect()).unwrap_or_default();
                        [l(0), l(1)]
                    },
                })
                .collect()
        })
        .unwrap_or_default()
}

/// What ended a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ending {
    /// the line ran to its end (a pass with an empty stack, or the window limit) with every window matching
    Complete,
    /// both games ended at the same point
    GameOver,
    /// a state, seat, stack or chosen-action difference at `window` (a real candidate unless the detail says otherwise)
    Diverged,
    /// the follower could not mirror a Forge decision, or the engines tapped different lands: the rest of the line is not comparable
    Unaligned,
    /// Forge could not play its own chosen action
    ForgeFailed,
}

#[derive(Clone, Debug)]
pub struct Diff {
    pub window: usize,
    pub seat: usize,
    pub kind: String,
    pub detail: Vec<String>,
}

pub struct LineResult {
    pub ending: Ending,
    pub windows: usize,
    /// the Forge-chosen action keys played, in order
    pub played: Vec<String>,
    /// differences found (the line ends at the first state or choice difference; menu differences are recorded and the line goes on)
    pub diffs: Vec<Diff>,
    pub rust_trace: Vec<String>,
}

pub fn norm(s: &str) -> BTreeMap<String, String> {
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
    let mut tapped_names: Vec<String> = vec![];
    for t in bf.split(';').filter(|t| !t.is_empty()) {
        let mut t = t.replace("|SummonSick", "");
        if t.contains("^L") && t.contains("|Tapped") {
            tapped_lands += 1;
            tapped_names.push(t.split("^L").next().unwrap_or("").to_string());
            t = t.replace("|Tapped", "");
        }
        if t.contains("^T") {
            let base = t.split("^T").next().unwrap_or("").split(" Token").next().unwrap_or("").trim().to_lowercase();
            t = format!("token:{}{}", base, t.split("^T").nth(1).unwrap_or(""));
        }
        toks.push(t);
    }
    toks.sort();
    tapped_names.sort();
    m.insert("bf".into(), toks.join(";"));
    m.insert("tapped_lands".into(), tapped_lands.to_string());
    m.insert("tapped_names".into(), tapped_names.join(";"));
    let mut hand: Vec<&str> = m["hand"].split(';').filter(|x| !x.is_empty()).collect();
    hand.sort();
    m.insert("hand".into(), hand.join(";"));
    m
}

fn count(s: &str) -> String {
    s.split(';').filter(|x| !x.is_empty()).count().to_string()
}

/// Differences between two summaries. After a shuffle card identities that came from the library cannot be compared (`weak`: zone sizes).
/// The names of the tapped lands are compared separately and returned as `payment`: the two engines may pay a cost from different lands.
fn compare(rs: &[String], fs: &[String], weak: bool, dmg_sources: bool) -> (Vec<String>, bool) {
    let mut diffs = vec![];
    let mut payment = false;
    for (r, fo) in rs.iter().zip(fs.iter()) {
        let (nr, nf) = (norm(r), norm(fo));
        let who = &r[..r.find(' ').unwrap_or(0)];
        for (k, v) in &nr {
            if k == "tapped_names" {
                if nf.get(k) != Some(v) {
                    payment = true;
                }
                continue;
            }
            if dmg_sources && matches!(k.as_str(), "life" | "tapped_lands") {
                if nf.get(k) != Some(v) {
                    payment = true;
                }
                continue;
            }
            let w = weak && matches!(k.as_str(), "hand" | "gy" | "ex" | "bf");
            let (a, b) = (v.clone(), nf.get(k).cloned().unwrap_or_default());
            let (a, b) = if w { (count(&a), count(&b)) } else { (a, b) };
            if a != b {
                diffs.push(format!("{who} {k}{}: rust [{a}] forge [{b}]", if w { " (size)" } else { "" }));
            }
        }
    }
    (diffs, payment)
}

fn dmg_sources(g: &Game) -> bool {
    let st = g.raw_state();
    let db = g.db();
    st.battlefield().iter().any(|&r| matches!(db.def(st.def_of(r)).name.as_str(), "Ancient Tomb" | "City of Traitors"))
}

/// A finished game: both lives and the winner (zone contents after a game ends are not meaningful: Forge clears them).
fn final_diffs(rs: &[String], fs: &[String], res: GameResult, winner: &str) -> Vec<String> {
    let mut d = vec![];
    for (r, f) in rs.iter().zip(fs.iter()) {
        let (a, b) = (norm(r), norm(f));
        if a.get("life") != b.get("life") {
            d.push(format!("{} life: rust [{}] forge [{}]", &r[..r.find(' ').unwrap_or(0)], a.get("life").cloned().unwrap_or_default(), b.get("life").cloned().unwrap_or_default()));
        }
    }
    let rw = match res {
        GameResult::Win(s) => if s.0 == 0 { "P1" } else { "P2" },
        GameResult::Draw => "draw",
    };
    if rw != winner && winner != "?" {
        d.push(format!("winner: rust [{rw}] forge [{winner}]"));
    }
    d
}

pub fn run_line(g: &mut Game, windows: &[Window], log: &[String], over: bool, final_summary: &[String], winner: &str) -> LineResult {
    let mut res = LineResult { ending: Ending::Complete, windows: 0, played: vec![], diffs: vec![], rust_trace: vec![] };
    crate::step::LEGEND.lock().unwrap().counter = 0;
    let mut shuffled = [false; 2];
    // the previous step put cards on the bottom of a library in a random order (Atraxa, Thassa's Oracle): the order below the top differs between the engines
    let mut random_bottom = false;
    for (i, w) in windows.iter().enumerate() {
        res.windows = i + 1;
        match g.advance() {
            Status::GameOver(_) => {
                res.ending = Ending::Diverged;
                res.diffs.push(Diff { window: i, seat: w.seat, kind: "rust_game_over".into(), detail: vec![format!("the engine's game ended before Forge's window {i}")] });
                return res;
            }
            Status::NeedDecision(_) => {}
        }
        for e in g.take_events() {
            if let mtg_core::event::Event::Shuffled { player } = e {
                shuffled[player.0 as usize] = true;
            }
        }
        let p = g.pending().unwrap().clone();
        if !matches!(p.kind, DecisionKind::Priority) {
            res.ending = Ending::Unaligned;
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "no_priority".into(), detail: vec![format!("engine decision {:?} where Forge has a priority window", p.kind)] });
            return res;
        }
        let rstack = g.raw_state().stack().len();
        if p.seat.0 as usize != w.seat || rstack != w.stack {
            res.ending = Ending::Diverged;
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "window".into(), detail: vec![format!("priority: rust seat {} stack {}, forge seat {} stack {}", p.seat.0, rstack, w.seat, w.stack), format!("rust stack (top first): {}", g.raw_state().stack().iter().rev().map(|e| format!("{}{}", g.db().def(e.def).name, if matches!(e.kind, mtg_core::state::StackKind::Spell) { "" } else { " (ability)" })).collect::<Vec<_>>().join(" | "))] });
            return res;
        }
        // state
        let rs = summarize(g);
        let (sd, payment) = compare(&rs, &w.summary, shuffled[0] || shuffled[1], dmg_sources(g));
        if !sd.is_empty() {
            res.ending = Ending::Diverged;
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "state".into(), detail: sd });
            return res;
        }
        if payment {
            res.ending = Ending::Unaligned;
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "payment".into(), detail: vec!["the engines paid a cost from different lands (or a damage source differs): later windows are not comparable".into()] });
            return res;
        }
        // libraries: the same cards; the same top cards unless a shuffle happened (then the engine's order is set to Forge's)
        if let Some(d) = sync_libraries(g, w, &mut shuffled, random_bottom, i) {
            res.ending = Ending::Diverged;
            res.diffs.push(d);
            return res;
        }
        // menu
        let (acts, okeys) = match window_keys(g).map(|(a, o)| {
            // Forge lists a modal double-faced card's land face as `play`; the engine as `playback`
            let f = |k: &str| k.replacen("playback|", "play|", 1);
            (a.iter().map(|k| f(k)).collect::<Vec<_>>(), o.iter().map(|k| k.as_ref().map(|k| f(k))).collect::<Vec<_>>())
        }) {
            Ok(x) => x,
            Err(e) => {
                res.ending = Ending::Unaligned;
                res.diffs.push(Diff { window: i, seat: w.seat, kind: "keys".into(), detail: vec![e] });
                return res;
            }
        };
        let (a, b): (BTreeSet<&String>, BTreeSet<&String>) = (acts.iter().collect(), w.keys.iter().collect());
        if a != b {
            let mut d: Vec<String> = a.difference(&b).map(|k| format!("RUST_ONLY {k}")).collect();
            d.extend(b.difference(&a).map(|k| format!("FORGE_ONLY {k}")));
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "menu".into(), detail: d });
        }
        if w.failed {
            res.ending = Ending::ForgeFailed;
            return res;
        }
        if (0..2).any(|k| shuffled[k] && lib_names(g, k) != w.lib[k]) || (random_bottom && (0..2).any(|k| lib_names(g, k) != w.lib[k])) {
            // a shuffle happened in the previous step: the two libraries are in different orders, so draws from here on cannot be compared
            res.ending = Ending::Unaligned;
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "shuffle".into(), detail: vec!["a shuffle in the previous step: the line ends here (library order differs between the engines)".into()] });
            return res;
        }
        if w.chosen == "END" || (w.chosen == "PASS" && w.stack == 0) {
            res.ending = Ending::Complete;
            return res;
        }
        // choice
        let idx = if w.chosen == "PASS" {
            p.options.iter().position(|o| matches!(o, Opt::Pass))
        } else {
            okeys.iter().position(|k| k.as_deref() == Some(w.chosen.as_str())).or_else(|| {
                // an `act|..|1` key stands for every ability of that card the engine lists under the same key
                None
            })
        };
        // Forge pays with Cavern-style restricted mana on its own; the engine needs that mana tapped first (kd-0010)
        let (idx, okeys, p) = if idx.is_none() && w.chosen.starts_with("cast|") && tap_restricted_mana(g, &w.chosen) {
            let p = g.pending().unwrap().clone();
            let okeys: Vec<Option<String>> = window_keys(g).map(|(_, o)| o.iter().map(|k| k.as_ref().map(|k| k.replacen("playback|", "play|", 1))).collect()).unwrap_or_default();
            (okeys.iter().position(|k| k.as_deref() == Some(w.chosen.as_str())), okeys, p)
        } else {
            (idx, okeys, p)
        };
        let _ = &p;
        let Some(idx) = idx else {
            res.ending = Ending::Diverged;
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "chosen_not_offered".into(), detail: vec![format!("forge chose {} which the engine does not offer", w.chosen)] });
            return res;
        };
        res.played.push(w.chosen.clone());
        let end = windows.get(i + 1).map_or(log.len(), |n| n.log_at).min(log.len());
        let seg = &log[w.log_at.min(end)..end];
        let r = step_reconciled(g, idx, seg, w.chosen != "PASS", windows.get(i + 1));
        res.rust_trace.extend(r.trace.iter().map(|t| format!("w{i}: {t}")));
        for k in 0..2 {
            shuffled[k] = r.shuffled_seats[k];
        }
        random_bottom = r.trace.iter().any(|t| t.contains("RevealPick") || t.contains("OracleTop"));
        if !r.unmatched.is_empty() || !r.leftover.is_empty() {
            res.ending = Ending::Unaligned;
            let mut d: Vec<String> = r.unmatched.iter().map(|u| format!("unmatched: {u}")).collect();
            d.extend(r.leftover.iter().map(|u| format!("leftover: {u}")));
            res.diffs.push(Diff { window: i, seat: w.seat, kind: "follow".into(), detail: d });
            return res;
        }
        if let Some(o) = r.over {
            // the engine's game ended during this step
            let rs = summarize(g);
            if over {
                let sd = final_diffs(&rs, final_summary, o, winner);
                if sd.is_empty() {
                    res.ending = Ending::GameOver;
                } else {
                    res.ending = Ending::Diverged;
                    res.diffs.push(Diff { window: i + 1, seat: w.seat, kind: "final_state".into(), detail: sd });
                }
            } else {
                res.ending = Ending::Diverged;
                res.diffs.push(Diff { window: i + 1, seat: w.seat, kind: "rust_game_over".into(), detail: vec!["the engine's game ended during the step; Forge's continued".into()] });
            }
            return res;
        }
    }
    // Forge's line ended in a game over after its last recorded window
    if over {
        if let Status::GameOver(o) = g.advance() {
            let rs = summarize(g);
            let sd = final_diffs(&rs, final_summary, o, winner);
            if sd.is_empty() {
                res.ending = Ending::GameOver;
            } else {
                res.ending = Ending::Diverged;
                res.diffs.push(Diff { window: windows.len(), seat: 0, kind: "final_state".into(), detail: sd });
            }
        } else {
            res.ending = Ending::Diverged;
            res.diffs.push(Diff { window: windows.len(), seat: 0, kind: "forge_game_over".into(), detail: vec!["Forge's game ended after the last window; the engine's did not".into()] });
        }
    }
    res
}


/// Library order, top first.
fn lib_names(g: &Game, seat: usize) -> Vec<String> {
    let st = g.raw_state();
    let db = g.db();
    st.library(mtg_core::ids::Seat(seat as u8)).iter().rev().map(|&r| crate::util::def_name(db, st, r)).collect()
}

/// Compares both libraries with Forge's. Contents must agree, and the order of the top three cards unless that library was shuffled (the two
/// engines shuffle differently): then only the contents are compared, and the caller ends the line after this window because later draws are not comparable.
fn sync_libraries(g: &mut Game, w: &Window, shuffled: &mut [bool; 2], random_bottom: bool, window: usize) -> Option<Diff> {
    for seat in 0..2 {
        let mine = lib_names(g, seat);
        let theirs = &w.lib[seat];
        if mine == *theirs {
            continue;
        }
        let (mut a, mut b) = (mine.clone(), theirs.clone());
        a.sort();
        b.sort();
        if a != b {
            return Some(Diff { window, seat: w.seat, kind: "state".into(), detail: vec![format!("seat {seat} library contents: rust {} cards, forge {} cards; only in rust {:?}, only in forge {:?}", mine.len(), theirs.len(), mset_minus(&a, &b), mset_minus(&b, &a))] });
        }
        if !shuffled[seat] && !random_bottom {
            let n = 3.min(mine.len());
            if mine[..n] != theirs[..n] {
                return Some(Diff { window, seat: w.seat, kind: "state".into(), detail: vec![format!("seat {seat} library order (no shuffle seen by the engine): rust top {:?}, forge top {:?}", &mine[..n], &theirs[..n])] });
            }
        }
    }
    None
}

fn mset_minus(a: &[String], b: &[String]) -> Vec<String> {
    let mut b = b.to_vec();
    let mut out = vec![];
    for x in a {
        if let Some(i) = b.iter().position(|y| y == x) {
            b.remove(i);
        } else {
            out.push(x.clone());
        }
    }
    out
}


/// Runs one step. If the next Forge window shows a library in an order the engine did not produce (a shuffle or a random bottom order, which the two
/// engines draw differently), the step is replayed with the engine's scripted-random hook set to Forge's resulting order, so later draws agree.
/// A step that no hook candidate reproduces runs unscripted (the caller then ends the line at the next window).
fn step_reconciled(g: &mut Game, idx: usize, seg: &[String], cast: bool, next: Option<&Window>) -> crate::step::StepResult {
    use mtg_core::ids::Seat;
    use mtg_core::state::RandKind;
    let base = crate::step::LEGEND.lock().unwrap().counter;
    // every trial of this step starts from the same legend-decision counter; the chosen trial's counter is kept
    let trial = |c: &mut Game| -> (crate::step::StepResult, usize) {
        crate::step::LEGEND.lock().unwrap().counter = base;
        let r = run_step_ex(c, idx, seg, None, None, true, cast);
        let after = crate::step::LEGEND.lock().unwrap().counter;
        (r, after)
    };
    let plain = {
        let mut c = g.clone();
        let (r, after) = trial(&mut c);
        (c, r, after)
    };
    let Some(next) = next else {
        crate::step::LEGEND.lock().unwrap().counter = plain.2;
        *g = plain.0;
        return plain.1;
    };
    let db = g.db().clone();
    let at_window = matches!(plain.0.pending().map(|p| p.kind), Some(DecisionKind::Priority)) && plain.1.over.is_none();
    if !at_window {
        crate::step::LEGEND.lock().unwrap().counter = plain.2;
        *g = plain.0;
        return plain.1;
    }
    let order_ids = |names: &[String]| -> Option<Vec<mtg_core::ids::CardDefId>> { names.iter().map(|n| db.id(n)).collect() };
    for seat in 0..2usize {
        let theirs = &next.lib[seat];
        let mine = lib_names(&plain.0, seat);
        let (mut a, mut b) = (mine.clone(), theirs.clone());
        a.sort();
        b.sort();
        // a shuffle followed by draws (Ponder): the engine's library holds the drawn cards on top of Forge's remaining library
        let mut drawn_after: Vec<Vec<String>> = vec![];
        let mut pre = lib_names(g, seat);
        pre.sort();
        if mine != *theirs && pre.len() > theirs.len() && pre.len() - theirs.len() <= 3 {
            let mut rest = pre.clone();
            let mut ok = true;
            for x in &b {
                match rest.iter().position(|y| y == x) {
                    Some(i) => {
                        rest.remove(i);
                    }
                    None => ok = false,
                }
            }
            if ok {
                // `rest` are the drawn cards; try every draw order (top first)
                fn perms(v: &mut Vec<String>, k: usize, out: &mut Vec<Vec<String>>) {
                    if k == v.len() {
                        if !out.contains(v) {
                            out.push(v.clone());
                        }
                        return;
                    }
                    for i in k..v.len() {
                        v.swap(k, i);
                        perms(v, k + 1, out);
                        v.swap(k, i);
                    }
                }
                perms(&mut rest, 0, &mut drawn_after);
            }
        }
        if mine == *theirs || (a != b && drawn_after.is_empty()) {
            continue;
        }
        // candidates: the whole library shuffled (a fetch), or the bottom `len` cards put in a random order (Atraxa, Thassa's Oracle)
        let mut cands: Vec<(RandKind, Vec<String>)> = vec![];
        for d in &drawn_after {
            let mut full = d.clone();
            full.extend(theirs.iter().cloned());
            cands.push((RandKind::Shuffle, full));
        }
        if a == b {
            cands.push((RandKind::Shuffle, theirs.clone()));
        }
        let first_diff = mine.iter().zip(theirs.iter()).position(|(x, y)| x != y).unwrap_or(0);
        for len in (theirs.len() - first_diff)..=theirs.len() {
            cands.push((RandKind::BottomOrder, theirs[theirs.len() - len..].to_vec()));
        }
        for (kind, names) in cands {
            let Some(ids) = order_ids(&names) else { continue };
            let mut c = g.clone();
            c.raw_state_mut().scenario_script_random(vec![(Seat(seat as u8), kind, ids)]);
            let (r, after) = trial(&mut c);
            let (viol, left) = c.raw_state().scenario_random_report();
            c.raw_state_mut().scenario_script_random(vec![]);
            if viol == 0 && left == 0 && lib_names(&c, seat) == *theirs {
                crate::step::LEGEND.lock().unwrap().counter = after;
                *g = c;
                return r;
            }
        }
    }
    crate::step::LEGEND.lock().unwrap().counter = plain.2;
    *g = plain.0;
    plain.1
}

/// Taps one restricted-mana source (Cavern of Souls) after which `chosen` is offered; commits it and returns true, or leaves `g` unchanged.
fn tap_restricted_mana(g: &mut Game, chosen: &str) -> bool {
    let Some(p) = g.pending().cloned() else { return false };
    for (i, o) in p.options.iter().enumerate() {
        let Opt::Mana { src, ability, .. } = *o else { continue };
        let def = g.raw_state().def_of(src);
        if !ability_text(g.db(), def, ability).contains("AddManaRestricted") {
            continue;
        }
        let mut c = g.clone();
        if c.apply(p.id, i).is_err() || !matches!(c.advance(), Status::NeedDecision(_)) {
            continue;
        }
        if c.pending().map_or(false, |q| q.seat == p.seat) {
            if let Ok((_, keys)) = window_keys(&c) {
                if keys.iter().flatten().any(|k| k == chosen) {
                    *g = c;
                    return true;
                }
            }
        }
    }
    false
}

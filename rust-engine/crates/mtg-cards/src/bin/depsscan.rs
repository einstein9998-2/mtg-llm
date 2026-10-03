//! `deps-scan` (doc 01 section 10.4): lists pairs of layered continuous effects in the pool that
//! share a layer and whose footprints overlap, so that timestamp-only evaluation (layers stage 1)
//! could be wrong for them. Every flagged pair must appear in `cards/deps-reviewed.txt` (a recorded
//! review: independent, or cut) or the scan fails. Usage: depsscan [deck dir]
//!
//! Footprint model per effect: `writes` = attributes it changes, `reads` = attributes its affected-set
//! filter (and an optional static condition) looks at. Attributes: types, supertypes, subtypes,
//! colors, abilities, pt. Two effects in one layer are flagged when one's writes meet the other's
//! reads (applying one first could change whether, or to what, the other applies: CR 613.8a).
//! Write/write overlap alone is ordered by timestamp (CR 613.7) and is only listed as information.

use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
struct Eff {
    card: String,
    origin: String,
    layer: String,
    kind: String,
    writes: BTreeSet<&'static str>,
    reads: BTreeSet<&'static str>,
}

fn writes_of(kind: &str) -> Vec<&'static str> {
    match kind {
        "ModifyPT" | "SetPT" | "SetPTExpr" => vec!["pt"],
        "GrantKeywords" | "HexproofFrom" => vec!["abilities"],
        "AddTypes" | "RemoveTypes" => vec!["types"],
        "AddSubtypes" | "SetCreatureType" => vec!["subtypes"],
        "SetColors" => vec!["colors"],
        "BecomeBasicLand" => vec!["subtypes", "abilities"],
        "SetTypes" => vec!["types", "subtypes"],
        _ => vec![],
    }
}

fn nonempty(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => false,
        Value::String(s) => !s.is_empty() && s != "(empty)" && s != "Rel::Any" && s != "Any",
        Value::Array(a) => !a.is_empty(),
        Value::Object(m) => !m.is_empty(),
        _ => true,
    }
}

/// Attributes a filter object reads (any JSON object with ObjFilter keys).
fn filter_reads(v: &Value, out: &mut BTreeSet<&'static str>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let attr = match k.as_str() {
                    "types_any" | "types_not" => Some("types"),
                    "supertypes_any" | "supertypes_not" => Some("supertypes"),
                    "subtypes_any" | "subtypes_not" => Some("subtypes"),
                    "colors_any" | "colors_not" | "colorless" => Some("colors"),
                    "keywords_any" => Some("abilities"),
                    "power" | "toughness" => Some("pt"),
                    _ => None,
                };
                if let Some(a) = attr {
                    if nonempty(x) {
                        out.insert(a);
                    }
                }
                filter_reads(x, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| filter_reads(x, out)),
        _ => {}
    }
}

fn effect_kind(e: &Value) -> String {
    match e {
        Value::String(s) => s.clone(),
        Value::Object(m) => m.keys().next().cloned().unwrap_or_default(),
        _ => String::new(),
    }
}

/// Walks an arbitrary JSON tree for `Continuous` effects and `Static` abilities.
fn collect(card: &str, v: &Value, ab: &str, out: &mut Vec<Eff>) {
    match v {
        Value::Object(m) => {
            if let Some(s) = m.get("Static") {
                push_static(card, ab, s, out);
            }
            if let Some(c) = m.get("Continuous") {
                if let Value::Object(cm) = c {
                    let mut e = Eff { card: card.into(), origin: format!("{ab} (resolved effect)"), layer: cm["layer"].as_str().unwrap_or("?").into(), kind: effect_kind(&cm["effect"]), writes: BTreeSet::new(), reads: BTreeSet::new() };
                    e.writes.extend(writes_of(&e.kind));
                    filter_reads(&cm["objs"], &mut e.reads);
                    out.push(e);
                }
            }
            for (k, x) in m {
                if k != "Static" {
                    collect(card, x, ab, out);
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| collect(card, x, ab, out)),
        _ => {}
    }
}

fn push_static(card: &str, ab: &str, s: &Value, out: &mut Vec<Eff>) {
    let kind = effect_kind(&s["effect"]);
    let mut e = Eff { card: card.into(), origin: format!("{ab} (static)"), layer: s["layer"].as_str().unwrap_or("?").into(), kind, writes: BTreeSet::new(), reads: BTreeSet::new() };
    e.writes.extend(writes_of(&e.kind));
    filter_reads(&s["applies_to"], &mut e.reads);
    filter_reads(&s["cond"], &mut e.reads);
    // A characteristic-defining expression reads whatever it counts.
    if e.kind == "SetPTExpr" {
        filter_reads(&s["effect"], &mut e.reads);
    }
    out.push(e);
}

fn main() {
    let db = mtg_cards::legacy::build();
    let dir = std::env::args().nth(1).unwrap_or_else(|| "decks".into());
    // Pool = cards named in the deck lists (main and sideboard) plus tokens/faces they create.
    let mut names: BTreeSet<String> = BTreeSet::new();
    for e in std::fs::read_dir(&dir).expect("deck dir").flatten() {
        if e.path().extension().map_or(false, |x| x == "txt") {
            for l in std::fs::read_to_string(e.path()).unwrap().lines() {
                if let Some((n, name)) = l.split_once(' ') {
                    if n.parse::<u32>().is_ok() {
                        names.insert(name.trim().to_string());
                    }
                }
            }
        }
    }
    let mut effs: Vec<Eff> = Vec::new();
    for def in &db.defs {
        let in_pool = names.contains(&def.name) || def.is_token || def.is_back || def.name.starts_with("Emblem");
        if in_pool {
            for (i, a) in def.abilities.iter().enumerate() {
                let v = serde_json::to_value(a).expect("serialize ability");
                collect(&def.name, &v, &format!("ability {i}"), &mut effs);
            }
        }
    }
    let reviewed: Vec<String> = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/cards/deps-reviewed.txt")).unwrap_or_default().lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(|l| l.to_string()).collect();
    println!("{} layered effects in the pool", effs.len());
    let mut flagged = 0;
    let mut unreviewed = 0;
    let mut timestamp_only = 0;
    for i in 0..effs.len() {
        for j in i..effs.len() {
            let (a, b) = (&effs[i], &effs[j]);
            if a.layer != b.layer {
                continue;
            }
            let dep = a.writes.intersection(&b.reads).next().is_some() || b.writes.intersection(&a.reads).next().is_some();
            if !dep {
                if a.writes.intersection(&b.writes).next().is_some() && i != j {
                    timestamp_only += 1;
                }
                continue;
            }
            flagged += 1;
            let (c1, c2) = if a.card <= b.card { (&a.card, &b.card) } else { (&b.card, &a.card) };
            let key = format!("{c1} | {c2} | {}", a.layer);
            if reviewed.iter().any(|r| r.starts_with(&key)) {
                continue;
            }
            unreviewed += 1;
            println!("UNREVIEWED {key}\n    {} [{}] {} writes {:?} reads {:?}\n    {} [{}] {} writes {:?} reads {:?}", a.card, a.kind, a.origin, a.writes, a.reads, b.card, b.kind, b.origin, b.writes, b.reads);
        }
    }
    println!("{flagged} dependent-looking pairs, {unreviewed} unreviewed; {timestamp_only} write/write pairs ordered by timestamp only");
    if unreviewed > 0 {
        std::process::exit(1);
    }
}

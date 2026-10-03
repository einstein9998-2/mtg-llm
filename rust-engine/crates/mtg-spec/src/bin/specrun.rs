//! usage: specrun SCENARIOS.json [filter-substring ...] [-v]
//! Runs the visible spec scenarios against the Legacy pool and prints a triage summary.

use mtg_spec::{run_scenario, Outcome};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("scenarios json");
    let verbose = args.iter().any(|a| a == "-v");
    let filters: Vec<&String> = args.iter().skip(1).filter(|a| *a != "-v").collect();
    let docs: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let db = mtg_cards::legacy::build();
    let (mut pass, mut fail, mut unsup) = (0, 0, 0);
    let mut fail_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut unsup_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut passed: Vec<String> = Vec::new();
    for d in &docs {
        let id = d["id"].as_str().unwrap_or("?").to_string();
        let file = d["_file"].as_str().unwrap_or("");
        if !filters.is_empty() && !filters.iter().any(|f| id.contains(f.as_str()) || file.contains(f.as_str())) {
            continue;
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_scenario(&db, d)));
        let r = match r {
            Ok(o) => o,
            Err(e) => {
                let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                Outcome::Fail(format!("PANIC {m}"))
            }
        };
        match r {
            Outcome::Pass => {
                pass += 1;
                passed.push(id.clone());
                if verbose {
                    println!("PASS {id}");
                }
            }
            Outcome::Fail(m) => {
                fail += 1;
                if verbose {
                    println!("FAIL {id}: {m}");
                }
                let key: String = m.chars().take(110).collect();
                fail_groups.entry(key).or_default().push(id);
            }
            Outcome::Unsupported(m) => {
                unsup += 1;
                if verbose {
                    println!("UNSUPPORTED {id}: {m}");
                }
                unsup_groups.entry(m).or_default().push(id);
            }
        }
    }
    println!("\n== {pass} pass, {fail} fail, {unsup} unsupported of {} ==", pass + fail + unsup);
    let mut v: Vec<_> = unsup_groups.iter().collect();
    v.sort_by_key(|(_, ids)| std::cmp::Reverse(ids.len()));
    println!("\nUnsupported (by reason):");
    for (k, ids) in v.iter().take(60) {
        println!("  {:4}  {k}   e.g. {}", ids.len(), ids[0]);
    }
    let mut v: Vec<_> = fail_groups.iter().collect();
    v.sort_by_key(|(_, ids)| std::cmp::Reverse(ids.len()));
    println!("\nFailures (by message prefix):");
    for (k, ids) in v.iter().take(60) {
        println!("  {:4}  {k}   e.g. {}", ids.len(), ids[0]);
    }
    if let Ok(p) = std::env::var("SPEC_PASSED_OUT") {
        std::fs::write(p, passed.join("\n")).unwrap();
    }
}

//! Forward-pass check for the Rust/Python parity test: `netcheck <net.bin> <sample.json>` prints
//! {"value": v, "priors": [...]} for the sample {"state": [[i, v]...], "options": [[kind, dec, def, zone, value]...]}.
//! `netcheck --random <out.bin> <state_len> <hidden> <emb> <seed>` writes an untrained net with the
//! real vocabulary sizes (for pipeline tests).
use mtg_agent::Net;
use mtg_view::OptionFeat;
use serde_json::{json, Value};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.get(1).map(String::as_str) == Some("--random") {
        let p = |i: usize| a[i].parse::<usize>().unwrap();
        let n = Net::random(p(3), p(4), p(5), 20, 18, mtg_cards::legacy::build().defs.len() + 1, 9, p(6) as u64);
        std::fs::write(&a[2], n.to_bytes()).unwrap();
        return;
    }
    let net = Net::load(std::path::Path::new(&a[1])).unwrap();
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&a[2]).unwrap()).unwrap();
    let state: Vec<(u32, f32)> = v["state"].as_array().unwrap().iter().map(|x| (x[0].as_u64().unwrap() as u32, x[1].as_f64().unwrap() as f32)).collect();
    let opts: Vec<OptionFeat> = v["options"].as_array().unwrap().iter().map(|x| OptionFeat { kind: x[0].as_u64().unwrap() as u8, decision: x[1].as_u64().unwrap() as u8, subject_def: x[2].as_u64().unwrap() as u16, subject_zone: x[3].as_u64().unwrap() as u8, value: x[4].as_u64().unwrap() as u32 }).collect();
    let (value, priors) = net.forward(&state, &opts);
    println!("{}", json!({"value": value, "priors": priors}));
}

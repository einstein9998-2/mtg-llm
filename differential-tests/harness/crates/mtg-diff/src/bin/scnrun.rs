//! Runs Forge `.scn` scenarios on the Rust engine.
//! Usage: scnrun [-v] [--only substr] <file.scn | dir> ...
use mtg_diff::scn::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut verbose = false;
    let mut only: Option<String> = None;
    let mut paths = vec![];
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-v" => verbose = true,
            "--only" => {
                i += 1;
                only = Some(args[i].clone());
            }
            p => paths.push(p.to_string()),
        }
        i += 1;
    }
    let db = mtg_cards::legacy::build();
    let mut all = vec![];
    for p in &paths {
        let path = std::path::Path::new(p);
        if path.is_dir() {
            let mut fs: Vec<_> = std::fs::read_dir(path).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "scn")).collect();
            fs.sort();
            for f in fs {
                all.extend(parse_file(&f));
            }
        } else {
            all.extend(parse_file(path));
        }
    }
    let (mut pass, mut partial, mut fail, mut err, mut unsup) = (0, 0, 0, 0, 0);
    for sc in &all {
        if let Some(o) = &only {
            if !sc.name.contains(o.as_str()) {
                continue;
            }
        }
        if verbose {
            println!("== {}", sc.name);
        }
        let (out, trace) = run(&db, sc, verbose);
        let tag = match &out {
            Outcome::Pass => {
                pass += 1;
                "PASS"
            }
            Outcome::PassPartial(_) => {
                partial += 1;
                "PARTIAL"
            }
            Outcome::Fail(_) => {
                fail += 1;
                "FAIL"
            }
            Outcome::Error(_) => {
                err += 1;
                "ERROR"
            }
            Outcome::Unsupported(_) => {
                unsup += 1;
                "UNSUP"
            }
        };
        let tag = if sc.xfail.is_some() { format!("{tag}(xf)") } else { tag.to_string() };
        println!("{:<9} {}/{}  {}", tag, sc.file, sc.name, sc.title);
        match &out {
            Outcome::Fail(f) => {
                for x in f {
                    println!("         - {x}");
                }
                if !verbose {
                    for l in &trace {
                        println!("           | {l}");
                    }
                }
            }
            Outcome::Error(e) => {
                println!("         ! {e}");
                if !verbose {
                    for l in &trace {
                        println!("           | {l}");
                    }
                }
            }
            Outcome::PassPartial(n) => {
                for x in n {
                    println!("         ~ {x}");
                }
            }
            Outcome::Unsupported(u) => println!("         ? {u}"),
            _ => {}
        }
    }
    println!("\nTOTAL pass={pass} partial={partial} fail={fail} error={err} unsupported={unsup}");
}

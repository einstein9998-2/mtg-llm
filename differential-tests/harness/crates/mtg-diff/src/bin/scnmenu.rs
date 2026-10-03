//! Prints the canonical action keys the engine offers at the start of each scenario state (no script is run).
//! Usage: scnmenu <file.scn>   (states are completed like the scenario harness does: p1 active, main 1)
use mtg_diff::pos::window_keys;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let db = mtg_cards::legacy::build();
    for sc in mtg_diff::scn::parse_file(std::path::Path::new(&a[0])) {
        match mtg_diff::scn::game_from_state(&db, &sc.state) {
            Ok(g) => match window_keys(&g) {
                Ok((acts, _)) => {
                    println!("== {}", sc.name);
                    for k in acts {
                        println!("   {k}");
                    }
                }
                Err(e) => println!("== {}: {e}", sc.name),
            },
            Err(e) => println!("== {}: {e}", sc.name),
        }
    }
}

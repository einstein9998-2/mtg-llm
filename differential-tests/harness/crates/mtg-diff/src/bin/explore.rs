use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_core::decision::Status;
use mtg_view::*;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pool = mtg_cards::testpool::TestPool { db: mtg_cards::legacy::build() };
    let dir = "/mnt/project-files/decks";
    let (a, _) = mtg_fuzz::load_deck_file(&pool.db, std::path::Path::new(&format!("{dir}/{}.txt", args[1])));
    let (b, _) = mtg_fuzz::load_deck_file(&pool.db, std::path::Path::new(&format!("{dir}/{}.txt", args[2])));
    let seed: u64 = args[3].parse().unwrap();
    let mut g = Game::new(pool.db.clone(), [&a, &b], seed, GameConfig::default());
    let mut rng = mtg_core::rng::Pcg64::from_seed(seed);
    for _ in 0..args.get(4).map(|s| s.parse().unwrap()).unwrap_or(60) {
        match g.advance() {
            Status::GameOver(r) => { println!("over {:?}", r); break; }
            Status::NeedDecision(s) => {
                let d = g.decision_for(s).unwrap();
                let o = g.observe(s);
                println!("--- T{} {:?} seat{} {:?} stack{}", o.turn, o.step, s.0, d.kind, o.stack.len());
                for op in &d.options { println!("    [{}] {:?} {}", op.idx, op.kind, op.label); }
                let idx = rng.below(d.options.len() as u64) as usize;
                g.apply(d.id, d.options[idx].idx as usize).unwrap();
            }
        }
    }
}

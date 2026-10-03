use mtg_cards::testpool;
use mtg_core::decision::*;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_view::*;

fn main() {
    let pool = testpool::build();
    let a = DeckList { main: pool.deck_red_green(), side: vec![] };
    let b = DeckList { main: pool.deck_white_black(), side: vec![] };
    let mut g = Game::new(pool.db.clone(), [&a, &b], 1, GameConfig::default());
    let mut rng = Pcg64::from_seed(99);
    let mut steps = 0;
    loop {
        match g.advance() {
            Status::GameOver(r) => {
                println!("game over {:?} after {} decisions, turn {}", r, steps, g.raw_state().turn_number());
                break;
            }
            Status::NeedDecision(_) => {
                let p = g.pending().unwrap().clone();
                let idx = rng.below(p.options.len() as u64) as usize;
                if steps < 40 {
                    println!("{:?} seat {:?} opts {} -> {:?}", p.kind, p.seat, p.options.len(), p.options[idx]);
                }
                g.apply(p.id, idx).unwrap();
                steps += 1;
                if steps > 100000 {
                    println!("runaway");
                    break;
                }
            }
        }
    }
}

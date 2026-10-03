//! Reviewer probe: Show and Tell fork rate stratified by the opponent's (public) hand size.
use mtg_core::decision::{CardsPurpose, DecisionKind, Opt, Status};
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::{DeckList, Game, UniformConsistentModel};
use std::collections::BTreeMap;
type Db = std::sync::Arc<mtg_core::card::CardDb>;
fn decks() -> (Db, Vec<DeckList>) {
    let db = mtg_cards::legacy::build();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decks");
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let d = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    (db, d)
}
#[test]
fn st_stratified() {
    let (db, d) = decks();
    let games: u64 = std::env::var("ST_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(1500);
    // stratum (opp hand_count, library_count/10) -> [states_none, states_some, fork_some_none, fork_tot_none, fork_some_some, fork_tot_some]
    let mut st: BTreeMap<usize, [u64; 6]> = BTreeMap::new();
    let bf_count = |g: &Game, who: Seat| -> usize { g.observe(who).battlefield.iter().filter(|p| !p.controlled_by_me).count() };
    for seed in 0..games {
        let (a, b) = (&d[0], &d[(seed % 8) as usize]);
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0x55);
        for _ in 0..3000 {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(seat) => {
                    let p = g.pending().unwrap().clone();
                    if matches!(p.kind, DecisionKind::ChooseCards { purpose: CardsPurpose::PutEach, .. }) && !g.observe(seat).active_is_me {
                        let done = p.options.iter().position(|o| matches!(o, Opt::Done)).unwrap();
                        let before = bf_count(&g, seat);
                        let hc = g.observe(seat).opp.hand_count;
                        let mut t = g.clone();
                        t.apply(p.id, done).unwrap();
                        t.advance();
                        let truth = bf_count(&t, seat) > before;
                        let (mut some, mut tot) = (0u64, 0u64);
                        for s in 0..24u64 {
                            let mut f = g.fork(seat, 900 + s, &m).unwrap();
                            let pf = f.pending().unwrap().clone();
                            let done_f = pf.options.iter().position(|o| matches!(o, Opt::Done)).unwrap();
                            let bf0 = bf_count(&f, seat);
                            f.apply(pf.id, done_f).unwrap();
                            f.advance();
                            tot += 1;
                            some += (bf_count(&f, seat) > bf0) as u64;
                        }
                        let e = st.entry(hc).or_insert([0; 6]);
                        if truth { e[1] += 1; e[4] += some; e[5] += tot; } else { e[0] += 1; e[2] += some; e[3] += tot; }
                    }
                    let (id, n) = (p.id, p.options.len());
                    g.apply(id, rng.below(n as u64) as usize).unwrap();
                }
            }
        }
    }
    let (mut tn, mut ts) = (0u64, 0u64);
    for (hc, e) in &st {
        let r0 = if e[3] > 0 { e[2] as f64 / e[3] as f64 } else { f64::NAN };
        let r1 = if e[5] > 0 { e[4] as f64 / e[5] as f64 } else { f64::NAN };
        println!("opp hand {hc}: states none {} some {} | fork rate | none {r0:.3} | some {r1:.3}", e[0], e[1]);
        tn += e[0]; ts += e[1];
    }
    println!("total states none {tn} some {ts}");
}

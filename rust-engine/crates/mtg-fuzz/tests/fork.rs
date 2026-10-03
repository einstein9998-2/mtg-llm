//! Forking and determinization over the real Legacy decks (doc 02 section 5.3, doc 04 section 7).
use mtg_core::decision::Status;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::{BeliefModel, DeckList, Game, UniformConsistentModel};

fn decks() -> (std::sync::Arc<mtg_core::card::CardDb>, Vec<DeckList>) {
    let db = mtg_cards::legacy::build();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decks");
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let d = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    (db, d)
}

fn model(a: &DeckList, b: &DeckList) -> UniformConsistentModel {
    UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] }
}

fn midgame(db: &std::sync::Arc<mtg_core::card::CardDb>, a: &DeckList, b: &DeckList, seed: u64, depth: u64) -> Option<Game> {
    let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
    let mut rng = Pcg64::from_seed(seed ^ 0x99);
    for _ in 0..depth {
        match g.advance() {
            Status::GameOver(_) => return None,
            Status::NeedDecision(_) => {
                let p = g.pending().unwrap();
                let (id, n) = (p.id, p.options.len());
                g.apply(id, rng.below(n as u64) as usize).unwrap();
            }
        }
    }
    g.advance();
    g.pending()?;
    Some(g)
}

/// Everything the observer sees is unchanged by forking, the fork is a legal game that can be
/// played out, and the true state is untouched.
#[test]
fn fork_preserves_the_observers_view_and_is_playable() {
    let (db, d) = decks();
    let mut forks = 0;
    let mut opp_hand_changed = 0;
    for seed in 0..fork_games() {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 30 + (seed * 53) % 400) else { continue };
        let m = model(a, b);
        let obs = g.pending().unwrap().seat;
        assert_eq!(g.fork(obs.other(), 1, &m).err(), Some(mtg_core::fork::ForkError::NotObserversDecision));
        {
            let h0 = g.state_hash();
            let f = g.fork(obs, seed + 1000, &m).unwrap_or_else(|e| panic!("seed {seed} observer {obs:?}: {e:?}"));
            assert_eq!(g.state_hash(), h0, "fork mutated the original");
            let (o1, o2) = (g.observe(obs), f.observe(obs));
            assert_eq!(format!("{o1:?}"), format!("{o2:?}"), "seed {seed}: observer's view changed by forking");
            // The fork satisfies the engine invariants and plays to the end.
            let mut f = f;
            let mut rng = Pcg64::from_seed(seed);
            let mut n = 0;
            loop {
                match f.advance() {
                    Status::GameOver(_) => break,
                    Status::NeedDecision(_) => {
                        mtg_debug::invariants::check(f.raw_state(), f.db()).unwrap_or_else(|e| panic!("seed {seed}: fork violates {} {}", e.id, e.msg));
                        let p = f.pending().unwrap();
                        let (id, len) = (p.id, p.options.len());
                        f.apply(id, rng.below(len as u64) as usize).unwrap();
                        n += 1;
                        if n > 20000 {
                            break;
                        }
                    }
                }
            }
            forks += 1;
            // Aggregate sanity: the sampled opponent hand usually differs from the true one.
            let f2 = g.fork(obs, seed + 1000, &m).unwrap();
            let opp = obs.other();
            let names = |g: &Game| {
                let s = g.raw_state();
                let mut v: Vec<u16> = s.hand(opp).iter().map(|&r| s.def_of(r).0).collect();
                v.sort();
                v
            };
            if names(&g) != names(&f2) {
                opp_hand_changed += 1;
            }
        }
    }
    assert!(forks > 120, "too few forks: {forks}");
    assert!(opp_hand_changed * 2 > forks, "forks rarely differ from the truth ({opp_hand_changed}/{forks}): sampling is degenerate");
}

/// Non-interference in its structural form (doc 02 section 5.3): two true states that agree on
/// everything the observer may know give identical forks for the same seed and model.
#[test]
fn forks_of_indistinguishable_states_are_identical() {
    let (db, d) = decks();
    let mut compared = 0;
    for seed in 0..fork_games() {
        let (a, b) = (&d[((seed + 1) % 8) as usize], &d[((seed + 5) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 20 + (seed * 71) % 500) else { continue };
        let m = model(a, b);
        {
            let obs = g.pending().unwrap().seat;
            let mut g2 = g.clone();
            g2.raw_state_mut().rerandomize_hidden(obs, seed ^ 0xABCD, true);
            let (f1, f2) = (g.fork(obs, 7, &m).unwrap(), g2.fork(obs, 7, &m).unwrap());
            assert_eq!(f1.state_hash(), f2.state_hash(), "seed {seed} observer {obs:?}: fork depends on hidden state");
            compared += 1;
        }
    }
    assert!(compared > 120);
}

/// A different seed gives a different world; the same seed the same one.
#[test]
fn fork_seeds_matter() {
    let (db, d) = decks();
    let (a, b) = (&d[0], &d[1]);
    let g = midgame(&db, a, b, 5, 120).expect("game alive");
    let m = model(a, b);
    let obs = g.pending().unwrap().seat;
    let h = |s| g.fork(obs, s, &m).unwrap().state_hash();
    assert_eq!(h(1), h(1));
    assert_ne!(h(1), h(2));
}

#[test]
fn inconsistent_decklist_is_refused() {
    let (db, d) = decks();
    let (a, b) = (&d[0], &d[1]);
    let g = midgame(&db, a, b, 9, 150).expect("game alive");
    let wrong = UniformConsistentModel { decks: [a.main.clone(), a.main.clone()] };
    assert!(g.fork(g.pending().unwrap().seat, 1, &wrong).is_err());
    let _ = std::any::type_name::<dyn BeliefModel>();
}

/// Number of games per fork property (env `FORK_GAMES` for long runs).
fn fork_games() -> u64 {
    std::env::var("FORK_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(160)
}

/// Forks taken at (nearly) every decision of whole games stay sound: the observer's view is
/// unchanged, the fork's own invariants hold and it plays on for a while. Show and Tell's second
/// chooser is forced into the sample (the first chooser's secret pick is state the fork must not
/// keep as is).
#[test]
fn forks_at_many_decision_points_are_sound() {
    let (db, d) = decks();
    let games = fork_games() / 4 + 8;
    let mut at_secret_choice = 0;
    let mut total = 0;
    for seed in 0..games {
        // Alurentell (index 0) casts Show and Tell; pair it with each deck in turn.
        let (a, b) = (&d[0], &d[(seed % 8) as usize]);
        let m = model(a, b);
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0x55);
        let mut n = 0u64;
        loop {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(_) => {
                    let p = g.pending().unwrap();
                    let secret = format!("{:?}", p.kind).contains("PutEach");
                    if secret || n % 11 == 0 {
                        let obs = p.seat;
                        let f = g.fork(obs, seed * 31 + n, &m).unwrap_or_else(|e| panic!("seed {seed} step {n}: {e:?}"));
                        assert_eq!(format!("{:?}", g.observe(obs)), format!("{:?}", f.observe(obs)), "seed {seed} step {n}: view changed");
                        let mut f = f;
                        let mut r2 = Pcg64::from_seed(n);
                        for _ in 0..60 {
                            match f.advance() {
                                Status::GameOver(_) => break,
                                Status::NeedDecision(_) => {
                                    mtg_debug::invariants::check(f.raw_state(), f.db()).unwrap_or_else(|e| panic!("seed {seed} step {n}: fork violates {} {}", e.id, e.msg));
                                    let q = f.pending().unwrap();
                                    let (id, len) = (q.id, q.options.len());
                                    f.apply(id, r2.below(len as u64) as usize).unwrap();
                                }
                            }
                        }
                        total += 1;
                        if secret {
                            at_secret_choice += 1;
                        }
                    }
                    let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                    n += 1;
                    if n > 4000 {
                        break;
                    }
                }
            }
        }
    }
    eprintln!("{total} forks, {at_secret_choice} at Show and Tell choices");
    assert!(total > 200);
}

/// The encoding has a fixed shape, is deterministic, is unchanged by forking (it only sees what
/// the observer sees) and gives one row per option.
#[test]
fn encoding_is_fixed_shape_and_view_only() {
    let (db, d) = decks();
    let n_defs = db.defs.len();
    let mut nonzero_options = 0;
    for seed in 0..40u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 2) % 8) as usize]);
        for depth in [60u64, 130, 200, 280, 350] {
        let Some(g) = midgame(&db, a, b, seed, depth + seed) else { continue };
        let obs = g.pending().unwrap().seat;
        let o = g.observe(obs);
        let (mut v1, mut v2) = (Vec::new(), Vec::new());
        mtg_view::encode_state(&o, n_defs, &mut v1);
        assert_eq!(v1.len(), mtg_view::state_len(n_defs));
        assert!(v1.iter().all(|x| x.is_finite()));
        let f = g.fork(obs, seed, &model(a, b)).unwrap();
        mtg_view::encode_state(&f.observe(obs), n_defs, &mut v2);
        assert_eq!(v1, v2, "encoding differs between the game and its fork");
        let opts = mtg_view::encode_options(&o);
        assert_eq!(opts.len(), g.pending().unwrap().options.len());
        assert_eq!(opts, mtg_view::encode_options(&f.observe(obs)));
        nonzero_options += opts.iter().filter(|o| o.subject_def != 0).count();
        }
    }
    eprintln!("options with a card subject: {nonzero_options}");
    assert!(nonzero_options > 20, "options never name a card");
}

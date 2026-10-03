//! Search, evaluator and environment checks over the real eight-deck pool.
use mtg_agent::*;
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_view::*;

fn setup() -> (std::sync::Arc<mtg_core::card::CardDb>, Vec<DeckList>) {
    let db = mtg_cards::legacy::build();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../decks");
    let (_, decks) = load_deck_dir(&db, dir);
    (db, decks)
}

/// Plays random moves until the first decision of `seat` with at least `min_opts` options after `skip` of them.
fn to_decision(g: &mut Game, skip: usize, min_opts: usize) -> Option<Seat> {
    let mut rnd = RandomPolicy::new(5);
    let mut seen = 0;
    for _ in 0..4000 {
        match g.advance() {
            Status::GameOver(_) => return None,
            Status::NeedDecision(seat) => {
                let d = g.seat_view(seat).decision().unwrap();
                if d.options.len() >= min_opts {
                    seen += 1;
                    if seen > skip {
                        return Some(seat);
                    }
                }
                let i = rnd.choose(&g.seat_view(seat), d.options.len());
                g.apply(d.id, i).unwrap();
            }
        }
    }
    None
}

#[test]
fn search_is_deterministic_and_accounts_for_every_iteration() {
    let (db, d) = setup();
    for seed in 0..6u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let Some(seat) = to_decision(&mut g, 20 + seed as usize * 7, 3) else { continue };
        let model = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let cfg = SearchConfig { iterations: 24, seed: 9, ..SearchConfig::default() };
        let run = || search(&g.seat_view(seat), &model, &mut RolloutEvaluator::new(800, 3), &cfg);
        let (r1, r2) = (run(), run());
        assert_eq!(r1.visits, r2.visits, "seed {seed}: same search twice must agree");
        assert_eq!(r1.fork_errors, 0);
        assert_eq!(r1.visits.iter().sum::<u32>(), r1.iterations);
        assert!(r1.iterations >= 20, "iterations lost: {}", r1.iterations);
        assert!(r1.best() < r1.visits.len());
        let p = r1.policy(1.0);
        assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-4);
    }
}

/// Searching never changes the real game.
#[test]
fn search_does_not_touch_the_real_game() {
    let (db, d) = setup();
    let (a, b) = (&d[0], &d[4]);
    let mut g = Game::new(db.clone(), [a, b], 3, GameConfig::default());
    let seat = to_decision(&mut g, 30, 3).expect("a decision");
    let model = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
    let before = format!("{:?}", g.seat_view(seat).observe());
    let _ = search(&g.seat_view(seat), &model, &mut RolloutEvaluator::new(500, 1), &SearchConfig { iterations: 8, ..SearchConfig::default() });
    assert_eq!(before, format!("{:?}", g.seat_view(seat).observe()));
}

#[test]
fn net_round_trips_and_gives_a_distribution() {
    let (db, _) = setup();
    let net = Net::random(state_len(db.defs.len()), 16, 8, 20, 18, db.defs.len() + 1, 9, 5);
    let back = Net::from_bytes(&net.to_bytes()).unwrap();
    let opts = vec![OptionFeat { kind: 1, decision: 0, subject_def: 5, subject_zone: 1, value: 0 }, OptionFeat { kind: 0, decision: 0, subject_def: 0, subject_zone: 0, value: 3 }];
    let state = vec![(3u32, 1.0f32), (900, 2.0)];
    let (v1, p1) = net.forward(&state, &opts);
    let (v2, p2) = back.forward(&state, &opts);
    assert_eq!((v1, &p1), (v2, &p2));
    assert!(v1.abs() <= 1.0 && (p1.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    assert!(Net::from_bytes(&net.to_bytes()[..100]).is_err());
}

#[test]
fn net_guided_search_runs_through_a_game() {
    let (db, d) = setup();
    let net = std::sync::Arc::new(Net::random(state_len(db.defs.len()), 16, 8, 20, 18, db.defs.len() + 1, 9, 5));
    let (a, b) = (&d[1], &d[6]);
    let model = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
    let mut g = Game::new(db.clone(), [a, b], 11, GameConfig::default());
    let mut pol = MctsPolicy::new(&model, Box::new(NetEvaluator::new(net, db.defs.len())), SearchConfig { iterations: 6, ..SearchConfig::default() });
    for _ in 0..200 {
        match g.advance() {
            Status::GameOver(_) => break,
            Status::NeedDecision(seat) => {
                let sv = g.seat_view(seat);
                let d = sv.decision().unwrap();
                let i = pol.choose(&sv, d.options.len());
                g.apply(d.id, i).unwrap();
            }
        }
    }
}

#[test]
fn env_is_deterministic_validates_answers_and_resets() {
    let (db, d) = setup();
    let cfg = EnvConfig { n_games: 6, seed0: 40, ..EnvConfig::default() };
    let mut e1 = BatchEnv::new(db.clone(), d.clone(), cfg.clone());
    let mut e2 = BatchEnv::new(db.clone(), d.clone(), cfg);
    let (mut o1, mut o2) = (e1.start(), e2.start());
    let mut done = 0;
    for step in 0..300 {
        let key = |o: &[Out]| o.iter().map(|x| match x {
            Out::Obs(b) => format!("o{}:{}:{}:{:?}:{:?}", b.game, b.seat.idx(), b.id.0, b.state, b.options),
            Out::Done(d) => format!("d{}:{}", d.game, d.result),
        }).collect::<Vec<_>>();
        assert_eq!(key(&o1), key(&o2), "step {step}");
        let acts: Vec<(usize, usize)> = o1.iter().filter_map(|x| if let Out::Obs(b) = x { Some((b.game, (step + b.game * 7) % b.options.len())) } else { None }).collect();
        done += o1.iter().filter(|x| matches!(x, Out::Done(_))).count();
        // every waiting game appears exactly once
        let mut w = e1.waiting();
        w.sort();
        let mut g: Vec<usize> = acts.iter().map(|a| a.0).collect();
        g.sort();
        assert!(g.iter().all(|x| w.contains(x)));
        o1 = e1.step(&acts).unwrap();
        o2 = e2.step(&acts).unwrap();
    }
    assert!(done > 0, "some game should have finished in 300 steps");
    // invalid answers are refused and change nothing
    let w = e1.waiting();
    assert!(e1.step(&[(w[0], 100_000)]).is_err());
    assert!(e1.step(&[(100_000, 0)]).is_err());
    assert!(e1.step(&[(w[0], 0), (w[0], 0)]).is_err());
    assert_eq!(e1.waiting(), w);
}

/// The agent crate must build against `mtg-view` without the harness feature, so it can only hold
/// seat-bound views. Built alone (`-p mtg-agent`), so workspace feature unification cannot hide a leak.
#[test]
fn agent_crate_builds_without_the_harness_feature() {
    let ws = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| ws.join("target").display().to_string());
    let out = std::process::Command::new(env!("CARGO"))
        .args(["check", "--offline", "-p", "mtg-agent", "--lib", "--bins", "--manifest-path"])
        .arg(ws.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", format!("{target}/agent-only-check"))
        .output()
        .unwrap();
    assert!(out.status.success(), "mtg-agent does not build without diff-harness:\n{}", String::from_utf8_lossy(&out.stderr));
}

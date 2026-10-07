//! Hidden-information regression tests over the real eight-deck pool. Each one is a decoder an
//! agent could use to recover the opponent's hidden state; the independent review (2026-10-01)
//! showed all of them succeeding before the fixes.
use mtg_core::decision::{CardsPurpose, DecisionKind, Opt, Status};
use mtg_core::fork::HiddenRequest;
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::{BeliefModel, DeckList, Game, UniformConsistentModel};
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

fn midgame(db: &Db, a: &DeckList, b: &DeckList, seed: u64, depth: u64) -> Option<Game> {
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

fn multiset(v: &[CardDefId]) -> BTreeMap<u16, i32> {
    let mut m = BTreeMap::new();
    for d in v {
        *m.entry(d.0).or_insert(0) += 1;
    }
    m
}

/// Forks must not let the agent recover which opponent hand card is which: decode the hand by
/// replaying the sampler and scoring each unknown slot. Chance level is the baseline; the old
/// engine recovered 51/51 hands exactly.
#[test]
fn fork_does_not_decode_opponent_hand() {
    let (db, d) = decks();
    let (mut exact, mut cards_ok, mut cards_tot, mut base_ok, mut states) = (0, 0, 0, 0.0f64, 0);
    for seed in 0..60u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 40 + (seed * 37) % 200) else { continue };
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let obs = g.pending().unwrap().seat;
        let opp = obs.other();
        let deck_opp = if opp == Seat::P0 { &a.main } else { &b.main };
        let st = g.raw_state();
        let req: HiddenRequest = st.hidden_request(&db, obs);
        let truth: Vec<CardDefId> = st.hand(opp).iter().map(|&r| st.def_of(r)).collect();
        if truth.is_empty() || !g.observe(obs).opp.revealed_hand.is_empty() {
            continue;
        }
        let h = truth.len();
        let mut unk: Vec<CardDefId> = deck_opp.clone();
        for k in &req.known[opp.idx()] {
            if let Some(i) = unk.iter().position(|x| x == k) {
                unk.remove(i);
            }
        }
        assert_eq!(unk.len(), req.unknown[opp.idx()]);
        let mut score = vec![0u32; unk.len()];
        for s in 0..48u64 {
            let seed_f = 1000 + s;
            let f = g.fork(obs, seed_f, &m).unwrap();
            let hand_f: Vec<CardDefId> = f.observe(opp).me.hand.iter().map(|c| c.def).collect();
            let hf = multiset(&hand_f);
            let mut rng = Pcg64::from_seed(seed_f ^ 0x3C6E_F372_FE94_F82B);
            let asg = m.sample(&req, &mut rng).unwrap();
            for (r, dd) in asg.defs[opp.idx()].iter().enumerate() {
                if hf.get(&dd.0).copied().unwrap_or(0) > 0 {
                    score[r] += 1;
                }
            }
        }
        let mut idx: Vec<usize> = (0..unk.len()).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(score[i]));
        let pred: Vec<CardDefId> = idx[..h].iter().map(|&i| unk[i]).collect();
        let (tm, pm) = (multiset(&truth), multiset(&pred));
        cards_ok += tm.iter().map(|(k, v)| (*v).min(*pm.get(k).unwrap_or(&0))).sum::<i32>();
        cards_tot += h as i32;
        let um = multiset(&unk);
        let tot = unk.len() as f64;
        base_ok += tm.iter().map(|(k, v)| (*v as f64).min(*um.get(k).unwrap_or(&0) as f64 * h as f64 / tot)).sum::<f64>();
        if tm == pm {
            exact += 1;
        }
        states += 1;
    }
    eprintln!("states {states}: exact {exact}, cards recovered {cards_ok}/{cards_tot}, chance baseline {base_ok:.1}");
    assert!(states > 20);
    // Statistical bound: at most chance plus a generous margin (the reviewer's old decoder got everything).
    assert!((cards_ok as f64) <= base_ok * 1.6 + 10.0, "hand decoder beats chance: {cards_ok} vs {base_ok:.1}");
    assert!(exact <= states / 4, "hand decoder recovered {exact}/{states} hands exactly");
}

/// Library order must not be recoverable: each fork shuffles the positions the observer does not
/// know, so replaying the position permutation reveals nothing about the true order.
#[test]
fn fork_does_not_decode_library_order() {
    let (db, d) = decks();
    let (mut pos_ok, mut pos_tot, mut states) = (0usize, 0usize, 0);
    for seed in 0..60u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 30 + (seed * 29) % 120) else { continue };
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let obs = g.pending().unwrap().seat;
        let o = g.observe(obs);
        if !o.me.library_known_top.is_empty() || !o.me.library_known_bottom.is_empty() || !o.opp.library_known_top.is_empty() || !o.opp.library_known_bottom.is_empty() {
            continue;
        }
        let st = g.raw_state();
        let req = st.hidden_request(&db, obs);
        let lens = [st.library(Seat::P0).len(), st.library(Seat::P1).len()];
        let target = obs.other();
        let deck_t = if target == Seat::P0 { &a.main } else { &b.main };
        let mut unk: Vec<CardDefId> = deck_t.clone();
        for k in &req.known[target.idx()] {
            if let Some(i) = unk.iter().position(|x| x == k) {
                unk.remove(i);
            }
        }
        let n = lens[target.idx()];
        let mut cand: Vec<Vec<bool>> = vec![vec![true; unk.len()]; n];
        for s in 0..64u64 {
            let seed_f = 5000 + s;
            let f = g.fork(obs, seed_f, &m).unwrap();
            let lib_f: Vec<CardDefId> = f.raw_state().library(target).iter().map(|&r| f.raw_state().def_of(r)).collect();
            // the old decoder's model of the fork's position shuffle
            let mut rng = Pcg64::from_seed(seed_f ^ 0x6A09_E667_F3BC_C908);
            let mut perms: Vec<Vec<usize>> = Vec::new();
            for p in 0..2 {
                let mut idx: Vec<usize> = (0..lens[p]).collect();
                rng.shuffle(&mut idx);
                perms.push(idx);
            }
            let perm = &perms[target.idx()];
            let mut rng2 = Pcg64::from_seed(seed_f ^ 0x3C6E_F372_FE94_F82B);
            let asg = m.sample(&req, &mut rng2).unwrap();
            for j in 0..n {
                let i = perm[j];
                for r in 0..unk.len() {
                    if asg.defs[target.idx()][r] != lib_f[j] {
                        cand[i][r] = false;
                    }
                }
            }
        }
        let tl = st.library(target);
        for i in 0..n {
            let defs: std::collections::BTreeSet<u16> = (0..unk.len()).filter(|&r| cand[i][r]).map(|r| unk[r].0).collect();
            pos_tot += 1;
            if defs.len() == 1 && *defs.iter().next().unwrap() == st.def_of(tl[i]).0 {
                pos_ok += 1;
            }
        }
        states += 1;
    }
    eprintln!("states {states}: library positions recovered {pos_ok}/{pos_tot}");
    assert!(states > 10);
    assert!(pos_ok * 20 <= pos_tot, "library order decoder recovered {pos_ok}/{pos_tot}");
}

/// Two worlds the observer cannot tell apart (they differ only in the hidden order of the
/// opponent's library) must give the observer identical forks for the same seed.
#[test]
fn indistinguishable_twins_fork_identically() {
    let (db, d) = decks();
    let (mut n, mut hand_differ, mut lib_differ) = (0, 0, 0);
    for seed in 0..120u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 40 + (seed * 37) % 300) else { continue };
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let obs = g.pending().unwrap().seat;
        let mut g2 = g.clone();
        g2.raw_state_mut().shuffle_unknown_library_order(obs, obs.other(), seed ^ 0x1234);
        assert_eq!(format!("{:?}", g.observe(obs)), format!("{:?}", g2.observe(obs)), "twin is distinguishable");
        let (f1, f2) = (g.fork(obs, 7, &m).unwrap(), g2.fork(obs, 7, &m).unwrap());
        n += 1;
        let defs = |f: &Game, z: bool| -> Vec<u16> {
            let s = f.raw_state();
            let v = if z { s.library(obs.other()) } else { s.hand(obs.other()) };
            v.iter().map(|&r| s.def_of(r).0).collect()
        };
        if defs(&f1, true) != defs(&f2, true) {
            lib_differ += 1;
        }
        if defs(&f1, false) != defs(&f2, false) {
            hand_differ += 1;
        }
    }
    assert!(n > 30);
    assert_eq!((hand_differ, lib_differ), (0, 0), "{n} twins: forks differ (hand {hand_differ}, library {lib_differ})");
}

/// The fork's world does not carry the opponent's private knowledge of its own library.
#[test]
fn fork_forgets_opponent_private_knowledge() {
    let (db, d) = decks();
    let (mut with_knowledge, mut kept) = (0, 0);
    for seed in 0..300u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 60 + (seed * 37) % 400) else { continue };
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let obs = g.pending().unwrap().seat;
        let opp = obs.other();
        let st = g.raw_state();
        let (t, bt) = st.known_library(opp, opp);
        let (to, bo) = st.known_library(opp, obs);
        if t.len() + bt.len() > to.len() + bo.len() {
            with_knowledge += 1;
            let f = g.fork(obs, 3, &m).unwrap();
            let fo = f.observe(opp);
            if fo.me.library_known_top.len() + fo.me.library_known_bottom.len() > to.len() + bo.len() {
                kept += 1;
            }
        }
    }
    eprintln!("opponent-private library knowledge in {with_knowledge} states, kept by the fork in {kept}");
    assert_eq!(kept, 0);
}

/// The observer's next decision id must not depend on the opponent's hidden scry choice.
#[test]
fn decision_id_independent_of_opponent_scry() {
    let (db, d) = decks();
    let (mut found, mut differ) = (0, 0);
    for seed in 0..400u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0x77);
        for _ in 0..1500 {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(seat) => {
                    let p = g.pending().unwrap().clone();
                    if matches!(p.kind, DecisionKind::ChooseCards { purpose: CardsPurpose::ScryBottom, .. }) && p.options.len() >= 3 {
                        let obs = seat.other();
                        let run = |bottom_first: bool| -> Option<u32> {
                            let mut w = g.clone();
                            let mut first = true;
                            for _ in 0..40 {
                                match w.advance() {
                                    Status::GameOver(_) => return None,
                                    Status::NeedDecision(s) => {
                                        let q = w.pending().unwrap().clone();
                                        if s == obs {
                                            return Some(w.observe(obs).decision.as_ref().unwrap().id.0);
                                        }
                                        let idx = if first && bottom_first { 0 } else { q.options.iter().position(|x| matches!(x, Opt::Done)).unwrap_or(0) };
                                        first = false;
                                        w.apply(q.id, idx).unwrap();
                                    }
                                }
                            }
                            None
                        };
                        if let (Some(x), Some(y)) = (run(false), run(true)) {
                            found += 1;
                            if x != y {
                                differ += 1;
                            }
                        }
                    }
                    let (id, n) = (p.id, p.options.len());
                    g.apply(id, rng.below(n as u64) as usize).unwrap();
                }
            }
        }
        if found >= 20 {
            break;
        }
    }
    eprintln!("opponent scries examined {found}, observer decision id differed in {differ}");
    assert!(found >= 5, "too few scry states reached ({found})");
    assert_eq!(differ, 0);
}

/// Show and Tell: the opponent's secret pick must not steer the fork. The fork's opponent pick is
/// resampled, so P(enters something) is the same whatever the truth was.
#[test]
fn show_and_tell_fork_independent_of_true_pick() {
    let (db, d) = decks();
    let (mut n_none, mut n_some) = (0u32, 0u32);
    let (mut fs_none, mut ft_none, mut fs_some, mut ft_some) = (0u32, 0u32, 0u32, 0u32);
    // Per opponent hand size (what the observer can see): (states, forks that enter something,
    // forks) for truth-nothing and truth-something. Hand size drives both the truth and the
    // fork rate, so the unstratified rates differ without any leak.
    let mut strata: std::collections::BTreeMap<usize, [(u32, u32, u32); 2]> = std::collections::BTreeMap::new();
    let bf_count = |g: &Game, who: Seat| -> usize { g.observe(who).battlefield.iter().filter(|p| !p.controlled_by_me).count() };
    for seed in 0..500u64 {
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
                        let mut t = g.clone();
                        t.apply(p.id, done).unwrap();
                        t.advance();
                        let truth_entered = bf_count(&t, seat) > before;
                        let (mut some, mut tot) = (0u32, 0u32);
                        for s in 0..24u64 {
                            let mut f = g.fork(seat, 900 + s, &m).unwrap();
                            let pf = f.pending().unwrap().clone();
                            let done_f = pf.options.iter().position(|o| matches!(o, Opt::Done)).unwrap();
                            let bf0 = bf_count(&f, seat);
                            f.apply(pf.id, done_f).unwrap();
                            f.advance();
                            tot += 1;
                            some += (bf_count(&f, seat) > bf0) as u32;
                        }
                        let hand = g.observe(seat).opp.hand_count as usize;
                        let cell = &mut strata.entry(hand).or_insert([(0, 0, 0); 2])[truth_entered as usize];
                        *cell = (cell.0 + 1, cell.1 + some, cell.2 + tot);
                        if truth_entered {
                            n_some += 1;
                            fs_some += some;
                            ft_some += tot;
                        } else {
                            n_none += 1;
                            fs_none += some;
                            ft_none += tot;
                        }
                    }
                    let (id, n) = (p.id, p.options.len());
                    g.apply(id, rng.below(n as u64) as usize).unwrap();
                }
            }
        }
    }
    eprintln!("S&T second-chooser states: truth nothing {n_none}, something {n_some}; fork enters-something rate | nothing {fs_none}/{ft_none}, | something {fs_some}/{ft_some}");
    assert!(n_none >= 5 && n_some >= 5, "need both truths represented ({n_none}/{n_some})");
    let (r0, r1) = (fs_none as f64 / ft_none as f64, fs_some as f64 / ft_some as f64);
    eprintln!("unstratified rates {r0:.2} vs {r1:.2} (differ because hand size drives both)");
    // Compare within each hand size, weighting each stratum by its smaller group.
    let (mut num, mut den) = (0f64, 0f64);
    for (hand, c) in &strata {
        let (a, b) = (c[0], c[1]);
        if a.0 >= 3 && b.0 >= 3 {
            let w = a.0.min(b.0) as f64;
            let d = (a.1 as f64 / a.2 as f64 - b.1 as f64 / b.2 as f64).abs();
            eprintln!("  opponent hand {hand}: nothing {}/{} states, something {}/{} states, rate diff {d:.2}", a.1, a.2, b.1, b.2);
            num += w * d;
            den += w;
        }
    }
    assert!(den > 0.0, "no hand size has both truths represented");
    assert!(num / den < 0.15, "fork pick depends on the truth within hand-size strata: {:.2}", num / den);
}

/// Thassa's Oracle puts the rest on the bottom in random order: the owner must not be left with a
/// "known" bottom run it cannot know.
#[test]
fn oracle_does_not_leave_known_bottom() {
    let (db, d) = decks();
    let (mut resolutions, mut known) = (0, 0);
    for seed in 0..800u64 {
        let a = &d[4];
        let b = &d[(seed % 8) as usize];
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0x55);
        let mut last_oracle: Option<(Seat, usize)> = None;
        for _ in 0..2500 {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(seat) => {
                    if let Some((s, before)) = last_oracle.take() {
                        let o = g.observe(s);
                        resolutions += 1;
                        if before == 0 && !o.me.library_known_bottom.is_empty() {
                            known += 1;
                        }
                    }
                    let p = g.pending().unwrap().clone();
                    if matches!(p.kind, DecisionKind::ChooseCards { purpose: CardsPurpose::OracleTop, .. }) {
                        last_oracle = Some((seat, g.observe(seat).me.library_known_bottom.len()));
                    }
                    let (id, n) = (p.id, p.options.len());
                    g.apply(id, rng.below(n as u64) as usize).unwrap();
                }
            }
        }
    }
    eprintln!("Oracle resolutions {resolutions}, bottom run wrongly known {known}");
    assert!(resolutions > 0);
    assert_eq!(known, 0);
}

/// The project's non-interference harness on the real pool (150 games x 2 observers x own-library
/// on/off in release; fewer under debug unless NI_GAMES is set).
#[test]
fn noninterference_on_real_pool() {
    use mtg_debug::noninterference::*;
    let (db, d) = decks();
    let default = if cfg!(debug_assertions) { 40 } else { 150 };
    let games: u64 = std::env::var("NI_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(default);
    let (mut compared, mut differing, mut pairs) = (0u64, 0u64, 0u64);
    let mut fails = Vec::new();
    let mut stops: BTreeMap<&'static str, u32> = BTreeMap::new();
    // NI_ONLY=seed,observer,own isolates one pair.
    let only: Option<(u64, u8, bool)> = std::env::var("NI_ONLY").ok().map(|v| {
        let p: Vec<&str> = v.split(',').collect();
        (p[0].parse().unwrap(), p[1].parse().unwrap(), p[2] == "1")
    });
    for own in [false, true] {
        for seed in 0..games {
            let (a, b) = (&d[(seed % 8) as usize], &d[((seed / 8 + seed + 3) % 8) as usize]);
            for observer in [Seat::P0, Seat::P1] {
                if let Some((s, o, w)) = only {
                    if (s, o, w) != (seed, observer.0, own) {
                        continue;
                    }
                }
                let cfg = NiConfig { observer, own_library: own, depth: 20 + (seed * 37) % 300, max_steps: 600 };
                pairs += 1;
                differing += hidden_differs(&db, [a, b], seed, cfg) as u64;
                match run_pair_ex(&db, [a, b], seed, cfg) {
                    Ok((n, why)) => {
                        compared += n;
                        *stops.entry(why).or_insert(0u32) += 1;
                    }
                    Err(f) => fails.push(format!("own_library={own} observer={} seed {} step {}: {}", observer.0, f.seed, f.step, f.what)),
                }
            }
        }
    }
    eprintln!("real pool NI: {pairs} pairs, {compared} steps compared, hidden state differed in {differing} pairs, failures {}", fails.len());
    assert!(differing * 2 > pairs, "the twin worlds rarely differ: the test proves little");
    eprintln!("why comparison stopped: {stops:#?}");
    if std::env::var_os("NI_LIST").is_some() {
        for f in &fails {
            let head = f.lines().next().unwrap_or("");
            let w1: Vec<&str> = f.split("--- world 1\n").nth(1).map(|x| x.split("--- world 2").next().unwrap().lines().collect()).unwrap_or_default();
            let w2: Vec<&str> = f.split("--- world 2\n").nth(1).map(|x| x.split("--- last actions").next().unwrap().lines().collect()).unwrap_or_default();
            let tail = |w: &[&str]| w[w.len().saturating_sub(7)..].iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
            eprintln!("NIFAIL {head} | W1 {} | W2 {}", tail(&w1), tail(&w2));
        }
    }
    assert!(fails.is_empty(), "{}", fails[0]);
}

/// If two states hash equal they must have the same future: perturb the internal fields that could
/// plausibly steer the engine, and require the full-state hash to change with them (or the future
/// to be identical when it does not). Template: the determinism review's `hole` probe.
#[test]
fn equal_hash_implies_same_future() {
    let (db, d) = decks();
    let (mut poked, mut hash_equal) = ([0u32; 6], [0u32; 6]);
    for seed in 0..40u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0xB07);
        let mut k = 0u64;
        loop {
            if let Status::GameOver(_) = g.advance() {
                break;
            }
            if k % 40 == 0 {
                for what in 0..6u8 {
                    let mut c = g.clone();
                    if !c.raw_state_mut().poke_for_hash_test(what) {
                        continue;
                    }
                    poked[what as usize] += 1;
                    if c.state_hash() != g.state_hash() {
                        continue;
                    }
                    hash_equal[what as usize] += 1;
                    let (mut o, mut r) = (g.clone(), rng.clone());
                    for _ in 0..200 {
                        let (s1, s2) = (o.advance(), c.advance());
                        assert!(s1 == s2 && o.state_hash() == c.state_hash(), "poke {what} left the hash equal but changed the future (seed {seed}, k {k})");
                        if let Status::GameOver(_) = s1 {
                            break;
                        }
                        let p = o.pending().unwrap();
                        let (id, len) = (p.id, p.options.len());
                        let idx = r.below(len as u64) as usize;
                        o.apply(id, idx).unwrap();
                        c.apply(id, idx).unwrap();
                    }
                }
            }
            let p = g.pending().unwrap();
            let (id, len) = (p.id, p.options.len());
            g.apply(id, rng.below(len as u64) as usize).unwrap();
            k += 1;
        }
    }
    eprintln!("pokes {poked:?}, hash stayed equal {hash_equal:?}");
    assert!(poked.iter().filter(|&&n| n > 0).count() >= 5);
}

/// A runaway token engine must end the game as a draw, not panic the engine (a long Ocelot Pride
/// game under random rollouts hit the 16-bit object arena in the first agent run). The arena is
/// pre-filled to just under the limit so the first or second token created ends the game.
#[test]
fn arena_exhaustion_is_a_draw_not_a_panic() {
    let (db, d) = decks();
    let (mut draws, mut games) = (0, 0);
    for seed in 0..400u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        g.raw_state_mut().fill_arena_for_test(mtg_core::state::RUNAWAY_OBJECTS - 2);
        let mut rng = Pcg64::from_seed(seed);
        let mut n = 0;
        let r = loop {
            match g.advance() {
                Status::GameOver(r) => break r,
                Status::NeedDecision(_) => {
                    let p = g.pending().unwrap();
                    let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                    n += 1;
                    if n > 3000 {
                        break mtg_core::decision::GameResult::Draw;
                    }
                }
            }
        };
        games += 1;
        draws += matches!(r, mtg_core::decision::GameResult::Draw) as u32;
    }
    eprintln!("{games} games with a nearly full arena, {draws} draws");
    assert!(draws > 0);
}

//! Reviewer probes (hidden-information review, 2026-10-01). Not part of the project.
use mtg_core::decision::Status;
use mtg_core::fork::HiddenRequest;
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_fuzz::load_deck_file;
use mtg_view::{BeliefModel, DeckList, Game, UniformConsistentModel};
use std::collections::BTreeMap;

fn decks() -> (std::sync::Arc<mtg_core::card::CardDb>, Vec<DeckList>) {
    let db = mtg_cards::legacy::build();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decks");
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let d = paths.iter().map(|p| load_deck_file(&db, p).0).collect();
    (db, d)
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

fn multiset(v: &[CardDefId]) -> BTreeMap<u16, i32> {
    let mut m = BTreeMap::new();
    for d in v {
        *m.entry(d.0).or_insert(0) += 1;
    }
    m
}

/// PoC 1: a fork of the *real* state, built with an agent-chosen seed, lets the agent recover the
/// opponent's true hand, using only: the decklists in the order given to `Game::new`, the
/// observer's own knowledge (`HiddenRequest`), the belief model it supplied, and
/// `fork.observe(opp)`.
#[test]
fn poc_fork_decodes_opponent_hand() {
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
        // hand cards the observer knows are not "hidden": skip states where it knows any
        if truth.is_empty() || !g.observe(obs).opp.revealed_hand.is_empty() {
            continue;
        }
        let h = truth.len();
        // unknown slots in ascending slot order, defs as given by the decklist order
        let mut unk: Vec<CardDefId> = deck_opp.clone();
        for k in &req.known[opp.idx()] {
            if let Some(i) = unk.iter().position(|x| x == k) {
                unk.remove(i);
            }
        }
        assert_eq!(unk.len(), req.unknown[opp.idx()]);
        let k_seeds = 48u64;
        let mut score = vec![0u32; unk.len()];
        for s in 0..k_seeds {
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
        let common: i32 = tm.iter().map(|(k, v)| (*v).min(*pm.get(k).unwrap_or(&0))).sum();
        cards_ok += common;
        cards_tot += h as i32;
        // baseline: expected overlap of a random hand drawn from the unseen multiset with the truth
        let um = multiset(&unk);
        let tot = unk.len() as f64;
        base_ok += tm.iter().map(|(k, v)| (*v as f64).min(*um.get(k).unwrap_or(&0) as f64 * h as f64 / tot)).sum::<f64>();
        if tm == pm {
            exact += 1;
        }
        states += 1;
    }
    eprintln!("states {states}: exact hand recovered {exact}, cards recovered {cards_ok}/{cards_tot}, random-guess baseline {:.1}", base_ok);
    assert!(states > 20);
}

/// PoC 2: raw `Game::pending()` shows `ObjRef` slots; slot == decklist index (CardId doc).
#[test]
fn poc_raw_pending_exposes_slots() {
    let (db, d) = decks();
    let mut shown = 0;
    let mut right = 0;
    for seed in 0..40u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 60 + (seed * 37) % 200) else { continue };
        let obs = g.pending().unwrap().seat;
        // The agent forks for its own seat and reads the opponent's pending decision of the FORK.
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let mut f = g.fork(obs, 5, &m).unwrap();
        // pass until the opponent has a decision
        for _ in 0..50 {
            match f.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(s) => {
                    if s != obs {
                        let p = f.pending().unwrap();
                        let opp = s;
                        let deck_opp = if opp == Seat::P0 { &a.main } else { &b.main };
                        let base = if opp == Seat::P0 { 0usize } else { a.main.len() };
                        let truth_g = g.raw_state();
                        for o in &p.options {
                            if let mtg_core::decision::Opt::PlayLand(r) | mtg_core::decision::Opt::Cast(r, _) = o {
                                let guess = deck_opp[r.slot as usize - base];
                                // is that slot in the TRUE opp hand with the same identity?
                                let in_true_hand = truth_g.hand(opp).iter().any(|x| x.slot == r.slot);
                                shown += 1;
                                if in_true_hand && truth_g.def_of(*r) == guess {
                                    right += 1;
                                }
                            }
                        }
                        break;
                    }
                    let (id, _) = { let p = f.pending().unwrap(); (p.id, 0) };
                    f.apply(id, 0).unwrap();
                }
            }
        }
    }
    eprintln!("options naming an opp hand card via raw slots: {shown}, true hand cards identified by decklist index: {right}");
}

/// PoC 3: the true library ORDER (both libraries) is recoverable from forks.
#[test]
fn poc_fork_decodes_library_order() {
    let (db, d) = decks();
    let (mut pos_ok, mut pos_tot, mut top5_ok, mut top5_tot, mut states) = (0usize, 0usize, 0usize, 0usize, 0);
    for seed in 0..80u64 {
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
        let target = obs.other(); // attack the opponent's library
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
            // replay the fork's position shuffles
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
            let asg_t = &asg.defs[target.idx()];
            // fork[j] holds the object that was at true index perm[j]
            for j in 0..n {
                let i = perm[j];
                for r in 0..unk.len() {
                    if asg_t[r] != lib_f[j] {
                        cand[i][r] = false;
                    }
                }
            }
        }
        let tl = st.library(target);
        // true rank is unknowable to us, but true DEF at index i is what we compare
        let mut pred: Vec<Option<CardDefId>> = Vec::new();
        for i in 0..n {
            let defs: std::collections::BTreeSet<u16> = (0..unk.len()).filter(|&r| cand[i][r]).map(|r| unk[r].0).collect();
            pred.push(if defs.len() == 1 { Some(CardDefId(*defs.iter().next().unwrap())) } else { None });
        }
        for i in 0..n {
            pos_tot += 1;
            let truth = st.def_of(tl[i]);
            if pred[i] == Some(truth) {
                pos_ok += 1;
            }
            if i + 5 >= n {
                top5_tot += 1;
                if pred[i] == Some(truth) {
                    top5_ok += 1;
                }
            }
        }
        states += 1;
    }
    eprintln!("states {states}: library positions recovered exactly {pos_ok}/{pos_tot}; top five {top5_ok}/{top5_tot}");
    assert!(states > 10);
}

/// PoC 4: the observer's next DecisionId depends on how many cards the opponent put on the
/// bottom while scrying (a hidden choice), because every `ask` of either seat bumps one counter.
#[test]
fn poc_decision_id_leaks_opponent_scry_choice() {
    use mtg_core::decision::{CardsPurpose, DecisionKind, Opt};
    let (db, d) = decks();
    let mut found = 0;
    let mut differ = 0;
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
                        // world 1: keep everything on top; world 2: bottom the first card, then keep the rest
                        let run = |bottom_first: bool| -> Option<(u32, String)> {
                            let mut w = g.clone();
                            let mut first = true;
                            for _ in 0..40 {
                                match w.advance() {
                                    Status::GameOver(_) => return None,
                                    Status::NeedDecision(s) => {
                                        let q = w.pending().unwrap().clone();
                                        if s == obs {
                                            let o = w.observe(obs);
                                            let dec = o.decision.as_ref().unwrap();
                                            return Some((dec.id.0, format!("{} hand={} opp_hand={} lib={} oplib={}", dec.options.len(), o.me.hand.len(), o.opp.hand_count, o.me.library_count, o.opp.library_count)));
                                        }
                                        let idx = if first && bottom_first {
                                            0
                                        } else {
                                            q.options.iter().position(|x| matches!(x, Opt::Done)).unwrap_or(0)
                                        };
                                        first = false;
                                        w.apply(q.id, idx).unwrap();
                                    }
                                }
                            }
                            None
                        };
                        if let (Some(x), Some(y)) = (run(false), run(true)) {
                            found += 1;
                            if x.0 != y.0 {
                                differ += 1;
                                if differ <= 3 {
                                    eprintln!("seed {seed}: observer's next decision id {} (keep all) vs {} (bottom one); other fields {:?} / {:?}", x.0, y.0, x.1, y.1);
                                }
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
    eprintln!("opp scry decisions examined {found}, observer decision id differed in {differ}");
}

/// PoC 5: the fork keeps the OPPONENT's own position-knowledge of its library (hidden to the
/// observer: scry/Brainstorm/Ponder choices) and its hand order / view ids.
#[test]
fn poc_fork_keeps_opponent_private_knowledge() {
    let (db, d) = decks();
    let (mut states, mut with_opp_knowledge, mut visible_in_fork) = (0, 0, 0);
    let mut example = String::new();
    for seed in 0..300u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 60 + (seed * 37) % 400) else { continue };
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let obs = g.pending().unwrap().seat;
        let opp = obs.other();
        let st = g.raw_state();
        let (t, bt) = st.known_library(opp, opp);
        let (to, bo) = st.known_library(opp, obs);
        states += 1;
        if (t.len() + bt.len()) > (to.len() + bo.len()) {
            with_opp_knowledge += 1;
            let f = g.fork(obs, 3, &m).unwrap();
            let fo = f.observe(opp);
            // The observer-facing view of the opponent says nothing about it...
            let mine = g.observe(obs);
            assert!(mine.opp.library_known_top.len() + mine.opp.library_known_bottom.len() == to.len() + bo.len());
            // ...but the fork's view from the opponent's seat has exactly the true structure.
            if fo.me.library_known_top.len() == t.len() && fo.me.library_known_bottom.len() == bt.len() {
                visible_in_fork += 1;
                if example.is_empty() {
                    example = format!("seed {seed}: opp truly knows top {} / bottom {} of its library; fork shows top {} / bottom {}", t.len(), bt.len(), fo.me.library_known_top.len(), fo.me.library_known_bottom.len());
                }
            }
        }
    }
    eprintln!("states {states}, opponent privately knows library positions in {with_opp_knowledge}, fork reproduces exactly that structure in {visible_in_fork}. {example}");
}

/// PoC 6 (structural form of non-interference): two states that differ ONLY in the hidden order of
/// the opponent's library (a different shuffle outcome) give different forks for the same seed.
#[test]
fn poc_forks_differ_for_library_order_twins() {
    let (db, d) = decks();
    let (mut n, mut differ, mut lib_def_differ) = (0, 0, 0);
    for seed in 0..120u64 {
        let (a, b) = (&d[(seed % 8) as usize], &d[((seed + 3) % 8) as usize]);
        let Some(g) = midgame(&db, a, b, seed, 40 + (seed * 37) % 300) else { continue };
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let obs = g.pending().unwrap().seat;
        let mut g2 = g.clone();
        g2.raw_state_mut().review_shuffle_unknown_order(obs, obs.other(), seed ^ 0x1234);
        // the twin is indistinguishable for the observer
        assert_eq!(format!("{:?}", g.observe(obs)), format!("{:?}", g2.observe(obs)));
        let (f1, f2) = (g.fork(obs, 7, &m).unwrap(), g2.fork(obs, 7, &m).unwrap());
        n += 1;
        if f1.state_hash() != f2.state_hash() {
            differ += 1;
        }
        let defs = |f: &Game| -> Vec<u16> { let s = f.raw_state(); s.library(obs.other()).iter().map(|&r| s.def_of(r).0).collect() };
        if defs(&f1) != defs(&f2) {
            lib_def_differ += 1;
        }
    }
    eprintln!("twins {n}: forks differ (state hash) in {differ}, sampled opponent library def order differs in {lib_def_differ}");
}

/// PoC 7: the project's own non-interference harness, run on the REAL pool (it only runs the test pool).
#[test]
fn ni_on_real_pool() {
    use mtg_debug::noninterference::*;
    let (db, d) = decks();
    let games: u64 = std::env::var("NI_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(150);
    let (mut compared, mut differing, mut fails) = (0u64, 0u64, Vec::new());
    let only: Option<(u64, u8, bool)> = std::env::var("NI_ONLY").ok().map(|v| { let p: Vec<&str> = v.split(',').collect(); (p[0].parse().unwrap(), p[1].parse().unwrap(), p[2] == "1") });
    for own in [false, true] {
        for seed in 0..games {
            if let Some((s, _, o)) = only { if s != seed || o != own { continue; } }
            let (a, b) = (&d[(seed % 8) as usize], &d[((seed / 8 + seed + 3) % 8) as usize]);
            for observer in [Seat::P0, Seat::P1] {
                if let Some((_, ob, _)) = only { if ob != observer.0 { continue; } }
                let cfg = NiConfig { observer, own_library: own, depth: 20 + (seed * 37) % 300, max_steps: 600 };
                if hidden_differs(&db, [a, b], seed, cfg) {
                    differing += 1;
                }
                match run_pair(&db, [a, b], seed, cfg) {
                    Ok(n) => compared += n,
                    Err(f) => fails.push((own, observer, f)),
                }
            }
        }
    }
    eprintln!("real pool NI: compared {compared} steps, hidden differed in {differing} pairs, failures {}", fails.len());
    for (own, ob, f) in fails.iter() {
        eprintln!("FAILHDR own_library={own} observer={} seed {} step {}", ob.0, f.seed, f.step);
    }
    if let Some((own, ob, f)) = fails.first() {
        eprintln!("FAIL own_library={own} observer={} seed {} step {}: {}", ob.0, f.seed, f.step, f.what);
    }
}

/// PoC 8: Show and Tell. The observer is the second chooser. Does the fork's opponent pick depend on
/// whether the true pick was "nothing"?
#[test]
fn poc_show_and_tell_fork_depends_on_true_pick() {
    use mtg_core::decision::{CardsPurpose, DecisionKind, Opt};
    let (db, d) = decks();
    let (mut n_true_none, mut n_true_some) = (0u32, 0u32);
    let (mut fork_some_given_none, mut fork_total_none) = (0u32, 0u32);
    let (mut fork_some_given_some, mut fork_total_some) = (0u32, 0u32);
    let bf_count = |g: &Game, who: Seat| -> usize { g.observe(who).battlefield.iter().filter(|p| !p.controlled_by_me).count() };
    for seed in 0..700u64 {
        let (a, b) = (&d[0], &d[(seed % 8) as usize]); // Alurentell casts S&T
        let m = UniformConsistentModel { decks: [a.main.clone(), b.main.clone()] };
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0x55);
        for _ in 0..3000 {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(seat) => {
                    let p = g.pending().unwrap().clone();
                    if matches!(p.kind, DecisionKind::ChooseCards { purpose: CardsPurpose::PutEach, .. }) && !g.observe(seat).active_is_me {
                        // observer = seat (second chooser). It picks "Done" (last option).
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
                            if bf_count(&f, seat) > bf0 {
                                some += 1;
                            }
                        }
                        if truth_entered {
                            n_true_some += 1;
                            fork_some_given_some += some;
                            fork_total_some += tot;
                        } else {
                            n_true_none += 1;
                            fork_some_given_none += some;
                            fork_total_none += tot;
                        }
                    }
                    let (id, n) = (p.id, p.options.len());
                    g.apply(id, rng.below(n as u64) as usize).unwrap();
                }
            }
        }
    }
    eprintln!(
        "S&T second-chooser states: true opp pick nothing {n_true_none}, something {n_true_some}. P(fork has opp enter something | truth nothing) = {}/{}, | truth something = {}/{}",
        fork_some_given_none, fork_total_none, fork_some_given_some, fork_total_some
    );
}

/// PoC 9: Thassa's Oracle puts the rest on the bottom in RANDOM order; the owner must not know that order.
#[test]
fn poc_oracle_random_bottom_knowledge() {
    use mtg_core::decision::{CardsPurpose, DecisionKind};
    let (db, d) = decks();
    let (mut oracle_resolutions, mut owner_knows_bottom) = (0, 0);
    for seed in 0..1500u64 {
        let a = &d[4]; // doomsday
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
                        oracle_resolutions += 1;
                        if before == 0 && !o.me.library_known_bottom.is_empty() {
                            owner_knows_bottom += 1;
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
    eprintln!("Oracle resolutions {oracle_resolutions}, owner shown a 'known' bottom run afterwards in {owner_knows_bottom}");
}

/// PoC 10: a Miracle card drawn by the opponent is announced on the stack (forced reveal).
#[test]
fn poc_miracle_forced_reveal() {
    let (db, d) = decks();
    let (mut seen, mut games_with_triumph_draw) = (0, 0);
    for seed in 0..400u64 {
        eprintln!("seed {seed}");
        let tri = db.id(&std::env::var("TRI").unwrap_or("Triumph of Saint Katherine".into())).unwrap();
        let mut a = d[7].clone();
        for i in 0..8 { a.main[i] = tri; }
        let a = &a;
        let b = &d[(seed % 8) as usize];
        let mut g = Game::new(db.clone(), [a, b], seed, GameConfig::default());
        let mut rng = Pcg64::from_seed(seed ^ 0x31);
        let mut hit = false;
        for _ in 0..2500 {
            match g.advance() {
                Status::GameOver(_) => break,
                Status::NeedDecision(seat) => {
                    let o = g.observe(Seat::P1);
                    if o.stack.iter().any(|s| s.is_ability && !s.controlled_by_me && s.name.starts_with("Triumph")) && !hit {
                        hit = true;
                        seen += 1;
                    }
                    let p = g.pending().unwrap();
                    let (id, n) = (p.id, p.options.len());
                    if std::env::var_os("TRI_TRACE").is_some() { let o = g.observe(seat); eprintln!("  {:?} stack={:?} opts={:?}", p.kind, o.stack.iter().map(|x| x.name.clone()).collect::<Vec<_>>(), o.decision.as_ref().map(|d| d.options.iter().map(|x| x.label.clone()).collect::<Vec<_>>())); }
                    let _ = seat;
                    g.apply(id, rng.below(n as u64) as usize).unwrap();
                }
            }
        }
        if hit { games_with_triumph_draw += 1; }
    }
    eprintln!("games where the opponent's drawn Triumph of Saint Katherine showed up as a stack ability to the observer: {seen} (games {games_with_triumph_draw})");
}

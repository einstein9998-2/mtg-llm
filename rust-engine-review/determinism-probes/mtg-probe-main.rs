use mtg_core::decision::*;
use mtg_core::hash::Fx64;
use mtg_core::ids::*;
use mtg_core::rng::Pcg64;
use mtg_core::state::{GameConfig, PERTURB_CTR, PERTURB_MODE};
use mtg_view::*;
use std::hash::Hasher;
use std::sync::atomic::Ordering;
use std::sync::Arc;

type Db = Arc<mtg_core::card::CardDb>;

fn load() -> (Db, Vec<DeckList>) {
    let db = mtg_cards::legacy::build();
    let mut paths: Vec<_> = std::fs::read_dir("/mnt/project-files/decks").unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    let d = paths.iter().map(|p| mtg_fuzz::load_deck_file(&db, p).0).collect();
    (db, d)
}

fn fx(s: &str) -> u64 {
    let mut h = Fx64::default();
    h.write(s.as_bytes());
    h.finish()
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
struct Dig {
    n: u64,
    state: u64,
    obs: u64,
    res: String,
}

fn pair(i: u64, nd: usize) -> (usize, usize) {
    let n = nd as u64;
    let a = (i % n) as usize;
    let b = ((a as u64 + 1 + (i / n) % (n - 1)) % n) as usize;
    (a, b)
}

/// Play a random game, return digest. `with_obs`: hash agent-visible observations too.
fn play(db: &Db, a: &DeckList, b: &DeckList, seed: u64, with_obs: bool, observe_always: bool) -> Dig {
    PERTURB_CTR.store(0, Ordering::Relaxed);
    let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
    let mut g = Game::new(db.clone(), [a, b], seed, cfg);
    let mut rng = Pcg64::from_seed(seed ^ 0xB07);
    let mut d = Dig::default();
    let mut hs = Fx64::default();
    let mut ho = Fx64::default();
    loop {
        let st = g.advance();
        match st {
            Status::GameOver(r) => {
                d.res = format!("{r:?}");
                break;
            }
            Status::NeedDecision(seat) => {
                hs.write_u64(g.state_hash());
                if with_obs {
                    let o = g.observe(seat);
                    ho.write_u64(fx(&format!("{o:?}")));
                } else if observe_always {
                    let _ = g.observe(seat);
                    let _ = g.observe(seat.other());
                }
                let p = g.pending().unwrap();
                let (id, n) = (p.id, p.options.len());
                let idx = rng.below(n as u64) as usize;
                g.apply(id, idx).unwrap();
                d.n += 1;
                if d.n > 20000 {
                    d.res = "trunc".into();
                    break;
                }
            }
        }
    }
    hs.write_u64(g.state_hash());
    d.state = hs.finish();
    d.obs = ho.finish();
    d
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args[1].as_str();
    let seed0: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let n: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(10);
    let (db, decks) = load();
    match cmd {
        // traj <seed0> <n> [mode] [stack_kb]: print digests (cross-process, slot-perturbation)
        "traj" => {
            let mode: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            let stack_kb: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            PERTURB_MODE.store(mode, Ordering::Relaxed);
            let body = move || {
                for i in 0..n {
                    let seed = seed0 + i;
                    let (a, b) = pair(i, decks.len());
                    let d = play(&db, &decks[a], &decks[b], seed, true, false);
                    println!("{seed} {a}v{b} n={} res={} state={:016x} obs={:016x}", d.n, d.res, d.state, d.obs);
                }
            };
            if stack_kb > 0 {
                std::thread::Builder::new().stack_size(stack_kb * 1024).spawn(body).unwrap().join().unwrap();
            } else {
                body();
            }
        }
        // lock <seed0> <n> <mode>: lockstep baseline vs perturbed, report first observation difference
        "lock" => {
            let mode: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
            let mut ndiff = 0;
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g0 = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut g1 = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                let mut k = 0;
                loop {
                    PERTURB_MODE.store(0, Ordering::Relaxed);
                    let s0 = g0.advance();
                    PERTURB_MODE.store(mode, Ordering::Relaxed);
                    let s1 = g1.advance();
                    if s0 != s1 { println!("seed {seed}: status differs at {k}: {s0:?} vs {s1:?}"); ndiff += 1; break; }
                    let seat = match s0 { Status::GameOver(_) => break, Status::NeedDecision(s) => s };
                    let (o0, o1) = (format!("{:?}", g0.observe(seat)), format!("{:?}", g1.observe(seat)));
                    if o0 != o1 {
                        ndiff += 1;
                        println!("seed {seed} {}v{} decision #{k}: OBSERVATION DIFFERS", a, b);
                        let p0 = g0.pending().unwrap(); let p1 = g1.pending().unwrap();
                        println!(" kind {:?}\n p0 {:?}\n p1 {:?}", p0.kind, p0.options, p1.options);
                        // find first differing char region
                        let bytes0 = o0.as_bytes(); let bytes1 = o1.as_bytes();
                        let mut j = 0; while j < bytes0.len().min(bytes1.len()) && bytes0[j]==bytes1[j] { j+=1; }
                        let lo = j.saturating_sub(300);
                        println!(" o0 ...{}", &o0[lo..(j+300).min(o0.len())]);
                        println!(" o1 ...{}", &o1[lo..(j+300).min(o1.len())]);
                        break;
                    }
                    let p = g0.pending().unwrap();
                    let (id, len) = (p.id, p.options.len());
                    let idx = rng.below(len as u64) as usize;
                    PERTURB_MODE.store(0, Ordering::Relaxed);
                    g0.apply(id, idx).unwrap();
                    PERTURB_MODE.store(mode, Ordering::Relaxed);
                    g1.apply(id, idx).unwrap();
                    k += 1;
                    if k > 20000 { break; }
                }
            }
            println!("games with differences: {ndiff} of {n}");
        }
        // clone <seed0> <n> <every> <variant>: clone at every decision (multiple of every), continue both
        "clone" => {
            let every: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(17);
            let variant: u32 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let mut bad = 0; let mut total = 0;
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                let mut hist: Vec<u64> = Vec::new();
                let mut snaps: Vec<(usize, Game, Pcg64)> = Vec::new();
                loop {
                    let st = g.advance();
                    let seat = match st { Status::GameOver(_) => { hist.push(g.state_hash()); break }, Status::NeedDecision(s) => s };
                    hist.push(g.state_hash());
                    if (hist.len() as u64 - 1) % every == 0 { snaps.push((hist.len() - 1, g.clone(), rng.clone())); }
                    let _ = seat;
                    let p = g.pending().unwrap();
                    let (id, len) = (p.id, p.options.len());
                    let idx = rng.below(len as u64) as usize;
                    g.apply(id, idx).unwrap();
                    if hist.len() > 20000 { break; }
                }
                for (k, mut c, mut r) in snaps {
                    total += 1;
                    if variant == 1 { c.raw_state_mut().mark_derived_dirty(); }
                    let mut j = k;
                    let mut ok = true;
                    loop {
                        let st = c.advance();
                        if variant == 2 { if let Status::NeedDecision(s) = st { let _ = c.observe(s); let _ = c.observe(s.other()); } }
                        if j >= hist.len() || c.state_hash() != hist[j] { ok = false; println!("seed {seed}: clone@{k} diverged at {j}"); break; }
                        match st { Status::GameOver(_) => break, Status::NeedDecision(_) => {
                            let p = c.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                            let idx = r.below(len as u64) as usize;
                            c.apply(id, idx).unwrap(); j += 1;
                        } }
                    }
                    if !ok { bad += 1; break; }
                }
            }
            println!("clone variant {variant}: {bad} diverging games; {total} clones checked");
        }
        // fork <seed0> <n> <every>
        "fork" => {
            let every: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(19);
            let mut bad = 0u64; let mut total = 0u64; let mut samehash_diffseed = 0u64;
            let mut dig = Fx64::default();
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let model = UniformConsistentModel { decks: [decks[a].main.clone(), decks[b].main.clone()] };
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                let mut k = 0u64;
                loop {
                    let st = g.advance();
                    let seat = match st { Status::GameOver(_) => break, Status::NeedDecision(s) => s };
                    if k % every == 0 {
                        total += 1;
                        let h0 = g.state_hash();
                        let fs = seed * 7919 + k;
                        let f1 = g.fork(seat, fs, &model);
                        let f2 = g.fork(seat, fs, &model);
                        let f3 = g.clone().fork(seat, fs, &model);
                        let f4 = g.fork(seat, fs + 1, &model);
                        if g.state_hash() != h0 { println!("seed {seed} k {k}: fork mutated original"); bad += 1; }
                        match (f1, f2, f3, f4) {
                            (Ok(mut f1), Ok(mut f2), Ok(f3), Ok(f4)) => {
                                dig.write_u64(f1.state_hash());
                                if f1.state_hash() != f2.state_hash() || f1.state_hash() != f3.state_hash() { println!("seed {seed} k {k}: same-seed forks differ"); bad += 1; }
                                if f1.state_hash() == f4.state_hash() { samehash_diffseed += 1; }
                                let mut r1 = Pcg64::from_seed(fs ^ 5);
                                let mut r2 = r1.clone();
                                let mut steps = 0;
                                loop {
                                    let s1 = f1.advance(); let s2 = f2.advance();
                                    if s1 != s2 || f1.state_hash() != f2.state_hash() { println!("seed {seed} k {k}: fork continuation diverged at step {steps}"); bad += 1; break; }
                                    if let Status::GameOver(_) = s1 { break; }
                                    let p1 = f1.pending().unwrap(); let (id, len) = (p1.id, p1.options.len());
                                    f1.apply(id, r1.below(len as u64) as usize).unwrap();
                                    f2.apply(id, r2.below(len as u64) as usize).unwrap();
                                    steps += 1;
                                    if steps > 3000 { break; }
                                }
                                dig.write_u64(f1.state_hash());
                            }
                            (a1, _, _, _) => { if !matches!(a1, Err(_)) { println!("odd"); } else { println!("seed {seed} k {k}: fork error {:?}", a1.err()); bad += 1; } }
                        }
                    }
                    let p = g.pending().unwrap();
                    let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                    k += 1;
                }
            }
            println!("fork: {total} forks, {bad} problems, {samehash_diffseed} different-seed forks with equal hash; digest {:016x}", dig.finish());
        }
        // rec <seed0> <n> <dir>: record with a checkpoint at every action
        "rec" => {
            let dir = args.get(4).cloned().unwrap_or("/tmp/review-det/recs".into());
            std::fs::create_dir_all(&dir).unwrap();
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let first = (seed % 2) as u8;
                let cfg = GameConfig { first_player: Seat(first), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                let mut rec = GameRecord::new(&db, [&decks[a], &decks[b]], seed, first);
                loop {
                    let st = g.advance();
                    rec.checkpoints.push((rec.actions.len() as u32, g.state_hash()));
                    if let Status::GameOver(_) = st { break; }
                    let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                    let idx = rng.below(len as u64) as usize;
                    rec.actions.push((id.0, idx as u16));
                    g.apply(id, idx).unwrap();
                }
                std::fs::write(format!("{dir}/r{seed}.rec"), rec.to_text()).unwrap();
            }
        }
        "rep" => {
            let dir = args.get(2).cloned().unwrap();
            let mut ok = 0; let mut bad = 0;
            let mut paths: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).collect();
            paths.sort();
            for p in paths {
                let rec = GameRecord::from_text(&std::fs::read_to_string(&p).unwrap()).unwrap();
                match replay(db.clone(), &rec) { Ok(_) => ok += 1, Err(e) => { bad += 1; println!("{}: {e:?}", p.display()); } }
            }
            println!("replayed ok {ok}, failed {bad}");
        }
        // tamper <seed>: record-format weaknesses
        "tamper" => {
            let seed = seed0;
            let (a, b) = (0usize, 3usize);
            let mut mk = |cfg: GameConfig| {
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                let mut rec = GameRecord::new(&db, [&decks[a], &decks[b]], seed, cfg.first_player.0);
                loop {
                    let st = g.advance();
                    if rec.actions.len() % 50 == 0 || matches!(st, Status::GameOver(_)) { rec.checkpoints.push((rec.actions.len() as u32, g.state_hash())); }
                    if let Status::GameOver(_) = st { break; }
                    let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                    let idx = rng.below(len as u64) as usize;
                    rec.actions.push((id.0, idx as u16));
                    g.apply(id, idx).unwrap();
                }
                rec
            };
            let base = mk(GameConfig::default());
            println!("baseline replay: {:?}", replay(db.clone(), &base).map(|_| "ok"));
            // (a) wrong value for a checkpoint beyond the action list
            let mut r = base.clone(); r.checkpoints.push((base.actions.len() as u32 + 5, 0xdead));
            println!("(a) bogus checkpoint past end: {:?}", replay(db.clone(), &r).map(|_| "ACCEPTED"));
            // (b) an unmatched early checkpoint (n between multiples) shadows later ones: put a bogus out-of-order checkpoint first
            let mut r = base.clone(); r.checkpoints.insert(0, (3, base.checkpoints.iter().find(|c| c.0 == 3).map(|c| c.1).unwrap_or(1)));
            // checkpoint (3) isn't in base (only multiples of 50 recorded) so value is bogus 1: peek sees n=3 at i=3 -> mismatch expected
            println!("(b0) bogus cp at 3: {:?}", replay(db.clone(), &r).map(|_| "ACCEPTED"));
            // (b) checkpoint list out of order: [(100, wrong), (50, ok)...]: 100 first blocks 50 until i==100
            let mut r = base.clone(); r.checkpoints.reverse();
            println!("(b) reversed checkpoint order (all correct values): {:?}", replay(db.clone(), &r).map(|_| "ACCEPTED"));
            let mut r = base.clone(); let last = r.checkpoints.len() - 1; r.checkpoints.swap(0, last); // final first
            r.checkpoints[1].1 ^= 1; // corrupt a mid checkpoint
            println!("(b2) corrupt mid checkpoint with final cp first: {:?}", replay(db.clone(), &r).map(|_| "ACCEPTED (corruption missed)"));
            // (c) no checkpoints at all
            let mut r = base.clone(); r.checkpoints.clear();
            println!("(c) no checkpoints: {:?}", replay(db.clone(), &r).map(|_| "ACCEPTED"));
            // (d) non-default config not recorded
            let cfg = GameConfig { first_player: Seat(0), starting_life: 25, hand_size: 7, explicit_mana: false };
            let r = mk(cfg);
            println!("(d) starting_life=25 record replay: {:?}", replay(db.clone(), &r).map(|_| "ok"));
            let cfg = GameConfig { first_player: Seat(0), starting_life: 20, hand_size: 7, explicit_mana: true };
            let r = mk(cfg);
            println!("(d2) explicit_mana=true record replay: {:?}", replay(db.clone(), &r).map(|_| "ok"));
            // (e) truncated action list still 'ok'
            let mut r = base.clone(); r.actions.truncate(r.actions.len() / 2); r.checkpoints.retain(|c| (c.0 as usize) < r.actions.len());
            println!("(e) truncated: {:?}", replay(db.clone(), &r).map(|g| format!("ok, game over={}", g.result().is_some())));
        }
        // hole <seed0> <n>: which unhashed fields change future behaviour while the hash stays equal
        "hole" => {
            let mut stat = [[0u64; 3]; 6]; // [poked, hash_equal, diverged_later]
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                let mut k = 0u64;
                loop {
                    let st = g.advance();
                    if let Status::GameOver(_) = st { break; }
                    if k % 40 == 0 {
                        for what in 0..6u8 {
                            let mut c = g.clone();
                            if !c.raw_state_mut().probe_poke(what) { continue; }
                            stat[what as usize][0] += 1;
                            let eq = c.state_hash() == g.state_hash();
                            if eq { stat[what as usize][1] += 1; }
                            // continue both with identical actions for up to 400 decisions, look for hash divergence
                            let mut o = g.clone();
                            let mut r = rng.clone();
                            let mut div = false;
                            for _ in 0..400 {
                                let (s1, s2) = (o.advance(), c.advance());
                                if s1 != s2 || o.state_hash() != c.state_hash() {
                                    if !(eq && false) { div = true; }
                                    break;
                                }
                                if let Status::GameOver(_) = s1 { break; }
                                let p = o.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                                let idx = r.below(len as u64) as usize;
                                o.apply(id, idx).unwrap(); if c.apply(id, idx).is_err() { div = true; break; }
                            }
                            // divergence counted only when states were hash-equal at the start
                            if eq && div { stat[what as usize][2] += 1; }
                        }
                    }
                    let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                    k += 1;
                }
            }
            let names = ["free-list order", "ts_counter", "moved", "cfg.starting_life", "steps", "derived_dirty flag"];
            for w in 0..6 { println!("{:20} poked {:6} hash-equal {:6} later-diverged {:6}", names[w], stat[w][0], stat[w][1], stat[w][2]); }
        }
        // collapse: card_options soundness probes
        "collapse" => {
            use mtg_core::scenario::*;
            use mtg_core::types::Step;
            let mk = |bf: Vec<PermSetup>| ScenarioSetup { turn: 3, active: Seat(0), step: Step::Main1, players: [PlayerSetup { life: 20, ..Default::default() }, PlayerSetup { life: 20, battlefield: bf, ..Default::default() }], seed: 1 };
            let mut g = Game::from_scenario(db.clone(), &mk(vec![PermSetup::new("Containment Priest"), PermSetup::new("Containment Priest")]));
            let refs: Vec<ObjRef> = g.raw_state().battlefield().to_vec();
            println!("battlefield {:?}", refs);
            g.raw_state_mut().probe_pump(refs[1]);
            let dbc = db.clone();
            let st = g.raw_state_mut();
            let mut cx = mtg_core::cx::Cx::new(st, &dbc);
            cx.refresh();
            for r in &refs { let o = cx.s.obj_data(&dbc, *r); println!("  {:?} power/toughness {}/{}", r, o.chars.power, o.chars.toughness); }
            let opts = mtg_core::resolve::card_options(&cx, Seat(1), &refs);
            println!("card_options (edict choice) offers {} option(s) for two creatures with different derived P/T: {:?}", opts.len(), opts);
            // Skyclave Apparition x2 with different linked exiles
            let mut g = Game::from_scenario(db.clone(), &mk(vec![PermSetup::new("Skyclave Apparition"), PermSetup::new("Skyclave Apparition")]));
            let refs: Vec<ObjRef> = g.raw_state().battlefield().to_vec();
            g.raw_state_mut().probe_link(refs[0], 4);
            g.raw_state_mut().probe_link(refs[1], 1);
            let st = g.raw_state_mut();
            let mut cx = mtg_core::cx::Cx::new(st, &dbc);
            cx.refresh();
            let opts = mtg_core::resolve::card_options(&cx, Seat(1), &refs);
            println!("two Skyclave Apparitions linked to a MV4 and a MV1 card: edict offers {} option(s): {:?}", opts.len(), opts);
        }
        // keepone: Ajani -4 (KeepOneOfEach): do two different first picks give the same state hash at the second decision?
        "keepone" => {
            use mtg_core::scenario::*;
            use mtg_core::types::{Step, CounterKind};
            let mut aj = PermSetup::new("Ajani, Nacatl Avenger"); aj.counters = vec![(CounterKind::Loyalty, 5)];
            let sc = ScenarioSetup { turn: 3, active: Seat(0), step: Step::Main1,
                players: [PlayerSetup { life: 20, battlefield: vec![aj], library: vec!["Island".to_string(); 5], ..Default::default() },
                          PlayerSetup { life: 20, battlefield: vec![PermSetup::new("Disruptor Flute"), PermSetup::new("Cori-Steel Cutter"), PermSetup::new("Containment Priest"), PermSetup::new("Phelia, Exuberant Shepherd")], library: vec!["Island".to_string(); 5], ..Default::default() }], seed: 1 };
            let mut g = Game::from_scenario(db.clone(), &sc);
            // drive: first Activate option, then pass until KeepOne decision
            let mut guard = 0;
            loop {
                let st = g.advance();
                if let Status::GameOver(_) = st { println!("game over"); return; }
                let p = g.pending().unwrap().clone();
                guard += 1; if guard > 60 { println!("no keepone"); return; }
                if std::env::var_os("KO_TRACE").is_some() { println!("{guard}: {:?} {:?}", p.kind, p.options); }
                if let DecisionKind::ChooseCards { purpose: CardsPurpose::KeepOne, .. } = p.kind { break; }
                let idx = p.options.iter().position(|o| matches!(o, Opt::Activate { ability: 3, .. })).unwrap_or(0);
                // never activate twice: after stack non-empty choose Pass
                let idx = if g.raw_state().stack().is_empty() && guard < 3 { idx } else { 0 };
                g.apply(p.id, idx).unwrap();
            }
            let p = g.pending().unwrap().clone();
            println!("first KeepOne decision: {} options {:?}", p.options.len(), p.options);
            let mut hs = vec![];
            let mut outcomes = vec![];
            for pick in 0..p.options.len() {
                let mut c = g.clone();
                c.apply(p.id, pick).unwrap();
                c.advance();
                let p2 = c.pending().unwrap().clone();
                hs.push(c.state_hash());
                // finish with option 0 and report what survives
                let mut c2 = c.clone();
                println!("pick {pick}: second decision kind {:?} options {:?} hash {:016x}", p2.kind, p2.options, c.state_hash());
                loop {
                    let st = c2.advance(); if let Status::GameOver(_) = st { break; }
                    let q = c2.pending().unwrap().clone();
                    if matches!(q.kind, DecisionKind::ChooseCards { purpose: CardsPurpose::KeepOne, .. }) { c2.apply(q.id, 0).unwrap(); continue; }
                    break;
                }
                let names: Vec<String> = c2.raw_state().battlefield().iter().map(|&r| db.def(c2.raw_state().def_of(r)).name.clone()).collect();
                println!("   survivors after finishing with option 0: {:?}", names);
                outcomes.push(names);
            }
            println!("hash equal at second decision for different first picks: {}  | final boards differ: {}", hs.len() > 1 && hs[0] == hs[1], outcomes.len() > 1 && outcomes[0] != outcomes[1]);
        }
        // gens <seed0> <n>: headroom of generation counters and slot growth
        "gens" => {
            let (mut maxgen, mut maxslots, mut maxfree) = (0u16, 0usize, 0usize);
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                loop {
                    let st = g.advance();
                    if let Status::GameOver(_) = st { break; }
                    let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                }
                let s = g.raw_state();
                maxslots = maxslots.max(s.num_slots());
                maxfree = maxfree.max(s.free_slots().len());
                for sl in 0..s.num_slots() { maxgen = maxgen.max(s.obj_raw_zone(sl as u16).2); }
            }
            println!("max gen {maxgen} (wraps at 65536), max slots {maxslots}, max free list {maxfree}");
        }
        // stale <seed0> <n>: count view_id lookups with a stale ObjRef while observing every decision
        "stale" => {
            let mut dec = 0u64;
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                loop {
                    let st = g.advance();
                    let seat = match st { Status::GameOver(_) => break, Status::NeedDecision(s) => s };
                    let before = mtg_core::read::STALE_VIEW_ID.load(Ordering::Relaxed);
                    let _ = g.observe(seat); let _ = g.observe(seat.other());
                    let after = mtg_core::read::STALE_VIEW_ID.load(Ordering::Relaxed);
                    if after != before && std::env::var_os("STALE_VERBOSE").is_some() { let p = g.pending().unwrap(); println!("seed {seed}: stale view_id at kind {:?}", p.kind); }
                    dec += 1;
                    let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                }
            }
            println!("timestamp ties between different objects' statics: {}", mtg_core::derive::TS_TIES.load(Ordering::Relaxed));
            println!("decisions observed {dec}; stale view_id lookups {}", mtg_core::read::STALE_VIEW_ID.load(Ordering::Relaxed));
        }
        // fresh <seed0> <n>: are observations built from a stale derived cache?
        "fresh" => {
            let (mut dec, mut dirty, mut differ) = (0u64, 0u64, 0u64);
            let mut shown = 0;
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut rng = Pcg64::from_seed(seed ^ 0xB07);
                loop {
                    let st = g.advance();
                    let seat = match st { Status::GameOver(_) => break, Status::NeedDecision(s) => s };
                    dec += 1;
                    if g.raw_state().derived_is_dirty() {
                        dirty += 1;
                        let o1 = format!("{:?}", g.observe(seat));
                        let mut c = g.clone();
                        { let dbc = db.clone(); let mut cx = mtg_core::cx::Cx::new(c.raw_state_mut(), &dbc); cx.refresh(); }
                        let o2 = format!("{:?}", c.observe(seat));
                        if o1 != o2 { differ += 1; if shown < 3 { shown += 1; let p = g.pending().unwrap(); println!("seed {seed}: stale observation at {:?}", p.kind); } }
                    }
                    let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                    g.apply(id, rng.below(len as u64) as usize).unwrap();
                }
            }
            println!("decisions {dec}; derived cache dirty at {dirty}; observation differs from refreshed one at {differ}");
        }
        // pol <seed0> <n> <policy: first|last|mid|alt>: degenerate policies, look for non-terminating play
        "pol" => {
            let pol = args.get(4).cloned().unwrap_or("last".into());
            let (mut trunc, mut total, mut maxd) = (0u64, 0u64, 0u64);
            for i in 0..n {
                let seed = seed0 + i;
                let (a, b) = pair(i, decks.len());
                let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
                let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
                let mut d = 0u64;
                loop {
                    let st = g.advance();
                    if let Status::GameOver(_) = st { break; }
                    let p = g.pending().unwrap(); let (id, len, seat) = (p.id, p.options.len(), p.seat);
                    let idx = match pol.as_str() { "first" => 0, "last" => len - 1, "mid" => len / 2, _ => if (d + seat.idx() as u64) % 2 == 0 { 0 } else { len - 1 } };
                    g.apply(id, idx).unwrap();
                    d += 1;
                    if d > 60000 { trunc += 1; if std::env::var_os("POL_VERBOSE").is_some() { println!("seed {seed} {a}v{b} truncated"); } break; }
                }
                total += 1; maxd = maxd.max(d);
            }
            println!("policy {pol}: {total} games, {trunc} not finished within 60000 decisions, longest {maxd}");
        }
        // loopinfo <seed> <deckA> <deckB>: look at the tail of a 'mid' policy game
        "loopinfo" => {
            let a: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap(); let b: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap();
            let seed = seed0;
            let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
            let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
            let mut d = 0u64;
            let mut hs = std::collections::BTreeMap::new();
            loop {
                let st = g.advance();
                if let Status::GameOver(_) = st { println!("over"); break; }
                let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                if d == 59905 || d == 59906 {
                    let o = g.observe(p.seat);
                    println!("--- decision {d} seat {:?} pool {:?} hand {:?}", p.seat, o.me.pool, o.me.hand.iter().map(|c| c.name.clone()).collect::<Vec<_>>());
                    println!("    bf {:?}", o.battlefield.iter().map(|c| format!("{}{}{}", c.name, if c.controlled_by_me {"(me)"} else {"(opp)"}, if c.tapped {"T"} else {""})).collect::<Vec<_>>());
                    println!("    stack {:?} events {:?}", o.stack.iter().map(|x| x.name.clone()).collect::<Vec<_>>(), o.events.iter().take(12).collect::<Vec<_>>());
                }
                if d > 59900 { 
                    let h = g.state_hash(); hs.entry(h).or_insert(0u32); *hs.get_mut(&h).unwrap() += 1;
                    if d < 59915 { let o = g.observe(p.seat); println!("{d}: turn {} {:?} life {}/{} kind {:?} opts {:?}", o.turn, o.step, o.me.life, o.opp.life, p.kind, o.decision.as_ref().map(|x| x.options.iter().map(|y| y.label.clone()).collect::<Vec<_>>())); }
                }
                g.apply(id, len / 2).unwrap();
                d += 1;
                if d > 60000 { break; }
            }
            println!("distinct state hashes in last 100 decisions: {}", hs.len());
        }
        // slots <seed> <deckA> <deckB> <maxd>: slot growth under the looping 'mid' policy
        "slots" => {
            let a: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap(); let b: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap();
            let maxd: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap();
            let seed = seed0;
            let cfg = GameConfig { first_player: Seat((seed % 2) as u8), ..GameConfig::default() };
            let mut g = Game::new(db.clone(), [&decks[a], &decks[b]], seed, cfg);
            g.set_track_view_events(false);
            let mut d = 0u64;
            let t = std::time::Instant::now();
            loop {
                let st = g.advance();
                if let Status::GameOver(_) = st { println!("over at {d}"); break; }
                let p = g.pending().unwrap(); let (id, len) = (p.id, p.options.len());
                if d % 20000 == 0 { println!("d={d} slots={} stack={} free={} t={:.1}s", g.raw_state().num_slots(), g.raw_state().stack().len(), g.raw_state().free_slots().len(), t.elapsed().as_secs_f64()); }
                g.apply(id, len / 2).unwrap();
                d += 1;
                if d > maxd { break; }
            }
            println!("end d={d} slots={} stack={}", g.raw_state().num_slots(), g.raw_state().stack().len());
        }
        // alloc: slot index wraparound
        "alloc" => {
            let mut g = Game::new(db.clone(), [&decks[0], &decks[1]], 1, GameConfig::default());
            g.advance();
            let z0 = g.raw_state().obj_raw_zone(0).0;
            let (len, last, z_after) = g.raw_state_mut().probe_alloc(65_536 + 8 - 120);
            println!("slots now {len} (u16 max 65535); last ObjRef {:?}; card slot 0 zone before {:?}, raw zone code after touching the new object {}", last, z0, z_after);
            println!("card slot {} (a real card, was Library) now: {:?}", last.slot, g.raw_state().obj_raw_zone(last.slot));
        }
        // obsonly: like traj but prints only obs digest (state hash differs under perturbation)
        _ => panic!("unknown"),
    }
}

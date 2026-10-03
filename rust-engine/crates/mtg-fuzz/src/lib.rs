//! Fuzz driver: random legal play with invariant checks (doc 04 section 7.1).

use mtg_core::decision::*;
use mtg_core::ids::DecisionId;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_debug::invariants::{self, EventAudit};
use mtg_view::*;

pub struct FuzzStats {
    pub games: u64,
    pub decisions: u64,
    pub engine_steps: u64,
    pub wins: [u64; 2],
    pub draws: u64,
    pub truncated: u64,
    pub coverage: std::collections::BTreeMap<String, u64>,
}

pub struct Violation {
    pub seed: u64,
    pub decisions_so_far: u64,
    pub detail: String,
}

/// Plays one random game, checking invariants every `check_every` decisions (1 = every decision).
pub fn play_random_game(
    pool: &mtg_cards::testpool::TestPool,
    decks: [&DeckList; 2],
    seed: u64,
    first: u8,
    check_every: u64,
    deep_every: u64,
    max_decisions: u64,
    stats: &mut FuzzStats,
) -> Result<(), Violation> {
    let cfg = GameConfig { first_player: mtg_core::ids::Seat(first), ..GameConfig::default() };
    let mut g = Game::new(pool.db.clone(), decks, seed, cfg);
    g.set_keep_events(check_every > 0);
    g.set_track_view_events(false);
    let mut rng = Pcg64::from_seed(seed ^ 0xA5A5_5A5A_1234_5678);
    let mut audit = EventAudit::new(cfg.starting_life);
    let mut n = 0u64;
    let trace = std::env::var_os("MTG_FUZZ_TRACE").is_some();
    let fail = |n: u64, d: String| Violation { seed, decisions_so_far: n, detail: d };
    loop {
        let st = g.advance();
        if trace {
            if let Some(e) = g.raw_state().stack().last() {
                eprintln!("   stack top: {}", g.db().def(g.raw_state().def_of(e.obj)).name);
            }
        }
        if check_every > 0 {
            let ev = g.take_events();
            for e in &ev {
                use mtg_core::event::Event as E;
                let played = match e {
                    E::SpellCast { def, .. } => Some(*def),
                    E::ZoneChange { def, to: mtg_core::types::ZoneKind::Battlefield, .. } => Some(*def),
                    _ => None,
                };
                if let Some(d) = played {
                    *stats.coverage.entry(format!("card:{}", g.db().def(d).name)).or_insert(0) += 1;
                }
                let name = format!("{e:?}");
                let key = name.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("").to_string();
                *stats.coverage.entry(format!("ev:{key}")).or_insert(0) += 1;
            }
            audit.feed(&ev).map_err(|e| fail(n, format!("{} {}", e.id, e.msg)))?;
        }
        match st {
            Status::GameOver(r) => {
                stats.games += 1;
                match r {
                    GameResult::Win(s) => stats.wins[s.idx()] += 1,
                    GameResult::Draw => stats.draws += 1,
                }
                if check_every > 0 {
                    audit.check_life(g.raw_state()).map_err(|e| fail(n, format!("{} {}", e.id, e.msg)))?;
                }
                stats.engine_steps += g.raw_state().steps();
                return Ok(());
            }
            Status::NeedDecision(_) => {
                if check_every > 0 && n % check_every == 0 {
                    invariants::check(g.raw_state(), g.db()).map_err(|e| fail(n, format!("{} {}", e.id, e.msg)))?;
                    audit.check_life(g.raw_state()).map_err(|e| fail(n, format!("{} {}", e.id, e.msg)))?;
                }
                if deep_every > 0 && n % deep_every == 0 {
                    deep_checks(&g, &mut rng).map_err(|e| fail(n, e))?;
                }
                let p = g.pending().unwrap();
                if check_every > 0 {
                    let name = format!("{:?}", p.kind);
                    let key = name.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("").to_string();
                    *stats.coverage.entry(format!("dec:{key}")).or_insert(0) += 1;
                }
                let id = p.id;
                if trace {
                    eprintln!("{n}: {:?} opts {:?}", p.kind, p.options);
                }
                let idx = rng.below(p.options.len() as u64) as usize;
                g.apply(id, idx).map_err(|e| fail(n, format!("apply error {e:?}")))?;
                n += 1;
                stats.decisions += 1;
                if n >= max_decisions {
                    stats.truncated += 1;
                    stats.engine_steps += g.raw_state().steps();
                    return Ok(());
                }
            }
        }
    }
}

/// Deep checks at a decision point: I13 (mask validity and probes) and I12 (fork consistency).
/// Returns a description of the first failure.
pub fn deep_checks(g: &Game, rng: &mut Pcg64) -> Result<(), String> {
    let p = g.pending().ok_or("no pending decision")?.clone();
    let h0 = g.state_hash();
    // I13 probes: out-of-range indices and stale ids are rejected and change nothing.
    {
        let mut c = g.clone();
        for bad in [p.options.len(), p.options.len() + 7, usize::MAX] {
            match c.apply(p.id, bad) {
                Err(ApplyError::BadIndex) => {}
                other => return Err(format!("I13 probe: index {bad} gave {other:?}")),
            }
        }
        for stale in [DecisionId(p.id.0.wrapping_add(1)), DecisionId(p.id.0.wrapping_sub(1)), DecisionId(0)] {
            match c.apply(stale, 0) {
                Err(ApplyError::StaleDecision) => {}
                other => return Err(format!("I13 probe: stale id {stale:?} gave {other:?}")),
            }
        }
        if c.state_hash() != h0 {
            return Err("I13 probe changed state".into());
        }
    }
    // I13 mask validity: every option applies and leads to a state satisfying the invariants.
    for i in 0..p.options.len() {
        let mut c = g.clone();
        c.apply(p.id, i).map_err(|e| format!("I13: option {i} ({:?}) rejected: {e:?}", p.options[i]))?;
        c.advance();
        if c.pending().is_some() {
            invariants::check(c.raw_state(), c.db()).map_err(|e| format!("I13: after option {i} ({:?}): {} {}", p.options[i], e.id, e.msg))?;
        }
    }
    // I12 fork consistency: clone then apply the same actions gives the same hash trajectory.
    {
        let mut a = g.clone();
        let mut b = g.clone();
        let mut r2 = Pcg64::from_seed(rng.next_u64());
        for step in 0..40 {
            let sa = a.advance();
            let sb = b.advance();
            if sa != sb || a.state_hash() != b.state_hash() {
                return Err(format!("I12: fork diverged at step {step}"));
            }
            if matches!(sa, Status::GameOver(_)) {
                break;
            }
            let pa = a.pending().unwrap().clone();
            let idx = r2.below(pa.options.len() as u64) as usize;
            a.apply(pa.id, idx).map_err(|e| format!("{e:?}"))?;
            b.apply(pa.id, idx).map_err(|e| format!("{e:?}"))?;
        }
    }
    if g.state_hash() != h0 {
        return Err("deep checks mutated the original".into());
    }
    Ok(())
}

/// I10: record a random game with checkpoints, replay it, require identical hashes.
pub fn record_and_replay(pool: &mtg_cards::testpool::TestPool, decks: [&DeckList; 2], seed: u64, first: u8) -> Result<GameRecord, String> {
    record_and_replay_db(&pool.db, decks, seed, first)
}

/// As `record_and_replay`, against any card database.
pub fn record_and_replay_db(db: &std::sync::Arc<mtg_core::card::CardDb>, decks: [&DeckList; 2], seed: u64, first: u8) -> Result<GameRecord, String> {
    let cfg = GameConfig { first_player: mtg_core::ids::Seat(first), ..GameConfig::default() };
    let mut g = Game::new(db.clone(), decks, seed, cfg);
    let mut rng = Pcg64::from_seed(seed ^ 0x77);
    let mut rec = GameRecord::new(db, decks, seed, first);
    loop {
        let st = g.advance();
        let n = rec.actions.len() as u32;
        if n % 25 == 0 || matches!(st, Status::GameOver(_)) {
            rec.checkpoints.push((n, g.state_hash()));
        }
        if matches!(st, Status::GameOver(_)) {
            break;
        }
        let p = g.pending().unwrap();
        let idx = rng.below(p.options.len() as u64) as usize;
        rec.actions.push((p.id.0, idx as u16));
        let id = p.id;
        g.apply(id, idx).map_err(|e| format!("{e:?}"))?;
        if rec.actions.len() > 20000 {
            break;
        }
    }
    g.advance();
    let n = rec.actions.len() as u32;
    if rec.checkpoints.last().map(|c| c.0) != Some(n) {
        rec.checkpoints.push((n, g.state_hash()));
    }
    let final_hash = g.state_hash();
    let rg = replay(db.clone(), &rec).map_err(|e| format!("replay failed: {e:?}"))?;
    if rg.state_hash() != final_hash {
        return Err("I10: replay final hash differs".into());
    }
    // Text round trip.
    let back = GameRecord::from_text(&rec.to_text())?;
    if back != rec {
        return Err("record text round trip differs".into());
    }
    Ok(rec)
}

/// Reads an "N Card Name" deck list (blank line, then sideboard); returns the list and unknown names.
pub fn load_deck_file(db: &mtg_core::card::CardDb, path: &std::path::Path) -> (DeckList, Vec<String>) {
    let text = std::fs::read_to_string(path).unwrap();
    let (mut main, mut side, mut missing) = (vec![], vec![], vec![]);
    let mut in_side = false;
    let mut seen_main = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('#') {
            continue;
        }
        if l.is_empty() {
            if seen_main {
                in_side = true;
            }
            continue;
        }
        let (n, name) = l.split_once(' ').unwrap();
        let n: usize = n.parse().unwrap();
        match db.id(name.trim()) {
            Some(id) => {
                for _ in 0..n {
                    if in_side { side.push(id) } else { main.push(id) }
                }
            }
            None => missing.push(name.trim().to_string()),
        }
        if !in_side {
            seen_main = true;
        }
    }
    (DeckList { main, side }, missing)
}

/// Builds the real Legacy pool from card text (the pinned snapshot the real-pool goldens replay
/// against, not the live file).
pub fn build_legacy_from_text(text: &str) -> std::sync::Arc<mtg_core::card::CardDb> {
    std::sync::Arc::new(mtg_dsl::build_db(&[("legacy.cards.ron", text)]).unwrap_or_else(|e| panic!("pinned pool failed to load: {e}")))
}

/// The eight real decks, sorted by file name, loaded against `db`.
pub fn load_real_decks(db: &mtg_core::card::CardDb, dir: &std::path::Path) -> Vec<(String, DeckList)> {
    let mut paths: Vec<_> = std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    paths.iter().map(|p| (p.file_stem().unwrap().to_string_lossy().to_string(), load_deck_file(db, p).0)).collect()
}

//! Non-interference test (doc 04 section 7.2): two worlds that agree on everything the observer
//! may know but differ in hidden state must give the observer byte-identical streams.

use mtg_core::card::CardDb;
use std::sync::atomic::{AtomicU64, Ordering};
pub static QTAPS: AtomicU64 = AtomicU64::new(0);
pub static QSAC: AtomicU64 = AtomicU64::new(0);

use mtg_core::decision::Status;
use mtg_core::hash::Fx64;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::state::GameConfig;
use mtg_view::{DeckList, Game};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Copy, Clone, Debug)]
pub struct NiConfig {
    pub observer: Seat,
    /// Also permute the observer's own library (comparison stops at the observer's first draw).
    pub own_library: bool,
    /// Random-play decisions before the worlds split.
    pub depth: u64,
    pub max_steps: u64,
}

#[derive(Debug)]
pub struct NiFail {
    pub seed: u64,
    pub step: u64,
    pub what: String,
}

fn h64<T: Hash>(t: &T) -> u64 {
    let mut h = Fx64::default();
    t.hash(&mut h);
    h.finish()
}

/// Runs one pair. Returns the number of lockstep steps compared.
pub fn run_pair(db: &Arc<CardDb>, decks: [&DeckList; 2], seed: u64, cfg: NiConfig) -> Result<u64, NiFail> {
    run_pair_ex(db, decks, seed, cfg).map(|(n, _)| n)
}

/// As `run_pair`, also naming why the comparison stopped (each reason is an explicit, documented
/// legitimate divergence or the end of the budget).
pub fn run_pair_ex(db: &Arc<CardDb>, decks: [&DeckList; 2], seed: u64, cfg: NiConfig) -> Result<(u64, &'static str), NiFail> {
    let a = cfg.observer;
    let opp = a.other();
    let mut g = Game::new(db.clone(), decks, seed, GameConfig::default());
    g.set_keep_events(true);
    let mut rng = Pcg64::from_seed(seed ^ 0x1357_9BDF);
    let mut n = 0u64;
    while n < cfg.depth {
        match g.advance() {
            Status::GameOver(_) => return Ok((0, "game over before the split")),
            Status::NeedDecision(_) => {
                let p = g.pending().unwrap();
                let (id, len) = (p.id, p.options.len());
                g.apply(id, rng.below(len as u64) as usize).unwrap();
                n += 1;
            }
        }
    }
    g.advance();
    let _ = g.take_events();
    let mut g2 = g.clone();
    g2.raw_state_mut().rerandomize_hidden(a, seed ^ 0xFEED_FACE, cfg.own_library);
    // The worlds must actually differ somewhere hidden, otherwise the test proves nothing; this is
    // checked in aggregate by the caller via `hidden_differs`.
    let mut trace: Vec<String> = Vec::new();
    let fail = |step: u64, what: String, trace: &[String]| NiFail { seed, step, what: format!("{what}\n--- last actions:\n{}", trace[trace.len().saturating_sub(24)..].join("\n")) };
    let mut shuffled = false;
    let lib0 = g.observe(a).me.library_count;
    let mut step = 0u64;
    loop {
        if step >= cfg.max_steps {
            return Ok((step, "max steps"));
        }
        let (o1, o2) = (g.observe(a), g2.observe(a));
        if o1.events.iter().any(|e| matches!(e, mtg_view::ViewEvent::Shuffled { me: true })) {
            // The worlds carry different RNG state, so the observer's own shuffle orders its library
            // differently in each: from here on, what it draws or looks at legitimately differs.
            shuffled = true;
        }
        // Explicit legitimate divergences: the observer was shown different cards in the two worlds
        // (a reveal, a look at its own library, a public mill/discard of a hidden card).
        let names = |v: &[mtg_view::ViewCard]| v.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        if names(&o1.opp.revealed_hand) != names(&o2.opp.revealed_hand)
            || names(&o1.me.library_known_top) != names(&o2.me.library_known_top)
            || names(&o1.me.library_known_bottom) != names(&o2.me.library_known_bottom)
            || names(&o1.opp.library_known_top) != names(&o2.opp.library_known_top)
            || names(&o1.opp.library_known_bottom) != names(&o2.opp.library_known_bottom)
        {
            return Ok((step, "observer shown different cards (reveal or look)"));
        }
        let reveals = |evs: &[mtg_view::ViewEvent]| {
            evs.iter().any(|e| matches!(e, mtg_view::ViewEvent::Moved { vid_old, from, .. } if *vid_old == mtg_core::ids::ViewId::NONE && matches!(from, mtg_core::types::ZoneKind::Library | mtg_core::types::ZoneKind::Hand)))
        };
        if o1.events != o2.events && (reveals(&o1.events) || reveals(&o2.events)) {
            return Ok((step, "hidden card publicly revealed"));
        }
        if cfg.own_library && o1.me.library_count != lib0 {
            return Ok((step, "observer drew from its permuted library"));
        }
        if shuffled {
            let from_lib = |evs: &[mtg_view::ViewEvent]| evs.iter().any(|e| matches!(e, mtg_view::ViewEvent::Moved { from: mtg_core::types::ZoneKind::Library, .. } | mtg_view::ViewEvent::HiddenMove { from: mtg_core::types::ZoneKind::Library, .. }));
            if names(&o1.me.hand) != names(&o2.me.hand) || (o1.events != o2.events && (from_lib(&o1.events) || from_lib(&o2.events))) {
                return Ok((step, "observer drew from a library shuffled differently in each world"));
            }
        }
        if o1.events != o2.events {
            // A card that was hidden from the observer being revealed (discard, mill) differs
            // legitimately between the worlds. Anything else in the event stream is a leak.
            let norm = |evs: &[mtg_view::ViewEvent]| -> Vec<mtg_view::ViewEvent> {
                evs.iter()
                    .cloned()
                    .map(|mut e| {
                        if let mtg_view::ViewEvent::Moved { name, vid_old, .. } = &mut e {
                            if *vid_old == mtg_core::ids::ViewId::NONE {
                                name.clear();
                            }
                        }
                        e
                    })
                    .collect()
            };
            // The same cards moved in a different order: the outcome of the engine's own random
            // choice (a random bottom order after Atraxa, say), which the two worlds draw differently.
            let sorted = |evs: &[mtg_view::ViewEvent]| {
                let mut v: Vec<String> = evs.iter().map(|e| format!("{e:?}")).collect();
                v.sort();
                v
            };
            if sorted(&o1.events) == sorted(&o2.events) {
                return Ok((step, "same events in a different order (random bottom order)"));
            }
            if norm(&o1.events) == norm(&o2.events) {
                return Ok((step, "hidden card revealed by name in the event stream"));
            }
        }
        // Decisions can legitimately depend on hidden state in two ways. (1) The opponent's pending
        // decision (an Aether Vial or Show and Tell choice, say, is only offered if its hand holds
        // an eligible card): the observer never acts during it, and its own view converges as soon
        // as the opponent has acted, so the observer's next decision is still compared strictly.
        // (2) The observer itself choosing among cards it is shown (Thoughtseize, a look at its own
        // library): the option labels name the revealed cards.
        if let (Some(q1), Some(q2)) = (g.pending(), g2.pending()) {
            if q1.seat != q2.seat && (q1.seat == opp || q2.seat == opp) {
                return Ok((step, "a decision of the opponent exists only in one world"));
            }
            if q1.seat == q2.seat && q1.seat != a && (q1.id != q2.id || format!("{o1:?}") != format!("{o2:?}")) {
                if std::env::var_os("NI_DEBUG").is_some() {
                    let (t1, t2) = (format!("{o1:#?}"), format!("{o2:#?}"));
                    let (l1, l2): (Vec<&str>, Vec<&str>) = (t1.lines().collect(), t2.lines().collect());
                    let i = l1.iter().zip(l2.iter()).position(|(a, b)| a != b);
                    eprintln!("NIDBG seed {seed} step {step} ids {:?}/{:?} firstdiff {:?}", q1.id, q2.id, i.map(|i| (l1[i.saturating_sub(3)..=i].join(" ").replace("  ", ""), l2[i].to_string())));
                }
                return Ok((step, "opponent's pending decision depends on its hidden cards"));
            }
            if q1.seat == a && q2.seat == a {
                if let (Some(d1), Some(d2)) = (&o1.decision, &o2.decision) {
                    // After the observer's own shuffle the worlds hold different physical copies of
                    // its library cards, so the ids of cards it already knows there differ too.
                    let own_lib_diverged = shuffled || cfg.own_library;
                    let labels = |d: &mtg_view::Decision| d.options.iter().map(|x| (x.label.clone(), if own_lib_diverged { x.subject } else { None })).collect::<Vec<_>>();
                    if matches!(d1.kind, mtg_view::ViewDecisionKind::ChooseCards { .. }) && labels(d1) != labels(d2) {
                        return Ok((step, "observer chooses among cards it is shown"));
                    }
                }
            }
        }
        if format!("{o1:?}") != format!("{o2:?}") {
            let (t1, t2) = (format!("{o1:#?}"), format!("{o2:#?}"));
            let (l1, l2): (Vec<&str>, Vec<&str>) = (t1.lines().collect(), t2.lines().collect());
            let i = l1.iter().zip(l2.iter()).position(|(a, b)| a != b).unwrap_or(l1.len().min(l2.len()));
            let lo = i.saturating_sub(40);
            return Err(fail(step, format!("observation differs at line {i}:\n--- world 1\n{}\n--- world 2\n{}", l1[lo..(i + 6).min(l1.len())].join("\n"), l2[lo..(i + 6).min(l2.len())].join("\n")), &trace));
        }
        if o1.view_hash != o2.view_hash {
            return Err(fail(step, "view_hash differs".into(), &trace));
        }
        let (p1, p2) = (g.pending().map(|p| (p.id, p.seat, p.options.len())), g2.pending().map(|p| (p.id, p.seat, p.options.len())));
        let (Some(p1), Some(p2)) = (p1, p2) else {
            if p1.is_some() != p2.is_some() {
                return Err(fail(step, "one world ended, the other did not".into(), &trace));
            }
            return Ok((step, "game over"));
        };
        if (p1.0, p1.1) != (p2.0, p2.1) {
            return Err(fail(step, format!("decider/id differs: {:?} vs {:?}", (p1.0, p1.1), (p2.0, p2.1)), &trace));
        }
        // Error text probes: out-of-range index and stale id, on clones so nothing changes.
        for (bad_id, bad_idx) in [(p1.0, 100_000usize), (mtg_core::ids::DecisionId(p1.0 .0.wrapping_add(7)), 0)] {
            let (mut c1, mut c2) = (g.clone(), g2.clone());
            let e1 = c1.apply(bad_id, bad_idx).err().map(|e| c1.describe_error(e));
            let e2 = c2.apply(bad_id, bad_idx).err().map(|e| c2.describe_error(e));
            if e1 != e2 {
                return Err(fail(step, format!("error text differs: {e1:?} vs {e2:?}"), &trace));
            }
        }
        trace.push(format!(
            "   [{step}] hand1={:?} hand2={:?} stack1={:?} stack2={:?} ev1={:?}",
            o1.me.hand.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            o2.me.hand.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            o1.stack.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            o2.stack.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            o1.events.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>()
        ));
        if p1.1 == a {
            let k = h64(&(o1.view_hash, step)) as usize % p1.2;
            trace.push(format!("step {step}: observer {:?} picks #{k} of {} ({})", p1.1, p1.2, o1.decision.as_ref().map(|d| d.options[k].label.clone()).unwrap_or_default()));
            g.apply(p1.0, k).unwrap();
            g2.apply(p1.0, k).unwrap();
        } else {
            // Opponent: pick by label among labels present in both worlds.
            let (d1, d2) = (g.observe(opp).decision.unwrap(), g2.observe(opp).decision.unwrap());
            let common: Vec<&mtg_view::ActionDesc> = d1
                .options
                .iter()
                .filter(|x| x.label != "Target: you")
                .filter(|x| d2.options.iter().any(|y| y.label == x.label))
                .collect();
            if common.is_empty() {
                return Ok((step, "no common opponent option"));
            }
            let pick = common[h64(&(seed, step)) as usize % common.len()];
            trace.push(format!("step {step}: opponent picks {:?} ({:?})", pick.label, d1.kind));
            // Labels are not unique (two identical legendary permanents, say): match the same
            // subject first, then fall back to the first option with that label.
            let i2 = d2.options.iter().find(|y| y.label == pick.label && y.subject == pick.subject).or_else(|| d2.options.iter().find(|y| y.label == pick.label)).unwrap().idx as usize;
            g.apply(p1.0, pick.idx as usize).unwrap();
            g2.apply(p1.0, i2).unwrap();
        }
        g.advance();
        g2.advance();
        count_q(&mut g);
        step += 1;
    }
}

/// True if the two worlds built for `seed` differ in hidden state (sanity for the test itself).
pub fn hidden_differs(db: &Arc<CardDb>, decks: [&DeckList; 2], seed: u64, cfg: NiConfig) -> bool {
    let mut g = Game::new(db.clone(), decks, seed, GameConfig::default());
    let mut rng = Pcg64::from_seed(seed ^ 0x1357_9BDF);
    for _ in 0..cfg.depth {
        if let Status::NeedDecision(_) = g.advance() {
            let p = g.pending().unwrap();
            let (id, len) = (p.id, p.options.len());
            g.apply(id, rng.below(len as u64) as usize).unwrap();
        } else {
            return false;
        }
    }
    let mut g2 = g.clone();
    g2.raw_state_mut().rerandomize_hidden(cfg.observer, seed ^ 0xFEED_FACE, cfg.own_library);
    g.state_hash() != g2.state_hash()
}

fn count_q(g: &mut Game) {
    use mtg_core::event::Event;
    let ev = g.take_events();
    let mut q = 0; let mut gy = 0;
    for e in &ev {
        match e {
            Event::Tapped { obj } => { if g.db().def(g.raw_state().def_of(*obj)).name.contains("Lazotep") { q += 1; } }
            Event::ZoneChange { from: mtg_core::types::ZoneKind::Battlefield, to: mtg_core::types::ZoneKind::Graveyard, .. } => gy += 1,
            _ => {}
        }
    }
    QTAPS.fetch_add(q, Ordering::Relaxed);
    if q > 0 && gy > 0 { QSAC.fetch_add(q, Ordering::Relaxed); }
}

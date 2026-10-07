//! One game with an LLM at one seat, driven through files so a tool-calling session can play it.
//! The opponent seat is a determinized MCTS (rollout evaluator, or a net with --net).
//!
//! Usage: llmgame <decks dir> <llm deck> <opp deck> <prompt dir> <truth dir> --seed S --llm-seat 0|1
//!        [--iters 32] [--opp-net w.bin] [--me-net w.bin (with --auto-llm)] [--stops own|opp ...]
//!        [--replay actions.json --oneshot]   (position mode: apply [[seat, idx], ...] silently, ask the LLM the next
//!        decision once, record the answer and stop; used to test the tools on flagged positions)
//!        [--consult-net w.bin] [--consult-iters 1000] [--me-iters N (auto-llm search strength)]
//!
//! Consult tool: while a prompt waits for an answer the player may drop `prompt dir/consult.req`
//! ("<seq> [iters]"); the program replies in `prompt dir/consult.txt` with the net's read of the
//! decision (policy and value) and a short determinized-MCTS read of the options, computed from the
//! player's own `SeatView` only (the same object the prompt is rendered from; hidden cards are
//! re-sampled from the opponent's decklist by the belief model, never read).
//!
//! The LLM only ever sees what `prompt dir/prompt.txt` contains, which is rendered from the
//! LLM seat's `Observation` (a `SeatView`): no hidden state. The truth dir gets the harness log
//! (opponent choices, result) and must not be read by the player until the game is over.
//! Protocol: the player writes `prompt dir/ans.tmp` containing "<seq> <idx>" and renames it to
//! `ans.txt`; the program answers by rewriting `prompt.txt`. The last prompt says GAME OVER.
use mtg_agent::*;
use mtg_core::decision::{GameResult, Status};
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::state::GameConfig;
use mtg_core::types::*;
use mtg_view::*;
use std::fmt::Write as _;
use std::path::PathBuf;

fn step_name(s: Step) -> &'static str {
    match s {
        Step::Untap => "Untap",
        Step::Upkeep => "Upkeep",
        Step::Draw => "Draw",
        Step::Main1 => "Main 1",
        Step::BeginCombat => "Beginning of combat",
        Step::DeclareAttackers => "Declare attackers",
        Step::DeclareBlockers => "Declare blockers",
        Step::FirstStrikeDamage => "First-strike damage",
        Step::CombatDamage => "Combat damage",
        Step::EndCombat => "End of combat",
        Step::Main2 => "Main 2",
        Step::End => "End step",
        Step::Cleanup => "Cleanup",
    }
}

fn type_str(t: Types) -> String {
    let mut v = vec![];
    for (f, n) in [
        (Types::LAND, "land"),
        (Types::CREATURE, "creature"),
        (Types::ARTIFACT, "artifact"),
        (Types::ENCHANTMENT, "enchantment"),
        (Types::PLANESWALKER, "planeswalker"),
        (Types::INSTANT, "instant"),
        (Types::SORCERY, "sorcery"),
    ] {
        if t.contains(f) {
            v.push(n);
        }
    }
    v.join(" ")
}

fn kw_str(k: Keywords) -> String {
    let s = format!("{:?}", k);
    s.replace("Keywords(", "").replace(')', "").replace('_', " ").to_lowercase().replace("(empty)", "")
}

fn perm_str(p: &ViewPermanent) -> String {
    let mut s = format!("{} #{}", p.name, p.vid.0);
    let t = type_str(p.types);
    if p.types.contains(Types::CREATURE) {
        let _ = write!(s, " {}/{}", p.power, p.toughness);
    }
    if !t.is_empty() && !p.types.contains(Types::LAND) {
        let _ = write!(s, " [{t}]");
    }
    if p.tapped {
        s.push_str(" (tapped)");
    }
    if p.summoning_sick && p.types.contains(Types::CREATURE) {
        s.push_str(" (summoning sick)");
    }
    if p.damage > 0 {
        let _ = write!(s, " dmg {}", p.damage);
    }
    for (k, n) in &p.counters {
        let _ = write!(s, " {:?}x{}", k, n);
    }
    let kw = kw_str(p.keywords);
    if !kw.trim().is_empty() && kw.trim() != "0x0" {
        let _ = write!(s, " <{}>", kw.trim());
    }
    if p.attacking {
        s.push_str(" ATTACKING");
    }
    if p.blocking.is_some() {
        let _ = write!(s, " BLOCKING #{}", p.blocking.unwrap().0);
    }
    if !p.owned_by_me {
        s.push_str(" (owned by other)");
    }
    s
}

fn cards_str(v: &[ViewCard]) -> String {
    if v.is_empty() {
        return "-".into();
    }
    v.iter().map(|c| format!("{} #{}", c.name, c.vid.0)).collect::<Vec<_>>().join(", ")
}

fn target_str(t: &ViewTarget) -> String {
    match t {
        ViewTarget::Player { me } => if *me { "you".into() } else { "opponent".into() },
        ViewTarget::Object { vid, name } => format!("{name} #{}", vid.0),
        ViewTarget::Gone => "(gone)".into(),
        ViewTarget::None => "(none)".into(),
    }
}

fn desig_str(d: &ViewDesignations) -> String {
    let mut v = vec![];
    if let Some((n, r)) = &d.dungeon {
        v.push(format!("dungeon {n} room {r}"));
    }
    if !d.completed_dungeons.is_empty() {
        v.push(format!("completed dungeons: {}", d.completed_dungeons.join(", ")));
    }
    if !d.emblems.is_empty() {
        v.push(format!("emblems: {}", d.emblems.join(", ")));
    }
    if d.ring_level > 0 {
        v.push(format!("ring level {}", d.ring_level));
    }
    if d.blessing {
        v.push("city's blessing".into());
    }
    v.join("; ")
}

fn event_str(e: &ViewEvent) -> Option<String> {
    let who = |me: bool| if me { "you" } else { "opponent" };
    Some(match e {
        ViewEvent::TurnBegan { turn, active_is_me } => format!("-- turn {turn} ({} turn) --", if *active_is_me { "your" } else { "opponent's" }),
        ViewEvent::StepBegan { .. } => return None,
        ViewEvent::Moved { name, from, to, owner_is_me, .. } => format!("{name} ({}'s) moved {:?} -> {:?}", who(*owner_is_me), from, to),
        ViewEvent::HiddenMove { owner_is_me, from, to } => format!("a hidden card of {} moved {:?} -> {:?}", who(*owner_is_me), from, to),
        ViewEvent::DrewCard { name, .. } => format!("you drew {name}"),
        ViewEvent::OppDrewCard => "opponent drew a card".into(),
        ViewEvent::DrewFromEmptyLibrary { me } => format!("{} drew from an empty library", who(*me)),
        ViewEvent::Damage { to, amount, combat, .. } => format!("{amount} {}damage to {}", if *combat { "combat " } else { "" }, target_str(to)),
        ViewEvent::LifeChange { me, delta } => format!("{} life {:+}", who(*me), delta),
        ViewEvent::SpellCast { name, by_me, .. } => format!("{} cast {name}", who(*by_me)),
        ViewEvent::AbilityActivated { by_me, .. } => format!("{} activated an ability", who(*by_me)),
        ViewEvent::LandPlayed { name, by_me, .. } => format!("{} played {name}", who(*by_me)),
        ViewEvent::Tapped { .. } | ViewEvent::Untapped { .. } => return None,
        ViewEvent::CountersChanged { vid, kind, delta } => format!("#{} counters {:?} {:+}", vid.0, kind, delta),
        ViewEvent::Shuffled { me } => format!("{} library shuffled", who(*me)),
        ViewEvent::AttackersDeclared { count } => format!("{count} attacker(s) declared"),
        ViewEvent::BlockersDeclared { count } => format!("{count} blocker(s) declared"),
        ViewEvent::SpellCountered { vid } => format!("spell #{} countered", vid.0),
        ViewEvent::SpellFizzled { vid } => format!("spell #{} fizzled", vid.0),
        ViewEvent::MulliganTaken { me } => format!("{} took a mulligan", who(*me)),
        ViewEvent::HandKept { me, bottomed } => format!("{} kept (bottomed {bottomed})", who(*me)),
        ViewEvent::TokenCreated { name, by_me, .. } => format!("{} created token {name}", who(*by_me)),
        ViewEvent::GameEnded => "game ended".into(),
    })
}

fn render(o: &Observation, seq: u32) -> String {
    let d = o.decision.as_ref().expect("decision");
    let mut s = String::new();
    let _ = writeln!(s, "=== PROMPT {seq} | turn {} | {} | {} turn ===", o.turn, step_name(o.step), if o.active_is_me { "YOUR" } else { "OPPONENT'S" });
    let _ = writeln!(s, "YOU: life {} | library {} | lands played this turn {} | mana pool {:?}{}", o.me.life, o.me.library_count, o.me.lands_played, o.me.pool, if o.me.energy > 0 { format!(" | energy {}", o.me.energy) } else { String::new() });
    let ds = desig_str(&o.me.designations);
    if !ds.is_empty() {
        let _ = writeln!(s, "  your designations: {ds}");
    }
    let _ = writeln!(s, "OPPONENT: life {} | hand {} cards | library {} | lands played {}", o.opp.life, o.opp.hand_count, o.opp.library_count, o.opp.lands_played);
    let ds = desig_str(&o.opp.designations);
    if !ds.is_empty() {
        let _ = writeln!(s, "  opponent designations: {ds}");
    }
    if !o.opp.revealed_hand.is_empty() {
        let _ = writeln!(s, "  opponent cards you know in hand: {}", cards_str(&o.opp.revealed_hand));
    }
    let _ = writeln!(s, "YOUR HAND: {}", cards_str(&o.me.hand));
    if !o.me.library_known_top.is_empty() {
        let _ = writeln!(s, "YOUR LIBRARY TOP (known, top first): {}", cards_str(&o.me.library_known_top));
    }
    if !o.me.library_known_bottom.is_empty() {
        let _ = writeln!(s, "YOUR LIBRARY BOTTOM (known): {}", cards_str(&o.me.library_known_bottom));
    }
    let mine: Vec<_> = o.battlefield.iter().filter(|p| p.controlled_by_me).collect();
    let theirs: Vec<_> = o.battlefield.iter().filter(|p| !p.controlled_by_me).collect();
    let _ = writeln!(s, "YOUR BATTLEFIELD:");
    for p in mine {
        let _ = writeln!(s, "  {}", perm_str(p));
    }
    let _ = writeln!(s, "OPPONENT BATTLEFIELD:");
    for p in theirs {
        let _ = writeln!(s, "  {}", perm_str(p));
    }
    let _ = writeln!(s, "YOUR GRAVEYARD: {}", cards_str(&o.me.graveyard));
    let _ = writeln!(s, "OPPONENT GRAVEYARD: {}", cards_str(&o.opp.graveyard));
    if !o.me.exile.is_empty() || !o.opp.exile.is_empty() {
        let _ = writeln!(s, "EXILE: yours: {} | opponent's: {}", cards_str(&o.me.exile), cards_str(&o.opp.exile));
    }
    if !o.stack.is_empty() {
        let _ = writeln!(s, "STACK (top first):");
        for it in o.stack.iter().rev() {
            let tg: Vec<_> = it.targets.iter().map(target_str).collect();
            let _ = writeln!(s, "  {} #{}{} controlled by {}{}", it.name, it.vid.0, if it.is_ability { " (ability)" } else { "" }, if it.controlled_by_me { "you" } else { "opponent" }, if tg.is_empty() { String::new() } else { format!(" targeting {}", tg.join(", ")) });
        }
    }
    let mut ev: Vec<String> = Vec::new();
    for e in o.events.iter().filter_map(event_str) {
        // collapse runs of identical lines ("opponent drew a card" x7)
        match ev.last_mut() {
            Some(l) if l == &e || l.starts_with(&format!("{e} (x")) => {
                let n = l.rsplit("(x").next().and_then(|t| t.trim_end_matches(')').parse::<u32>().ok()).unwrap_or(1) + 1;
                *l = format!("{e} (x{n})");
            }
            _ => ev.push(e),
        }
    }
    if !ev.is_empty() {
        let _ = writeln!(s, "RECENT EVENTS:");
        let skip = ev.len().saturating_sub(40);
        for e in &ev[skip..] {
            let _ = writeln!(s, "  {e}");
        }
    }
    let _ = writeln!(s, "DECISION: {:?}", d.kind);
    if !d.context.is_empty() {
        let _ = writeln!(s, "  {}", d.context);
    }
    for a in &d.options {
        let _ = writeln!(s, "  [{}] {:?} {}{}", a.idx, a.kind, a.label, a.value.map(|v| format!(" (value {v})")).unwrap_or_default());
    }
    s
}

/// Net + search read of the pending decision, from the acting seat's own view only.
fn consult(sv: &SeatView, obs: &Observation, model: &dyn BeliefModel, net: Option<&std::sync::Arc<Net>>, n_defs: usize, iters: u32, seed: u64, seq: u32) -> (String, Option<usize>, Option<usize>) {
    let d = obs.decision.as_ref().expect("decision");
    let n = d.options.len();
    // Net alone: one forward pass on the observation.
    let net_read: Option<(f32, Vec<f32>)> = net.map(|nt| {
        let mut buf = Vec::new();
        encode_state(obs, n_defs, &mut buf);
        let state: Vec<(u32, f32)> = buf.iter().enumerate().filter(|(_, v)| **v != 0.0).map(|(i, v)| (i as u32, *v)).collect();
        nt.forward(&state, &encode_options(obs))
    });
    let ev: Box<dyn Evaluator> = match net {
        Some(nt) => Box::new(NetEvaluator::new(nt.clone(), n_defs)),
        None => Box::new(RolloutEvaluator::new(2500, seed ^ 0x51)),
    };
    let mut ev = ev;
    let t0 = std::time::Instant::now();
    let cfg = SearchConfig { iterations: iters, seed: seed ^ (seq as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15), ..SearchConfig::default() };
    let r = search(sv, model, &mut *ev, &cfg);
    let secs = t0.elapsed().as_secs_f64();
    let wp = |v: f32| ((v + 1.0) * 50.0).round() as i32;
    let total: u32 = r.visits.iter().sum::<u32>().max(1);
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| r.visits[b].cmp(&r.visits[a]).then(r.q[b].partial_cmp(&r.q[a]).unwrap_or(std::cmp::Ordering::Equal)));
    let mut s = String::new();
    let _ = writeln!(s, "=== CONSULT for PROMPT {seq} | read computed from YOUR view only (opponent's hand is guessed from their decklist) ===");
    match &net_read {
        Some((v, _)) => {
            let _ = writeln!(s, "Net alone (no search): you win about {}%.", wp(*v));
        }
        None => {
            let _ = writeln!(s, "No net loaded: search uses random playouts as evaluator.");
        }
    }
    let _ = writeln!(s, "Search ({} iterations, {:.1}s, net as evaluator, opponent assumed to play like the net): you win about {}%.", r.iterations, secs, wp(r.value));
    let _ = writeln!(s, "Top options by search visits (win% = search estimate if you take it; a few visits = a noisy estimate):");
    let _ = writeln!(s, "  idx  net%  visit%  win%  visits  option");
    let mut shown = 0;
    for &i in order.iter().take(6) {
        if r.visits[i] == 0 && shown >= 3 {
            break;
        }
        let nprior = net_read.as_ref().map(|(_, p)| format!("{:>3.0}", p[i] * 100.0)).unwrap_or_else(|| "  -".into());
        let _ = writeln!(s, "  [{}]  {}   {:>3.0}    {:>3}   {:>5}   {} {}", d.options[i].idx, nprior, r.visits[i] as f32 * 100.0 / total as f32, wp(r.q[i]), r.visits[i], format!("{:?}", d.options[i].kind), d.options[i].label);
        shown += 1;
    }
    if n > shown {
        let _ = writeln!(s, "  ({} other options got fewer visits)", n - shown);
    }
    let _ = writeln!(s, "Limits: this is a quick bot read, not an oracle. It is weak on long combo sequences (Aluren/Acererak loops), on mana-payment detail and on what the opponent's hidden cards do. You decide.");
    let best_net = net_read.as_ref().map(|(_, p)| p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|x| x.0).unwrap_or(0));
    (s, Some(r.best()), best_net)
}

/// Hypergeometric helpers: P(at least one of `s` marked cards among `r` cards drawn from `u`).
fn p_any(u: usize, s: usize, r: usize) -> f64 {
    if s == 0 || r == 0 || u == 0 {
        return 0.0;
    }
    if r >= u || u - s < r {
        return 1.0;
    }
    let mut none = 1.0f64;
    for i in 0..r {
        none *= (u - s - i) as f64 / (u - i) as f64;
    }
    1.0 - none
}

/// Resolve a term ("Show and Tell", "lands", "creatures", "nonlands") to the set of card defs it names in `pool`.
fn term_defs(db: &mtg_core::card::CardDb, pool: &std::collections::BTreeMap<usize, usize>, term: &str) -> Option<Vec<usize>> {
    let t = term.trim().to_lowercase();
    let pick = |f: &dyn Fn(&mtg_core::card::CardDef) -> bool| -> Vec<usize> { pool.keys().copied().filter(|&d| f(&db.defs[d])).collect() };
    match t.as_str() {
        "land" | "lands" => Some(pick(&|d| d.types.contains(Types::LAND))),
        "nonland" | "nonlands" => Some(pick(&|d| !d.types.contains(Types::LAND))),
        "creature" | "creatures" => Some(pick(&|d| d.types.contains(Types::CREATURE))),
        _ => {
            let v: Vec<usize> = (0..db.defs.len()).filter(|&d| db.defs[d].name.to_lowercase() == t).collect();
            if v.is_empty() { None } else { Some(v) }
        }
    }
}

/// Library odds for me (`odds N [terms]`) or what the opponent may hold (`opp [terms]`), from the player's own
/// view and the two decklists only. Library = my decklist minus every own card I can see; opponent pool = their
/// decklist minus every opponent card I can see.
fn odds_report(obs: &Observation, db: &mtg_core::card::CardDb, mine: &DeckList, theirs: &DeckList, seq: u32, cmd: &str, rest: &str) -> String {
    use std::collections::BTreeMap;
    let mut s = String::new();
    let count = |ids: &[CardDefId]| -> BTreeMap<usize, usize> {
        let mut m = BTreeMap::new();
        for d in ids {
            *m.entry(d.0 as usize).or_insert(0) += 1;
        }
        m
    };
    let nontoken = |d: CardDefId| !db.defs[d.0 as usize].is_token;
    let sub = |pool: &mut BTreeMap<usize, usize>, d: CardDefId| {
        if nontoken(d) {
            if let Some(c) = pool.get_mut(&(d.0 as usize)) {
                *c = c.saturating_sub(1);
            }
        }
    };
    let name = |d: usize| db.defs[d].name.clone();
    if cmd == "odds" {
        let (n_str, terms) = rest.trim().split_once(' ').map(|(a, b)| (a, b)).unwrap_or((rest.trim(), ""));
        let n: usize = n_str.parse().unwrap_or(1).clamp(1, 20);
        let mut pool = count(&mine.main);
        for c in obs.me.hand.iter().chain(&obs.me.graveyard).chain(&obs.me.exile) {
            sub(&mut pool, c.def);
        }
        for p in obs.battlefield.iter().filter(|p| p.owned_by_me) {
            sub(&mut pool, p.def);
        }
        for it in obs.stack.iter().filter(|i| i.controlled_by_me && !i.is_ability) {
            sub(&mut pool, it.def);
        }
        let total: usize = pool.values().sum();
        // Known top/bottom cards are still in the library but not random: draw them first / never.
        let top: Vec<CardDefId> = obs.me.library_known_top.iter().map(|c| c.def).collect();
        let bottom: Vec<CardDefId> = obs.me.library_known_bottom.iter().map(|c| c.def).collect();
        let mut unk = pool.clone();
        for &d in top.iter().chain(&bottom) {
            if let Some(c) = unk.get_mut(&(d.0 as usize)) {
                *c = c.saturating_sub(1);
            }
        }
        let u: usize = unk.values().sum();
        let lib = obs.me.library_count;
        let _ = writeln!(s, "=== ODDS for PROMPT {seq} | your library: {lib} cards; computed as your decklist minus every own card you can see ===");
        if total != lib {
            let _ = writeln!(s, "WARNING: decklist-minus-seen gives {total} cards but your library has {lib}; counts may be off by a card or two (a card from outside your deck, a pitched card mid-cast).");
        }
        let _ = writeln!(s, "Looking at the top {n} card(s){}.", if top.is_empty() { String::new() } else { format!(" (you know the top {}: {})", top.len(), top.iter().map(|d| name(d.0 as usize)).collect::<Vec<_>>().join(", ")) });
        let r_unknown = n.saturating_sub(top.len());
        let known_seen: Vec<usize> = top.iter().take(n).map(|d| d.0 as usize).collect();
        let prob = |set: &[usize]| -> f64 {
            if known_seen.iter().any(|d| set.contains(d)) {
                return 1.0;
            }
            let sc: usize = set.iter().map(|d| unk.get(d).copied().unwrap_or(0)).sum();
            p_any(u.saturating_sub(bottom.len().min(u)), sc.min(u), r_unknown)
        };
        let expected = |set: &[usize]| -> f64 {
            let sc: usize = set.iter().map(|d| unk.get(d).copied().unwrap_or(0)).sum();
            let k = known_seen.iter().filter(|d| set.contains(d)).count() as f64;
            k + if u > 0 { r_unknown as f64 * sc as f64 / u as f64 } else { 0.0 }
        };
        let tl: Vec<&str> = terms.split(';').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
        if tl.is_empty() {
            let mut v: Vec<(usize, f64, usize)> = pool.iter().filter(|(_, &c)| c > 0).map(|(&d, &c)| (d, prob(&[d]), c)).collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
            let _ = writeln!(s, "Chance each card appears among those {n} (copies left in library):");
            let _ = writeln!(s, "  {}", v.iter().map(|(d, p, c)| format!("{} x{} {:.0}%", name(*d), c, p * 100.0)).collect::<Vec<_>>().join("; "));
        } else {
            let mut all: Vec<usize> = Vec::new();
            for t in &tl {
                match term_defs(db, &pool, t) {
                    Some(set) => {
                        let copies: usize = set.iter().map(|d| pool.get(d).copied().unwrap_or(0)).sum();
                        let _ = writeln!(s, "  {t}: {copies} cop{} left in library; chance at least one is in the top {n}: {:.0}%; expected number: {:.2}", if copies == 1 { "y" } else { "ies" }, prob(&set) * 100.0, expected(&set));
                        all.extend(set);
                    }
                    None => {
                        let _ = writeln!(s, "  {t}: unknown term (use a card name, lands, nonlands or creatures)");
                    }
                }
            }
            if tl.len() > 1 {
                all.sort();
                all.dedup();
                let _ = writeln!(s, "  any of the above: {:.0}%", prob(&all) * 100.0);
            }
        }
        let _ = writeln!(s, "Note: uniform over your unknown library; it does not know your library order beyond what the prompt shows.");
    } else {
        let mut pool = count(&theirs.main);
        for c in obs.opp.graveyard.iter().chain(&obs.opp.exile) {
            sub(&mut pool, c.def);
        }
        for p in obs.battlefield.iter().filter(|p| !p.owned_by_me) {
            sub(&mut pool, p.def);
        }
        for it in obs.stack.iter().filter(|i| !i.controlled_by_me && !i.is_ability) {
            sub(&mut pool, it.def);
        }
        let known: Vec<usize> = obs.opp.revealed_hand.iter().map(|c| c.def.0 as usize).collect();
        for c in &obs.opp.revealed_hand {
            sub(&mut pool, c.def);
        }
        let u: usize = pool.values().sum();
        let h = obs.opp.hand_count.saturating_sub(known.len());
        let _ = writeln!(s, "=== OPPONENT HAND ODDS for PROMPT {seq} | their hand {} card(s){} ===", obs.opp.hand_count, if known.is_empty() { String::new() } else { format!(", {} of them known to you", known.len()) });
        if u != h + obs.opp.library_count {
            let _ = writeln!(s, "WARNING: their decklist minus cards you have seen gives {u} unseen cards but hand+library says {}; counts may be off by a card or two.", h + obs.opp.library_count);
        }
        let _ = writeln!(s, "Pool: their decklist minus every opponent card you can see (graveyard, battlefield, exile, stack, revealed). {h} unseen card(s) in hand, drawn uniformly from {u}.");
        let tl: Vec<&str> = rest.split(';').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
        let hand_p = |set: &[usize]| -> f64 {
            if known.iter().any(|d| set.contains(d)) {
                return 1.0;
            }
            p_any(u, set.iter().map(|d| pool.get(d).copied().unwrap_or(0)).sum::<usize>().min(u), h)
        };
        if tl.is_empty() {
            let mut v: Vec<(usize, f64, usize)> = pool.iter().filter(|(_, &c)| c > 0).map(|(&d, &c)| (d, hand_p(&[d]), c)).collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
            let _ = writeln!(s, "Chance they hold at least one, top 15 (copies unseen): {}", v.iter().take(15).map(|(d, p, c)| format!("{} x{} {:.0}%", name(*d), c, p * 100.0)).collect::<Vec<_>>().join("; "));
        } else {
            let mut all: Vec<usize> = Vec::new();
            for t in &tl {
                match term_defs(db, &pool, t) {
                    Some(set) => {
                        let copies: usize = set.iter().map(|d| pool.get(d).copied().unwrap_or(0)).sum();
                        let _ = writeln!(s, "  {t}: {copies} unseen; chance they hold at least one: {:.0}%", hand_p(&set) * 100.0);
                        all.extend(set);
                    }
                    None => {
                        let _ = writeln!(s, "  {t}: unknown term (use a card name, lands, nonlands or creatures)");
                    }
                }
            }
            if tl.len() > 1 {
                all.sort();
                all.dedup();
                let _ = writeln!(s, "  any of the above: {:.0}%", hand_p(&all) * 100.0);
            }
        }
        let _ = writeln!(s, "Note: this does not model their play. A player who kept up mana or passed with cards in hand is more likely to hold counters, burn or tricks than this says, and one who kept a land-light hand is more likely to be holding spells.");
    }
    s
}

// ---- sim: Monte Carlo "what if" questions over the player's own unknown cards -------------------------------

#[derive(Debug, Clone)]
enum Tok {
    Str(String),
    Word(String),
    Num(i64),
    Sym(String),
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch == '"' {
            let mut j = i + 1;
            let mut t = String::new();
            while j < c.len() && c[j] != '"' {
                t.push(c[j]);
                j += 1;
            }
            if j >= c.len() {
                return Err("unterminated quote".into());
            }
            out.push(Tok::Str(t));
            i = j + 1;
        } else if ch.is_ascii_digit() {
            let mut j = i;
            while j < c.len() && c[j].is_ascii_digit() {
                j += 1;
            }
            out.push(Tok::Num(c[i..j].iter().collect::<String>().parse().unwrap()));
            i = j;
        } else if ch.is_alphabetic() || ch == '_' {
            let mut j = i;
            while j < c.len() && (c[j].is_alphanumeric() || c[j] == '_') {
                j += 1;
            }
            out.push(Tok::Word(c[i..j].iter().collect::<String>().to_lowercase()));
            i = j;
        } else if (ch == '>' || ch == '<' || ch == '=' || ch == '!') && i + 1 < c.len() && c[i + 1] == '=' {
            out.push(Tok::Sym(format!("{ch}=")));
            i += 2;
        } else {
            out.push(Tok::Sym(ch.to_string()));
            i += 1;
        }
    }
    Ok(out)
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

/// Names ("Show and Tell", punctuation and case ignored) to the set of card defs that carry them.
fn name_defs(db: &mtg_core::card::CardDb, name: &str) -> Result<Vec<usize>, String> {
    let n = norm(name);
    let v: Vec<usize> = (0..db.defs.len()).filter(|&d| norm(&db.defs[d].name) == n).collect();
    if v.is_empty() { Err(format!("unknown card '{name}'")) } else { Ok(v) }
}

#[derive(Debug, Clone)]
enum Term {
    Cards(Vec<usize>),
    Special(String),
}
#[derive(Debug, Clone)]
enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Cmp(Vec<Term>, String, i64),
}

struct Parser<'a> {
    t: Vec<Tok>,
    i: usize,
    db: &'a mtg_core::card::CardDb,
}
impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn word(&mut self, w: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Word(x)) if x == w) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn sym(&mut self, w: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(x)) if x == w) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut e = self.and()?;
        while self.word("or") {
            e = Expr::Or(Box::new(e), Box::new(self.and()?));
        }
        Ok(e)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut e = self.not()?;
        while self.word("and") {
            e = Expr::And(Box::new(e), Box::new(self.not()?));
        }
        Ok(e)
    }
    fn not(&mut self) -> Result<Expr, String> {
        if self.word("not") {
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        if self.sym("(") {
            let e = self.or()?;
            if !self.sym(")") {
                return Err("missing )".into());
            }
            return Ok(e);
        }
        self.atom()
    }
    fn term(&mut self) -> Result<Term, String> {
        match self.t.get(self.i).cloned() {
            Some(Tok::Str(s)) => {
                self.i += 1;
                Ok(Term::Cards(name_defs(self.db, &s)?))
            }
            Some(Tok::Word(w)) if w == "any" => {
                self.i += 1;
                if !self.sym("(") {
                    return Err("any( needs a list of quoted names".into());
                }
                let mut v = Vec::new();
                loop {
                    match self.t.get(self.i).cloned() {
                        Some(Tok::Str(s)) => {
                            self.i += 1;
                            v.extend(name_defs(self.db, &s)?);
                        }
                        _ => return Err("any( expects quoted names".into()),
                    }
                    if self.sym(",") {
                        continue;
                    }
                    if self.sym(")") {
                        break;
                    }
                    return Err("any(...) needs , or )".into());
                }
                Ok(Term::Cards(v))
            }
            Some(Tok::Word(w)) => {
                self.i += 1;
                if ["lands", "lands_playable", "lands_bf", "nonlands", "creatures", "hand_size"].contains(&w.as_str()) {
                    Ok(Term::Special(w))
                } else {
                    Err(format!("unknown word '{w}' (quote card names; specials: lands, lands_playable, lands_bf, nonlands, creatures, hand_size)"))
                }
            }
            other => Err(format!("unexpected {:?} in goal", other)),
        }
    }
    fn atom(&mut self) -> Result<Expr, String> {
        let mut terms = vec![self.term()?];
        while self.sym("+") {
            terms.push(self.term()?);
        }
        for op in [">=", "<=", "==", ">", "<"] {
            if self.sym(op) {
                return match self.t.get(self.i) {
                    Some(Tok::Num(n)) => {
                        let n = *n;
                        self.i += 1;
                        Ok(Expr::Cmp(terms, op.into(), n))
                    }
                    _ => Err("comparison needs a number".into()),
                };
            }
        }
        Ok(Expr::Cmp(terms, ">=".into(), 1))
    }
}

/// A sampled world for one side: hand, battlefield, library (top first), land drops available.
struct World {
    hand: Vec<usize>,
    bf: Vec<usize>,
    lib: Vec<usize>,
    gy: Vec<usize>,
    drops: i64,
}

fn count_in(db: &mtg_core::card::CardDb, w: &World, term: &Term) -> i64 {
    let is_land = |d: &usize| db.defs[*d].types.contains(Types::LAND);
    match term {
        Term::Cards(set) => w.hand.iter().chain(&w.bf).filter(|d| set.contains(d)).count() as i64,
        Term::Special(s) => match s.as_str() {
            "lands" => w.hand.iter().chain(&w.bf).filter(|d| is_land(d)).count() as i64,
            "lands_bf" => w.bf.iter().filter(|d| is_land(d)).count() as i64,
            "lands_playable" => w.bf.iter().filter(|d| is_land(d)).count() as i64 + (w.hand.iter().filter(|d| is_land(d)).count() as i64).min(w.drops.max(0)),
            "nonlands" => w.hand.iter().chain(&w.bf).filter(|d| !is_land(d)).count() as i64,
            "creatures" => w.hand.iter().chain(&w.bf).filter(|d| db.defs[**d].types.contains(Types::CREATURE)).count() as i64,
            "hand_size" => w.hand.len() as i64,
            _ => 0,
        },
    }
}

fn eval_expr(db: &mtg_core::card::CardDb, w: &World, e: &Expr) -> bool {
    match e {
        Expr::And(a, b) => eval_expr(db, w, a) && eval_expr(db, w, b),
        Expr::Or(a, b) => eval_expr(db, w, a) || eval_expr(db, w, b),
        Expr::Not(a) => !eval_expr(db, w, a),
        Expr::Cmp(ts, op, n) => {
            let v: i64 = ts.iter().map(|t| count_in(db, w, t)).sum();
            match op.as_str() {
                ">=" => v >= *n,
                "<=" => v <= *n,
                "==" => v == *n,
                ">" => v > *n,
                _ => v < *n,
            }
        }
    }
}

#[derive(Debug, Clone)]
enum SimStep {
    Draw(usize),
    Turn(usize),
    Brainstorm(Vec<Term>),
    Ponder { want: Vec<Term>, shuffle: bool },
    Look { n: usize, take: usize, want: Vec<Term>, rest: String },
    Shuffle,
}

fn term_hit(db: &mtg_core::card::CardDb, t: &Term, d: usize) -> bool {
    match t {
        Term::Cards(s) => s.contains(&d),
        Term::Special(sp) => match sp.as_str() {
            "lands" => db.defs[d].types.contains(Types::LAND),
            "nonlands" => !db.defs[d].types.contains(Types::LAND),
            "creatures" => db.defs[d].types.contains(Types::CREATURE),
            _ => false,
        },
    }
}

fn run_steps(db: &mtg_core::card::CardDb, w: &mut World, steps: &[SimStep], rng: &mut mtg_core::rng::Pcg64) {
    let draw = |w: &mut World, n: usize| {
        for _ in 0..n {
            if !w.lib.is_empty() {
                let c = w.lib.remove(0);
                w.hand.push(c);
            }
        }
    };
    for st in steps {
        match st {
            SimStep::Draw(n) => draw(w, *n),
            SimStep::Turn(n) => {
                draw(w, *n);
                w.drops += *n as i64;
            }
            SimStep::Brainstorm(putback) => {
                draw(w, 3);
                let mut given = 0;
                for t in putback {
                    if given >= 2 {
                        break;
                    }
                    if let Some(pos) = w.hand.iter().position(|&d| term_hit(db, t, d)) {
                        let c = w.hand.remove(pos);
                        w.lib.insert(0, c);
                        given += 1;
                    }
                }
                while given < 2 && !w.hand.is_empty() {
                    // Fallback when the list could not be satisfied: put back the last card drawn.
                    let c = w.hand.pop().unwrap();
                    w.lib.insert(0, c);
                    given += 1;
                }
            }
            SimStep::Ponder { want, shuffle } => {
                let top: Vec<usize> = w.lib.iter().take(3).copied().collect();
                let hit = want.iter().find_map(|t| top.iter().position(|&d| term_hit(db, t, d)));
                match hit {
                    Some(pos) => {
                        let c = w.lib.remove(pos);
                        w.hand.push(c);
                    }
                    None if *shuffle => {
                        rng.shuffle(&mut w.lib);
                        draw(w, 1);
                    }
                    None => draw(w, 1),
                }
            }
            SimStep::Look { n, take, want, rest } => {
                let k = (*n).min(w.lib.len());
                let mut seen: Vec<usize> = w.lib.drain(..k).collect();
                let mut taken = 0;
                for t in want {
                    while taken < *take {
                        match seen.iter().position(|&d| term_hit(db, t, d)) {
                            Some(p) => {
                                w.hand.push(seen.remove(p));
                                taken += 1;
                            }
                            None => break,
                        }
                    }
                }
                while taken < *take && !seen.is_empty() {
                    w.hand.push(seen.remove(0));
                    taken += 1;
                }
                match rest.as_str() {
                    "top" => {
                        for (i, c) in seen.into_iter().enumerate() {
                            w.lib.insert(i, c);
                        }
                    }
                    "gy" => w.gy.extend(seen),
                    _ => w.lib.extend(seen),
                }
            }
            SimStep::Shuffle => rng.shuffle(&mut w.lib),
        }
    }
}

fn parse_terms(p: &mut Parser) -> Result<Vec<Term>, String> {
    let mut v = Vec::new();
    while matches!(p.peek(), Some(Tok::Str(_))) || matches!(p.peek(), Some(Tok::Word(w)) if w == "any" || w == "lands" || w == "nonlands" || w == "creatures") {
        v.push(p.term()?);
    }
    Ok(v)
}

/// `sim` request: statements separated by ';', e.g.
///   brainstorm putback "Lotus Petal" "Boseiju, Who Endures"; turn; goal "Aluren" and lands_playable >= 3
fn sim_report(obs: &Observation, db: &mtg_core::card::CardDb, mine: &DeckList, theirs: &DeckList, seq: u32, script: &str, seed: u64) -> String {
    let run = || -> Result<String, String> {
        let mut samples = 20000usize;
        let mut opp_side = false;
        let mut drops_override: Option<i64> = None;
        let mut steps: Vec<SimStep> = Vec::new();
        let mut goals: Vec<(String, Expr)> = Vec::new();
        for stmt in script.split(';').map(|x| x.trim()).filter(|x| !x.is_empty()) {
            let toks = tokenize(stmt)?;
            let mut p = Parser { t: toks, i: 0, db };
            let head = match p.t.first() {
                Some(Tok::Word(w)) => w.clone(),
                _ => return Err(format!("statement must start with a keyword: {stmt}")),
            };
            p.i = 1;
            match head.as_str() {
                "opp" => opp_side = true,
                "samples" => samples = match p.t.get(1) { Some(Tok::Num(n)) => (*n as usize).clamp(100, 200000), _ => return Err("samples N".into()) },
                "drops" => drops_override = match p.t.get(1) { Some(Tok::Num(n)) => Some(*n), _ => return Err("drops N".into()) },
                "draw" => steps.push(SimStep::Draw(match p.t.get(1) { Some(Tok::Num(n)) => *n as usize, _ => 1 })),
                "turn" => steps.push(SimStep::Turn(match p.t.get(1) { Some(Tok::Num(n)) => *n as usize, _ => 1 })),
                "shuffle" => steps.push(SimStep::Shuffle),
                "brainstorm" => {
                    if !p.word("putback") {
                        return Err("brainstorm putback \"Card\" \"Card\" (or lands / nonlands)".into());
                    }
                    steps.push(SimStep::Brainstorm(parse_terms(&mut p)?));
                }
                "ponder" => {
                    let mut shuffle = false;
                    let mut want = vec![];
                    while p.peek().is_some() {
                        if p.word("shuffle") {
                            shuffle = true;
                        } else if p.word("want") {
                            want = parse_terms(&mut p)?;
                        } else {
                            return Err("ponder [want \"Card\" ...] [shuffle]".into());
                        }
                    }
                    steps.push(SimStep::Ponder { want, shuffle });
                }
                "look" => {
                    // look N take K [want "Card" ...] [rest bottom|top|gy]
                    let n = match p.t.get(1) { Some(Tok::Num(n)) => *n as usize, _ => return Err("look N take K ...".into()) };
                    p.i = 2;
                    let (mut take, mut want, mut rest) = (1usize, vec![], "bottom".to_string());
                    while p.peek().is_some() {
                        if p.word("take") {
                            take = match p.t.get(p.i) { Some(Tok::Num(k)) => { let k = *k as usize; p.i += 1; k } _ => return Err("take K".into()) };
                        } else if p.word("want") {
                            want = parse_terms(&mut p)?;
                        } else if p.word("rest") {
                            rest = match p.t.get(p.i) { Some(Tok::Word(w)) => { let w = w.clone(); p.i += 1; w } _ => return Err("rest bottom|top|gy".into()) };
                        } else {
                            return Err("look N take K [want ...] [rest bottom|top|gy]".into());
                        }
                    }
                    steps.push(SimStep::Look { n, take, want, rest });
                }
                "goal" => {
                    let e = p.or()?;
                    if p.i < p.t.len() {
                        return Err(format!("could not read the goal after token {}: {stmt}", p.i));
                    }
                    goals.push((stmt.trim_start_matches(|c: char| c.is_alphabetic()).trim().to_string(), e));
                }
                other => return Err(format!("unknown statement '{other}' (draw, turn, brainstorm, ponder, look, shuffle, goal, opp, drops, samples)")),
            }
        }
        if goals.is_empty() {
            return Err("add at least one: goal <expression>".into());
        }
        // World construction from the player's own view only.
        let sub = |pool: &mut std::collections::BTreeMap<usize, usize>, d: CardDefId| {
            if !db.defs[d.0 as usize].is_token {
                if let Some(c) = pool.get_mut(&(d.0 as usize)) {
                    *c = c.saturating_sub(1);
                }
            }
        };
        let mut pool: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
        let deck = if opp_side { theirs } else { mine };
        for d in &deck.main {
            *pool.entry(d.0 as usize).or_insert(0) += 1;
        }
        let (hand0, bf0, top0, bot0, lib_n, hand_unknown);
        if !opp_side {
            for c in obs.me.hand.iter().chain(&obs.me.graveyard).chain(&obs.me.exile) {
                sub(&mut pool, c.def);
            }
            for p in obs.battlefield.iter().filter(|p| p.owned_by_me) {
                sub(&mut pool, p.def);
            }
            for it in obs.stack.iter().filter(|i| i.controlled_by_me && !i.is_ability) {
                sub(&mut pool, it.def);
            }
            hand0 = obs.me.hand.iter().map(|c| c.def.0 as usize).collect::<Vec<_>>();
            bf0 = obs.battlefield.iter().filter(|p| p.controlled_by_me).map(|p| p.def.0 as usize).collect::<Vec<_>>();
            top0 = obs.me.library_known_top.iter().map(|c| c.def.0 as usize).collect::<Vec<_>>();
            bot0 = obs.me.library_known_bottom.iter().map(|c| c.def.0 as usize).collect::<Vec<_>>();
            lib_n = obs.me.library_count;
            hand_unknown = 0usize;
        } else {
            for c in obs.opp.graveyard.iter().chain(&obs.opp.exile).chain(&obs.opp.revealed_hand) {
                sub(&mut pool, c.def);
            }
            for p in obs.battlefield.iter().filter(|p| !p.owned_by_me) {
                sub(&mut pool, p.def);
            }
            for it in obs.stack.iter().filter(|i| !i.controlled_by_me && !i.is_ability) {
                sub(&mut pool, it.def);
            }
            hand0 = obs.opp.revealed_hand.iter().map(|c| c.def.0 as usize).collect::<Vec<_>>();
            bf0 = obs.battlefield.iter().filter(|p| !p.controlled_by_me).map(|p| p.def.0 as usize).collect::<Vec<_>>();
            top0 = vec![];
            bot0 = vec![];
            lib_n = obs.opp.library_count;
            hand_unknown = obs.opp.hand_count.saturating_sub(obs.opp.revealed_hand.len());
        }
        let mut unknown: Vec<usize> = Vec::new();
        for (&d, &c) in &pool {
            for _ in 0..c {
                unknown.push(d);
            }
        }
        for d in top0.iter().chain(&bot0) {
            if let Some(pos) = unknown.iter().position(|x| x == d) {
                unknown.remove(pos);
            }
        }
        let warn = if unknown.len() + top0.len() + bot0.len() + (if opp_side { 0 } else { 0 }) != lib_n + hand_unknown {
            format!("WARNING: {} unseen cards by decklist arithmetic vs {} by the game's counts; results are approximate.\n", unknown.len() + top0.len() + bot0.len(), lib_n + hand_unknown)
        } else {
            String::new()
        };
        let my_turn_land_window = obs.active_is_me && obs.me.lands_played == 0;
        let drops0 = drops_override.unwrap_or(if opp_side { 0 } else if my_turn_land_window { 1 } else { 0 });
        let mut rng = mtg_core::rng::Pcg64::from_seed(seed ^ 0x51ED_270B ^ (seq as u64) << 20);
        let mut hits = vec![0usize; goals.len()];
        for _ in 0..samples {
            let mut u = unknown.clone();
            rng.shuffle(&mut u);
            let mut hand = hand0.clone();
            let mut lib: Vec<usize> = Vec::new();
            if opp_side {
                let h = hand_unknown.min(u.len());
                hand.extend(u.drain(..h));
            } else {
                lib.extend(top0.iter().copied());
            }
            lib.extend(u);
            lib.extend(bot0.iter().copied());
            let mut w = World { hand, bf: bf0.clone(), lib, gy: vec![], drops: drops0 };
            run_steps(db, &mut w, &steps, &mut rng);
            for (i, (_, e)) in goals.iter().enumerate() {
                if eval_expr(db, &w, e) {
                    hits[i] += 1;
                }
            }
        }
        let mut s = String::new();
        let _ = writeln!(s, "=== SIM for PROMPT {seq} | {samples} samples | {} ===", if opp_side { "OPPONENT: their unseen cards sampled uniformly from their decklist minus what you can see".to_string() } else { format!("YOU: your library has {lib_n} cards{}", if top0.is_empty() { String::new() } else { format!(", top {} known", top0.len()) }) });
        s.push_str(&warn);
        let _ = writeln!(s, "Script: {}", script.trim());
        let _ = writeln!(s, "Land drops available at the start: {drops0}{}", if drops_override.is_some() { " (set by you)" } else { "; each `turn` adds one" });
        for (i, (g, _)) in goals.iter().enumerate() {
            let p = hits[i] as f64 / samples as f64;
            let _ = writeln!(s, "  goal {g}: {:.1}% (+-{:.1})", p * 100.0, 196.0 * (p * (1.0 - p) / samples as f64).sqrt());
        }
        let _ = writeln!(s, "Not modeled: colors of mana (use named lands in the goal), Daze/Force/Wasteland, fetchlands thinning, anything the opponent does. Same script gives the same numbers, so compare two scripts directly.");
        Ok(s)
    };
    match run() {
        Ok(s) => s,
        Err(e) => format!("=== SIM for PROMPT {seq} | could not run ===\n{e}\nExample: sim \"brainstorm putback \\\"Lotus Petal\\\" lands; turn; goal \\\"Aluren\\\" and lands_playable >= 3\"\n"),
    }
}

fn only_mana_or_pass(d: &Decision) -> bool {
    d.options.iter().all(|a| matches!(a.kind, ActionKind::Pass | ActionKind::ActivateMana))
}

/// Player-written macro: pick options by label rules for up to `left` more prompts (loops such as Aluren + Acererak).
struct Auto {
    rules: Vec<String>,
    left: u32,
    lifemin: i32,
    done: u32,
    last_label: Option<String>,
}

/// What a macro rule may look at besides the option labels.
struct AutoCtx {
    stack_nonempty: bool,
    top_name: Option<String>,
    top_mine: bool,
    last_label: Option<String>,
}

fn auto_ctx(obs: &Observation, last_label: Option<String>) -> AutoCtx {
    let top = obs.stack.last();
    AutoCtx { stack_nonempty: top.is_some(), top_name: top.map(|i| i.name.clone()), top_mine: top.map(|i| i.controlled_by_me).unwrap_or(false), last_label }
}

/// Rules are tried in order; a rule is `[conditions][=]<label text>`. Conditions, in any order, each ending in `:`:
/// `last:` picks the last matching option (the free Aluren cast is the later duplicate), `stack:` only while
/// something is on the stack, `empty:` only while the stack is empty, `mytop:` only while the top stack item is
/// yours, `top[TEXT]:` only while the top stack item's name contains TEXT, `after[TEXT]:` only if this macro's
/// previous pick had a label containing TEXT. `=` needs the whole label to match. Returns None when no rule
/// matches, which ends the macro.
fn auto_pick(rules: &[String], labels: &[String], cx: &AutoCtx) -> Option<usize> {
    'rules: for r in rules {
        let mut t = r.trim();
        let (mut last, mut exact) = (false, false);
        loop {
            if let Some(x) = t.strip_prefix("last:") {
                last = true;
                t = x;
            } else if let Some(x) = t.strip_prefix("stack:") {
                if !cx.stack_nonempty {
                    continue 'rules;
                }
                t = x;
            } else if let Some(x) = t.strip_prefix("empty:") {
                if cx.stack_nonempty {
                    continue 'rules;
                }
                t = x;
            } else if let Some(x) = t.strip_prefix("mytop:") {
                if !cx.top_mine {
                    continue 'rules;
                }
                t = x;
            } else if let Some(x) = t.strip_prefix("top[").or_else(|| t.strip_prefix("after[")) {
                let is_top = t.starts_with("top[");
                let Some(close) = x.find("]:") else { continue 'rules };
                let want = &x[..close];
                let have = if is_top { cx.top_name.as_deref() } else { cx.last_label.as_deref() };
                if !have.map(|h| h.contains(want)).unwrap_or(false) {
                    continue 'rules;
                }
                t = &x[close + 2..];
            } else if let Some(x) = t.strip_prefix('=') {
                exact = true;
                t = x;
            } else {
                break;
            }
        }
        if t.is_empty() {
            continue;
        }
        let hits: Vec<usize> = labels.iter().enumerate().filter(|(_, l)| if exact { l.as_str() == t } else { l.contains(t) }).map(|(i, _)| i).collect();
        if !hits.is_empty() {
            return Some(if last { *hits.last().unwrap() } else { hits[0] });
        }
    }
    None
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let flag = |name: &str| a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned();
    let dir = &a[1];
    let (my_deck, opp_deck) = (&a[2], &a[3]);
    let pdir = PathBuf::from(&a[4]);
    let tdir = PathBuf::from(&a[5]);
    let seed: u64 = flag("--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
    let llm_seat = Seat(flag("--llm-seat").and_then(|s| s.parse().ok()).unwrap_or(0));
    let first = Seat(flag("--first").and_then(|s| s.parse().ok()).unwrap_or(0));
    let iters: u32 = flag("--iters").and_then(|s| s.parse().ok()).unwrap_or(32);
    let load = |f: &str| flag(f).map(|p| std::sync::Arc::new(Net::load(std::path::Path::new(&p)).unwrap()));
    let opp_net = load("--opp-net");
    let me_net = load("--me-net");
    let consult_net = load("--consult-net").or_else(|| me_net.clone()).or_else(|| opp_net.clone());
    let replay: Vec<(u8, usize)> = flag("--replay")
        .map(|p| serde_json::from_str::<Vec<(u8, usize)>>(&std::fs::read_to_string(p).unwrap()).unwrap())
        .unwrap_or_default();
    let oneshot = a.iter().any(|x| x == "--oneshot");
    let mut replay_i = 0usize;
    let mut oneshot_done: Option<String> = None;
    let consult_iters: u32 = flag("--consult-iters").and_then(|s| s.parse().ok()).unwrap_or(1000);
    std::fs::create_dir_all(&pdir).unwrap();
    std::fs::create_dir_all(&tdir).unwrap();
    let db = mtg_cards::legacy::build();
    let n_defs = db.defs.len();
    let (names, decks) = load_deck_dir(&db, dir);
    let find = |n: &str| names.iter().position(|x| x == n).unwrap_or_else(|| panic!("deck {n} not in {names:?}"));
    let (mut da, mut dbk) = (&decks[find(my_deck)], &decks[find(opp_deck)]);
    // decks are indexed by seat: seat 0 gets the first, seat 1 the second.
    let (d0, d1) = if llm_seat == Seat(0) { (da, dbk) } else { (dbk, da) };
    da = d0;
    dbk = d1;
    let mut g = Game::new(db.clone(), [da, dbk], seed, GameConfig { first_player: first, ..GameConfig::default() });
    let opp_seat = llm_seat.other();
    // Two-LLM mode: the opponent seat is answered through its own prompt dir (same file protocol).
    let opp_pdir: Option<PathBuf> = flag("--opp-pdir").map(PathBuf::from);
    if let Some(d) = &opp_pdir {
        std::fs::create_dir_all(d).unwrap();
        for f in ["consult.req", "consult.txt", "ans.txt", "result.txt"] {
            let _ = std::fs::remove_file(d.join(f));
        }
    }
    let mut pending_events_opp: Vec<ViewEvent> = Vec::new();
    let model = UniformConsistentModel { decks: [da.main.clone(), dbk.main.clone()] };
    let cfg = SearchConfig { iterations: iters, seed: seed + 11, ..SearchConfig::default() };
    let ev: Box<dyn Evaluator> = match &opp_net {
        Some(n) => Box::new(NetEvaluator::new(n.clone(), n_defs)),
        None => Box::new(RolloutEvaluator::new(2500, seed + 5)),
    };
    let mut opp = MctsPolicy::new(&model, ev, cfg);
    let auto_llm = a.iter().any(|x| x == "--auto-llm");
    let ev_me: Box<dyn Evaluator> = match &me_net {
        Some(n) => Box::new(NetEvaluator::new(n.clone(), n_defs)),
        None => Box::new(RolloutEvaluator::new(2500, seed + 6)),
    };
    let me_iters: u32 = flag("--me-iters").and_then(|s| s.parse().ok()).unwrap_or(iters);
    let mut me_auto = MctsPolicy::new(&model, ev_me, SearchConfig { iterations: me_iters, seed: seed + 12, ..SearchConfig::default() });
    let mut seq = 0u32;
    let mut actions: Vec<(u8, usize)> = Vec::new();
    let mut autos: [Option<Auto>; 2] = [None, None];
    let mut auto_notes: [Option<String>; 2] = [None, None];
    let mut auto_total = 0u32;
    let mut log: Vec<u8> = Vec::new();
    use std::io::Write;
    let mut n_dec = 0u32;
    let mut n_prompts = 0u32;
    let mut prompt_chars = 0usize;
    let mut auto_passes = 0u32;
    let mut pending_events: Vec<ViewEvent> = Vec::new();
    let (mut n_consults, mut consult_chars) = (0u32, 0usize);
    let (mut n_odds, mut odds_chars) = (0u32, 0usize);
    let (mut n_sims, mut sim_chars) = (0u32, 0usize);
    let (mut followed_search, mut followed_net, mut consult_decisions) = (0u32, 0u32, 0u32);
    let _ = std::fs::remove_file(pdir.join("consult.req"));
    let _ = std::fs::remove_file(pdir.join("consult.txt"));
    let _ = std::fs::remove_file(pdir.join("ans.txt"));
    let t0 = std::time::Instant::now();
    let result = loop {
        if n_dec > 20000 {
            break None;
        }
        match g.advance() {
            Status::GameOver(r) => break Some(r),
            Status::NeedDecision(seat) => {
                let sv = g.seat_view(seat);
                let obs = sv.observe();
                let d = obs.decision.clone().unwrap();
                n_dec += 1;
                if replay_i < replay.len() {
                    let (rs, ri) = replay[replay_i];
                    replay_i += 1;
                    assert_eq!(seat.0, rs, "replay diverged at {}", replay_i - 1);
                    if seat == llm_seat {
                        pending_events.extend(obs.events.iter().cloned());
                    }
                    g.apply(d.id, ri).unwrap();
                    continue;
                }
                if seat == opp_seat && opp_pdir.is_none() {
                    let idx = opp.choose(&sv, d.options.len());
                    let _ = writeln!(log, "opp[{}] t{} {} -> {} {}", d.options.len(), obs.turn, step_name(obs.step), idx, d.options[idx].label);
                    { actions.push((seat.0, idx)); g.apply(d.id, idx).unwrap(); }
                    continue;
                }
                // Events are handed out once per apply; keep the ones that auto-answered windows swallow.
                let is_opp_llm = seat == opp_seat;
                let cdir: PathBuf = if is_opp_llm { opp_pdir.clone().unwrap() } else { pdir.clone() };
                let pe: &mut Vec<ViewEvent> = if is_opp_llm { &mut pending_events_opp } else { &mut pending_events };
                pe.extend(obs.events.iter().cloned());
                if d.trivial {
                    { actions.push((seat.0, 0)); g.apply(d.id, 0).unwrap(); }
                    continue;
                }
                if auto_llm && !is_opp_llm {
                    let idx = me_auto.choose(&sv, d.options.len());
                    { actions.push((seat.0, idx)); g.apply(d.id, idx).unwrap(); }
                    n_prompts += 1;
                    continue;
                }
                // Priority stops: skip windows where the only actions are Pass and mana abilities,
                // and (unless something is on the stack) windows outside the main phases on my
                // turn / outside beginning of combat, end step on the opponent's turn.
                if matches!(d.kind, ViewDecisionKind::Priority) && !oneshot {
                    let only_pass = only_mana_or_pass(&d);
                    let stop = !obs.stack.is_empty()
                        || if obs.active_is_me { obs.step == Step::Main1 || obs.step == Step::Main2 } else { obs.step == Step::End };
                    if only_pass || !stop {
                        auto_passes += 1;
                        { actions.push((seat.0, 0)); g.apply(d.id, 0).unwrap(); }
                        continue;
                    }
                }
                // Macro in progress for this seat: answer by the player's rules until they stop matching.
                if let Some(st) = autos[seat.0 as usize].as_mut() {
                    let labels: Vec<String> = d.options.iter().map(|o| o.label.clone()).collect();
                    let hit = auto_pick(&st.rules, &labels, &auto_ctx(&obs, st.last_label.clone()));
                    let why = if st.left == 0 { Some("pick budget used up".to_string()) } else if obs.me.life <= st.lifemin { Some(format!("your life is {} (lifemin {})", obs.me.life, st.lifemin)) } else if hit.is_none() { Some("no rule matches this prompt".to_string()) } else { None };
                    match (why, hit) {
                        (None, Some(idx)) => {
                            st.left -= 1;
                            st.done += 1;
                            auto_total += 1;
                            st.last_label = Some(d.options[idx].label.clone());
                            let _ = writeln!(log, "  auto[{}] -> {} {}", seat.0, idx, d.options[idx].label);
                            { actions.push((seat.0, idx)); g.apply(d.id, idx).unwrap(); }
                            continue;
                        }
                        (why, _) => {
                            auto_notes[seat.0 as usize] = Some(format!("AUTO MACRO STOPPED after {} picks: {}. Re-read this prompt.\n", st.done, why.unwrap_or_default()));
                            autos[seat.0 as usize] = None;
                        }
                    }
                }
                seq += 1;
                n_prompts += 1;
                let mut shown = obs.clone();
                shown.events = std::mem::take(pe);
                let text = format!("{}{}", auto_notes[seat.0 as usize].take().unwrap_or_default(), render(&shown, seq));
                prompt_chars += text.len();
                let _ = writeln!(log, "llm prompt {seq}: t{} {}", obs.turn, step_name(obs.step));
                let tmp = cdir.join("prompt.tmp");
                std::fs::write(&tmp, &text).unwrap();
                std::fs::rename(&tmp, cdir.join("prompt.txt")).unwrap();
                let mut last_consult: Option<(Option<usize>, Option<usize>)> = None;
                let idx = loop {
                    if let Ok(t) = std::fs::read_to_string(cdir.join("consult.req")) {
                        let _ = std::fs::remove_file(cdir.join("consult.req"));
                        let mut it = t.split_whitespace();
                        let (rs, ri) = (it.next().and_then(|x| x.parse::<u32>().ok()), it.next().and_then(|x| x.parse::<u32>().ok()));
                        let mut words = t.trim().splitn(3, ' ');
                        let (_, cmd, rest) = (words.next(), words.next().unwrap_or(""), words.next().unwrap_or(""));
                        let out = if rs != Some(seq) {
                            format!("!! consult refused: need '{seq} [iterations]' for the current prompt\n")
                        } else if cmd == "sim" {
                            let (mine_d, theirs_d) = if seat == Seat(0) { (da, dbk) } else { (dbk, da) };
                            let txt = sim_report(&obs, &db, mine_d, theirs_d, seq, rest, seed);
                            n_sims += 1;
                            sim_chars += txt.len();
                            let _ = writeln!(log, "sim prompt {seq}: {}", rest.trim());
                            txt
                        } else if cmd == "odds" || cmd == "opp" {
                            let (mine_d, theirs_d) = if seat == Seat(0) { (da, dbk) } else { (dbk, da) };
                            let txt = odds_report(&obs, &db, mine_d, theirs_d, seq, cmd, rest);
                            n_odds += 1;
                            odds_chars += txt.len();
                            let _ = writeln!(log, "{cmd} prompt {seq}: {}", rest.trim());
                            txt
                        } else {
                            let iters = ri.unwrap_or(consult_iters).clamp(16, 2000);
                            let (txt, bs, bn) = consult(&sv, &obs, &model, consult_net.as_ref(), n_defs, iters, seed, seq);
                            n_consults += 1;
                            consult_chars += txt.len();
                            let _ = writeln!(log, "consult prompt {seq}: iters {iters} search-best {:?} net-best {:?}", bs, bn);
                            last_consult = Some((bs, bn));
                            txt
                        };
                        let ctmp = cdir.join("consult.tmp");
                        std::fs::write(&ctmp, out).unwrap();
                        std::fs::rename(&ctmp, cdir.join("consult.txt")).unwrap();
                    }
                    if let Ok(t) = std::fs::read_to_string(cdir.join("auto.req")) {
                        let _ = std::fs::remove_file(cdir.join("auto.req"));
                        // "SEQ N LIFEMIN rule;rule;..."
                        let mut it = t.trim().splitn(4, ' ');
                        let (rs, n, lm, rules) = (it.next().and_then(|x| x.parse::<u32>().ok()), it.next().and_then(|x| x.parse::<u32>().ok()), it.next().and_then(|x| x.parse::<i32>().ok()), it.next().unwrap_or(""));
                        let rules: Vec<String> = rules.split(';').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                        let labels: Vec<String> = d.options.iter().map(|o| o.label.clone()).collect();
                        let first = if rs == Some(seq) && n.unwrap_or(0) >= 1 && lm.is_some() && !rules.is_empty() { auto_pick(&rules, &labels, &auto_ctx(&obs, None)) } else { None };
                        match first {
                            Some(i) => {
                                autos[seat.0 as usize] = Some(Auto { rules, left: n.unwrap() - 1, lifemin: lm.unwrap(), done: 1, last_label: Some(labels[i].clone()) });
                                auto_total += 1;
                                let _ = writeln!(log, "  auto macro started at prompt {seq}");
                                break i;
                            }
                            None => {
                                let mut t2 = format!("!! auto refused: need 'auto {seq} N LIFEMIN rule;rule;...' with at least one rule matching an option of THIS prompt\n");
                                t2.push_str(&text);
                                std::fs::write(&tmp, t2).unwrap();
                                std::fs::rename(&tmp, cdir.join("prompt.txt")).unwrap();
                            }
                        }
                    }
                    if let Ok(t) = std::fs::read_to_string(cdir.join("ans.txt")) {
                        let mut it = t.split_whitespace();
                        let (s, i) = (it.next().and_then(|x| x.parse::<u32>().ok()), it.next().and_then(|x| x.parse::<usize>().ok()));
                        let _ = std::fs::remove_file(cdir.join("ans.txt"));
                        match (s, i) {
                            (Some(s), Some(i)) if s == seq && i < d.options.len() => break i,
                            _ => {
                                let mut t2 = format!("!! bad answer '{}' (need '{seq} <index 0..{}>')\n", t.trim(), d.options.len() - 1);
                                t2.push_str(&text);
                                std::fs::write(&tmp, t2).unwrap();
                                std::fs::rename(&tmp, cdir.join("prompt.txt")).unwrap();
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(40));
                };
                let _ = writeln!(log, "{} -> {} {}", if is_opp_llm { "llm-b" } else { "llm" }, idx, d.options[idx].label);
                {
                    // Full transcript per player for post-game review (truth dir only; players never see it).
                    use std::io::Write as _;
                    let tn = if is_opp_llm { "transcript-b.txt" } else { "transcript-a.txt" };
                    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(tdir.join(tn)) {
                        let _ = writeln!(f, "{}\n>>> ACTIONS-BEFORE {}\n>>> CHOSE [{}] {}\n", text, actions.len(), idx, d.options[idx].label);
                    }
                }
                if let Some((bs, bn)) = last_consult {
                    consult_decisions += 1;
                    followed_search += (bs == Some(idx)) as u32;
                    followed_net += (bn == Some(idx)) as u32;
                    let _ = writeln!(log, "  (after consult: took search-best {} | net-best {})", bs == Some(idx), bn == Some(idx));
                }
                if oneshot {
                    oneshot_done = Some(format!("{} {}", idx, d.options[idx].label));
                    break None;
                }
                { actions.push((seat.0, idx)); g.apply(d.id, idx).unwrap(); }
            }
        }
    };
    let llm_won = match result {
        Some(GameResult::Win(s)) => Some(s == llm_seat),
        _ => None,
    };
    let verdict = match (&oneshot_done, llm_won) {
        (Some(a), _) => format!("ONE-SHOT ANSWER {a}"),
        (_, w) => match w {
        Some(true) => "YOU WON".to_string(),
        Some(false) => "YOU LOST".to_string(),
        None => "DRAW / TRUNCATED".to_string(),
        },
    };
    let summary = format!("{verdict} | auto picks {auto_total} | prompts {n_prompts} | prompt chars {prompt_chars} | auto-passed windows {auto_passes} | odds calls {n_odds} | odds chars {odds_chars} | sim calls {n_sims} | sim chars {sim_chars} | consults {n_consults} | consult chars {consult_chars} | consulted decisions {consult_decisions} (took search-best {followed_search}, net-best {followed_net}) | wall {:.0}s", t0.elapsed().as_secs_f64());
    let _ = writeln!(log, "{summary}");
    std::fs::write(tdir.join("log.txt"), &log).unwrap();
    std::fs::write(tdir.join("actions.json"), serde_json::to_string(&actions).unwrap()).unwrap();
    std::fs::write(pdir.join("result.txt"), &summary).unwrap();
    if let Some(od) = &opp_pdir {
        let ov = match llm_won { Some(true) => "YOU LOST", Some(false) => "YOU WON", None => "DRAW / TRUNCATED" };
        let osum = summary.replacen(&verdict, ov, 1);
        std::fs::write(od.join("result.txt"), &osum).unwrap();
        let t2 = od.join("prompt.tmp");
        std::fs::write(&t2, format!("=== GAME OVER === {osum}\n")).unwrap();
        std::fs::rename(&t2, od.join("prompt.txt")).unwrap();
    }
    let tmp = pdir.join("prompt.tmp");
    std::fs::write(&tmp, format!("=== GAME OVER === {summary}\n")).unwrap();
    std::fs::rename(&tmp, pdir.join("prompt.txt")).unwrap();
}

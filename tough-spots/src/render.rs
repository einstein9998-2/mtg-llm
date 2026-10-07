//! Text rendering of a seat's Observation (copied from llm-benchmark/harness/llmgame.rs, option list
//! removed: the flagger prints its own option table). Reads only the Observation, nothing hidden.
use mtg_core::types::*;
use mtg_view::*;
use std::fmt::Write as _;

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

pub fn event_str(e: &ViewEvent) -> Option<String> {
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

pub fn render(o: &Observation, _seq: u32) -> String {
    let d = o.decision.as_ref().expect("decision");
    let mut s = String::new();
    let _ = writeln!(s, "Turn {} | {} | {} turn", o.turn, step_name(o.step), if o.active_is_me { "YOUR" } else { "OPPONENT'S" });
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
    s
}


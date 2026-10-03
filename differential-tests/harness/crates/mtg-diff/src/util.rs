//! Naming helpers over the engine's core types (harness side only; `diff-harness` feature).

use mtg_core::card::CardDb;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::state::State;
use mtg_core::types::*;

pub fn def_name(db: &CardDb, s: &State, r: ObjRef) -> String {
    db.def(s.def_of(r)).name.clone()
}

/// The way a cast option uses, as a short lowercase text: "" for the plain mana cost, otherwise the
/// alternative cost's key and label, or the name of the virtual way.
/// A creature's identity for combat specs: its name plus its counters, `Name{P1P1=2}` (same-named creatures with the same counters are interchangeable).
pub fn creature_label(db: &CardDb, s: &State, r: ObjRef) -> String {
    let mut cs: Vec<String> = s.obj_data(db, r).counters.iter().filter(|(_, n)| *n > 0).map(|(k, n)| format!("{}={}", counter_name(*k), n)).collect();
    cs.sort();
    format!("{}{{{}}}", def_name(db, s, r), cs.join(","))
}

pub fn way_text(db: &CardDb, s: &State, r: ObjRef, way: u8) -> String {
    use mtg_core::restrict::*;
    let def = db.def(s.def_of(r));
    if way == 0 {
        return String::new();
    }
    if way == ADVENTURE_WAY {
        return "adventure".into();
    }
    if way == FREE_WAY {
        return "free-effect".into();
    }
    if way == ENERGY_WAY {
        return "energy".into();
    }
    if way == GY_WAY {
        return "graveyard-effect".into();
    }
    if way >= VIRTUAL_WAY_BASE {
        return format!("permit:{}", db.permit_keys.get((way - VIRTUAL_WAY_BASE) as usize).cloned().unwrap_or_default());
    }
    match mtg_core::cost::alt_of(def, way) {
        Some(a) => format!("{} {}", a.key, a.label),
        None => format!("way{way}"),
    }
}

pub fn ability_text(db: &CardDb, def: CardDefId, ability: u8) -> String {
    db.def(def).abilities.get(ability as usize).map(|a| format!("{a:?}")).unwrap_or_default()
}

pub fn subtype_name(t: u8) -> &'static str {
    SUBTYPE_NAMES[t as usize]
}

pub fn step_name(s: Step) -> &'static str {
    match s {
        Step::Untap => "UNTAP",
        Step::Upkeep => "UPKEEP",
        Step::Draw => "DRAW",
        Step::Main1 => "MAIN1",
        Step::BeginCombat => "COMBAT_BEGIN",
        Step::DeclareAttackers => "COMBAT_DECLARE_ATTACKERS",
        Step::DeclareBlockers => "COMBAT_DECLARE_BLOCKERS",
        Step::FirstStrikeDamage => "COMBAT_FIRST_STRIKE_DAMAGE",
        Step::CombatDamage => "COMBAT_DAMAGE",
        Step::EndCombat => "COMBAT_END",
        Step::Main2 => "MAIN2",
        Step::End => "END_OF_TURN",
        Step::Cleanup => "CLEANUP",
    }
}

pub fn step_from_name(n: &str) -> Option<Step> {
    Some(match n {
        "UNTAP" => Step::Untap,
        "UPKEEP" => Step::Upkeep,
        "DRAW" => Step::Draw,
        "MAIN1" => Step::Main1,
        "COMBAT_BEGIN" => Step::BeginCombat,
        "COMBAT_DECLARE_ATTACKERS" => Step::DeclareAttackers,
        "COMBAT_DECLARE_BLOCKERS" => Step::DeclareBlockers,
        "COMBAT_FIRST_STRIKE_DAMAGE" => Step::FirstStrikeDamage,
        "COMBAT_DAMAGE" => Step::CombatDamage,
        "COMBAT_END" => Step::EndCombat,
        "MAIN2" => Step::Main2,
        "END_OF_TURN" => Step::End,
        "CLEANUP" => Step::Cleanup,
        _ => return None,
    })
}

pub fn counter_from_forge(n: &str) -> Option<CounterKind> {
    Some(match n {
        "P1P1" => CounterKind::PlusOne,
        "M1M1" => CounterKind::MinusOne,
        "LOYALTY" => CounterKind::Loyalty,
        "ENERGY" => CounterKind::Energy,
        "TIME" => CounterKind::Time,
        "STUN" => CounterKind::Stun,
        "CHARGE" => CounterKind::Charge,
        "FLYING" => CounterKind::Flying,
        _ => return None,
    })
}

pub fn counter_name(k: CounterKind) -> String {
    match k {
        CounterKind::PlusOne => "P1P1".into(),
        CounterKind::MinusOne => "M1M1".into(),
        CounterKind::Loyalty => "LOYALTY".into(),
        CounterKind::Energy => "ENERGY".into(),
        CounterKind::Time => "TIME".into(),
        CounterKind::Stun => "STUN".into(),
        CounterKind::Charge => "CHARGE".into(),
        CounterKind::Flying => "FLYING".into(),
        CounterKind::Other(n) => format!("OTHER{n}"),
    }
}

pub fn types_text(t: Types) -> Vec<&'static str> {
    let mut v = Vec::new();
    for (f, n) in [
        (Types::LAND, "Land"),
        (Types::CREATURE, "Creature"),
        (Types::ARTIFACT, "Artifact"),
        (Types::ENCHANTMENT, "Enchantment"),
        (Types::PLANESWALKER, "Planeswalker"),
        (Types::INSTANT, "Instant"),
        (Types::SORCERY, "Sorcery"),
        (Types::KINDRED, "Kindred"),
        (Types::DUNGEON, "Dungeon"),
        (Types::EMBLEM, "Emblem"),
    ] {
        if t.contains(f) {
            v.push(n);
        }
    }
    v
}

//! Position export for the action-set differential: a Rust-engine priority window as Forge `GameState`
//! lines plus the engine's own legal-action set in canonical keys.

use crate::util::*;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::state::ObjKind;
use mtg_core::types::*;
use mtg_view::Game;
use std::collections::BTreeMap;

pub struct Position {
    pub turn: u16,
    pub step: &'static str,
    pub active: u8,
    /// 0 = the active player's first window, 1 = the non-active player's window after the active player passed
    pub window: u8,
    pub priority: u8,
    pub state: Vec<String>,
    /// canonical action keys of the priority seat (kind|zone|name|alt; activations carry a count)
    pub actions: Vec<String>,
    /// Forge-side canonical state summary to verify the injection, per seat
    pub summary: Vec<String>,
    /// names of permanents whose abilities were activated this turn (per-turn limits are history Forge cannot be given)
    pub activated: Vec<String>,
    /// canonical key per priority option (None for Pass and mana abilities), parallel to `Pending::options`
    pub option_keys: Vec<Option<String>>,
}

fn forge_player(s: Seat) -> &'static str {
    if s.0 == 0 {
        "human"
    } else {
        "ai"
    }
}

fn zone_text(z: ZoneKind) -> &'static str {
    match z {
        ZoneKind::Hand => "Hand",
        ZoneKind::Battlefield => "Battlefield",
        ZoneKind::Graveyard => "Graveyard",
        ZoneKind::Exile => "Exile",
        ZoneKind::Command => "Command",
        ZoneKind::Library => "Library",
        ZoneKind::Stack => "Stack",
        ZoneKind::Gone => "Gone",
    }
}

/// Canonical action keys of the priority window `g` is at (any stack): the sorted key set and the key per option (None for Pass and mana abilities).
pub fn window_keys(g: &Game) -> Result<(Vec<String>, Vec<Option<String>>), String> {
    let s = g.raw_state();
    let db = g.db();
    let p = g.pending().ok_or("no decision")?;
    let name = |r: ObjRef| def_name(db, s, r);
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut act_idx: BTreeMap<(String, String), std::collections::BTreeSet<u8>> = BTreeMap::new();
    let mut okeys: Vec<Option<String>> = vec![];
    for o in &p.options {
        match *o {
            Opt::Pass => okeys.push(None),
            Opt::PlayLand(r) => {
                let k = format!("play|{}|{}|", zone_text(s.zone_of(r)), name(r));
                counts.insert(k.clone(), 1);
                okeys.push(Some(k));
            }
            Opt::PlayLandBack(r, _) => {
                let k = format!("playback|{}|{}|", zone_text(s.zone_of(r)), name(r));
                counts.insert(k.clone(), 1);
                okeys.push(Some(k));
            }
            Opt::Cast(r, w) => {
                let alt = if w == 0 || w == mtg_core::restrict::ADVENTURE_WAY { "" } else { "alt" };
                let k = format!("cast|{}|{}|{}", zone_text(s.zone_of(r)), name(r), alt);
                counts.insert(k.clone(), 1);
                okeys.push(Some(k));
            }
            Opt::Activate { src, ability } => {
                act_idx.entry((zone_text(s.zone_of(src)).to_string(), name(src))).or_default().insert(ability);
                okeys.push(Some(format!("act|{}|{}|", zone_text(s.zone_of(src)), name(src))));
            }
            Opt::Mana { .. } => okeys.push(None),
            _ => return Err(format!("unexpected priority option {o:?}")),
        }
    }
    for ((z, n), set) in &act_idx {
        counts.insert(format!("act|{z}|{n}|{}", set.len()), 1);
    }
    for k in okeys.iter_mut().flatten() {
        if k.starts_with("act|") {
            let parts: Vec<&str> = k.split('|').collect();
            let set = &act_idx[&(parts[1].to_string(), parts[2].to_string())];
            *k = format!("{k}{}", set.len());
        }
    }
    Ok((counts.into_keys().collect(), okeys))
}

/// Exports the current priority window of the seat that has priority with an empty stack in a main phase.
/// Returns `Err(reason)` when the position cannot be injected faithfully into Forge.
pub fn export(g: &Game, activated: &std::collections::BTreeSet<String>) -> Result<Position, String> {
    let s = g.raw_state();
    let db = g.db();
    let p = g.pending().ok_or("no decision")?;
    if !matches!(p.kind, DecisionKind::Priority) {
        return Err("not priority".into());
    }
    let td = s.turn_data();
    let step = match td.step {
        Step::Upkeep => "UPKEEP",
        Step::Draw => "DRAW",
        Step::Main1 => "MAIN1",
        Step::BeginCombat => "COMBAT_BEGIN",
        Step::Main2 => "MAIN2",
        Step::End => "END_OF_TURN",
        _ => return Err("step".into()),
    };
    if !s.stack().is_empty() {
        return Err("stack".into());
    }
    if s.combat().is_some() {
        return Err("combat".into());
    }
    for seat in [Seat(0), Seat(1)] {
        if s.spells_cast(seat) > 0 {
            return Err("spells cast this turn".into());
        }
        if s.energy(seat) > 0 {
            return Err("energy".into());
        }
        if s.pool(seat).iter().any(|&x| x > 0) {
            return Err("mana in pool".into());
        }
        if s.dungeon_of(seat).is_some() || !s.emblem_objs(seat).is_empty() || !s.completed_dungeons(seat).is_empty() {
            return Err("dungeon/emblem".into());
        }
    }
    let name = |r: ObjRef| def_name(db, s, r);
    let mut state: Vec<String> = vec![];
    let mut summary: Vec<String> = vec![];
    for seat in [Seat(0), Seat(1)] {
        let pre = forge_player(seat);
        state.push(format!("{pre}life={}", s.life(seat)));
        let hand: Vec<String> = s.hand(seat).iter().map(|&r| name(r)).collect();
        state.push(format!("{pre}hand={}", hand.join(";")));
        let lib: Vec<String> = s.library(seat).iter().rev().map(|&r| name(r)).collect();
        state.push(format!("{pre}library={}", lib.join(";")));
        let gy: Vec<String> = s.graveyard(seat).iter().map(|&r| name(r)).collect();
        state.push(format!("{pre}graveyard={}", gy.join(";")));
        let mut ex = vec![];
        for &r in s.exile() {
            let d = s.obj_data(db, r);
            if d.kind != ObjKind::Card {
                return Err("non-card exile".into());
            }
            if format!("{:?}", db.def(d.def).abilities).contains("Adventure") {
                return Err("adventure-capable card in exile (Forge needs the OnAdventure flag)".into());
            }
            if d.owner == seat {
                ex.push(name(r));
            }
        }
        state.push(format!("{pre}exile={}", ex.join(";")));
        let mut bf = vec![];
        let mut bf_sum = vec![];
        for &r in s.battlefield() {
            let d = s.obj_data(db, r);
            if d.controller != seat {
                continue;
            }
            if d.kind != ObjKind::Card {
                return Err("token".into());
            }
            if d.owner != d.controller {
                return Err("stolen permanent".into());
            }
            if d.attached_to.is_some() {
                return Err("attachment".into());
            }
            if format!("{:?}", db.def(d.def).abilities).contains("EntersChoice") {
                return Err("chosen-state permanent".into());
            }
            let mut t = name(r);
            if t == "Overlord of the Balemurk" && d.counters.iter().any(|(k, n)| *k == CounterKind::Time && *n > 0) {
                return Err("impending permanent (Forge cannot be injected with the impending flag)".into());
            }
            let mut line_t = t.clone();
            if db.def(d.def).is_back {
                // a modal double-faced card played as its back face: Forge injects the front card in its back state
                let front = db.defs.iter().find(|f| f.back.as_deref() == Some(t.as_str())).ok_or("front face of back def not found")?;
                if front.back_kind == Some(mtg_core::card::FaceKind::Transform) {
                    return Err("transformed permanent (Forge cannot be injected in its back state)".into());
                }
                line_t = format!("{}|Modal|NoETBTrigs", front.name);
            }
            if d.tapped {
                t.push_str("|Tapped");
            }
            if d.summoning_sick {
                t.push_str("|SummonSick");
            }
            if d.damage > 0 {
                t.push_str(&format!("|Damage:{}", d.damage));
            }
            let mut cs = vec![];
            for (k, n) in &d.counters {
                if let mtg_core::types::CounterKind::Other(_) = k {
                    return Err("exotic counter".into());
                }
                cs.push(format!("{}={}", counter_name(*k), n));
            }
            cs.sort();
            if !cs.is_empty() {
                t.push_str(&format!("|Counters:{}", cs.join(",")));
            }
            let suffix = t[name(r).len()..].to_string();
            if d.chars.types.contains(Types::LAND) {
                t = format!("{}^L{}", name(r), suffix);
            }
            bf_sum.push(t.clone());
            bf.push(format!("{line_t}{suffix}"));
        }
        state.push(format!("{pre}battlefield={}", bf.join(";")));
        if seat == td.active {
            state.push(format!("{pre}landsplayed={}", s.lands_played(seat)));
        }
        let mut hs = hand.clone();
        hs.sort();
        bf_sum.sort();
        let mut gys = gy.clone();
        gys.sort();
        let mut exs = ex.clone();
        exs.sort();
        summary.push(format!("{pre} life={} hand={} bf={} gy={} ex={} lib={}", s.life(seat), hs.join(";"), bf_sum.join(";"), gys.join(";"), exs.join(";"), lib.len()));
    }
    state.push(format!("turn={}", td.turn.max(1) + 1));
    state.push(format!("activeplayer={}", forge_player(td.active)));
    state.push(format!("activephase={step}"));
    state.push("removesummoningsickness=false".into());

    let (actions, okeys) = window_keys(g)?;
    Ok(Position { turn: td.turn, step, active: td.active.0, window: if p.seat == td.active { 0 } else { 1 }, priority: p.seat.0, state, actions, summary, activated: activated.iter().cloned().collect(), option_keys: okeys })
}


/// Post-state summary in the same text format as the Forge probe's `summary` (tokens included, `^L` marks lands, `^T` tokens).
pub fn summarize(g: &Game) -> Vec<String> {
    let s = g.raw_state();
    let db = g.db();
    let mut out = vec![];
    for seat in [Seat(0), Seat(1)] {
        let pre = forge_player(seat);
        let mut hand: Vec<String> = s.hand(seat).iter().map(|&r| def_name(db, s, r)).collect();
        hand.sort();
        let mut gy: Vec<String> = s.graveyard(seat).iter().map(|&r| def_name(db, s, r)).collect();
        gy.sort();
        let mut ex = vec![];
        for &r in s.exile() {
            let d = s.obj_data(db, r);
            if d.owner == seat {
                ex.push(def_name(db, s, r));
            }
        }
        ex.sort();
        let mut bf = vec![];
        for &r in s.battlefield() {
            let d = s.obj_data(db, r);
            if d.controller != seat {
                continue;
            }
            let mut t = def_name(db, s, r);
            if d.chars.types.contains(Types::LAND) {
                t.push_str("^L");
            }
            if d.kind != ObjKind::Card {
                t.push_str("^T");
            }
            if d.tapped {
                t.push_str("|Tapped");
            }
            if d.summoning_sick {
                t.push_str("|SummonSick");
            }
            if d.damage > 0 {
                t.push_str(&format!("|Damage:{}", d.damage));
            }
            let mut cs: Vec<String> = d.counters.iter().filter(|(_, n)| *n > 0).map(|(k, n)| format!("{}={}", counter_name(*k), n)).collect();
            cs.sort();
            if !cs.is_empty() {
                t.push_str(&format!("|Counters:{}", cs.join(",")));
            }
            bf.push(t);
        }
        bf.sort();
        out.push(format!("{pre} life={} hand={} bf={} gy={} ex={} lib={}", s.life(seat), hand.join(";"), bf.join(";"), gy.join(";"), ex.join(";"), s.library(seat).len()));
    }
    out
}

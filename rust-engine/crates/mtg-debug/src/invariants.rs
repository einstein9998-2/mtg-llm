//! Invariants I1-I14 (doc 04 section 7.1) that can be checked from a single state at a decision
//! point. Event-based invariants (I3, I9) are checked by `EventAudit`.

use mtg_core::card::CardDb;
use mtg_core::cx::Cx;
use mtg_core::event::Event;
use mtg_core::ids::*;
use mtg_core::state::*;
use mtg_core::types::*;

#[derive(Debug, Clone)]
pub struct Violation {
    pub id: &'static str,
    pub msg: String,
}

fn v(id: &'static str, msg: String) -> Result<(), Violation> {
    Err(Violation { id, msg })
}

/// Checks I1, I2, I4, I5, I6, I8, I11, I14 on a state that is waiting at a decision.
pub fn check(s: &State, db: &CardDb) -> Result<(), Violation> {
    check_zones(s)?;
    check_refs(s)?;
    check_counters(s)?;
    check_decision(s)?;
    check_sba_stable(s, db)?;
    check_derived(s, db)?;
    check_permanent_types(s, db)?;
    Ok(())
}

/// I15: an instant or sorcery card is never a permanent on the battlefield (what a sampled hidden
/// identity put where a pick no longer fits would produce).
fn check_permanent_types(s: &State, db: &CardDb) -> Result<(), Violation> {
    for &r in s.battlefield() {
        let t = db.def(s.def_of(r)).types;
        if t.intersects(Types::INSTANT | Types::SORCERY) {
            return v("I15", format!("{r:?} ({}) is an instant or sorcery on the battlefield", db.def(s.def_of(r)).name));
        }
    }
    Ok(())
}

fn check_zones(s: &State) -> Result<(), Violation> {
    let n = s.num_slots();
    let mut seen = vec![0u8; n];
    let mut mark = |r: ObjRef, zone: ZoneKind, owner_zone: Option<Seat>| -> Result<(), Violation> {
        if r.slot as usize >= n {
            return v("I2", format!("slot {} out of range", r.slot));
        }
        if !s.is_live_ref(r) {
            return v("I2", format!("stale ref {r:?} in zone {zone:?}"));
        }
        let (z, _k, _g, owner) = s.obj_raw_zone(r.slot);
        if z != zone {
            return v("I2", format!("{r:?} listed in {zone:?} but obj.zone={z:?}"));
        }
        if let Some(o) = owner_zone {
            if owner != o {
                return v("I2", format!("{r:?} in {o:?}'s {zone:?} but owned by {owner:?}"));
            }
        }
        seen[r.slot as usize] += 1;
        if seen[r.slot as usize] > 1 {
            return v("I2", format!("{r:?} appears in two zone lists"));
        }
        Ok(())
    };
    for p in 0..2u8 {
        let seat = Seat(p);
        for &r in s.library(seat) {
            mark(r, ZoneKind::Library, Some(seat))?;
        }
        for &r in s.hand(seat) {
            mark(r, ZoneKind::Hand, Some(seat))?;
        }
        for &r in s.graveyard(seat) {
            mark(r, ZoneKind::Graveyard, Some(seat))?;
        }
    }
    for &r in s.battlefield() {
        mark(r, ZoneKind::Battlefield, None)?;
    }
    for &r in s.exile() {
        mark(r, ZoneKind::Exile, None)?;
    }
    for p in 0..2u8 {
        for &r in s.emblem_objs(Seat(p)) {
            mark(r, ZoneKind::Command, None)?;
        }
        if let Some(r) = s.dungeon_obj(Seat(p)) {
            mark(r, ZoneKind::Command, None)?;
        }
    }
    for e in s.stack() {
        mark(e.obj, ZoneKind::Stack, None)?;
    }
    // I1: every physical card is in exactly one zone; non-card objects are listed iff not Gone.
    for slot in 0..n as u16 {
        let (z, k, _g, _o) = s.obj_raw_zone(slot);
        let listed = seen[slot as usize];
        match k {
            ObjKind::Card => {
                if listed != 1 {
                    return v("I1", format!("card slot {slot} (zone {z:?}) is in {listed} zone lists"));
                }
            }
            _ => {
                if z == ZoneKind::Gone {
                    if listed != 0 {
                        return v("I1", format!("gone object slot {slot} still listed"));
                    }
                    if !s.free_slots().contains(&slot) {
                        return v("I1", format!("gone object slot {slot} not on the free list"));
                    }
                } else if listed != 1 {
                    return v("I1", format!("live token/ability slot {slot} (zone {z:?}) in {listed} lists"));
                }
            }
        }
    }
    Ok(())
}

fn check_refs(s: &State) -> Result<(), Violation> {
    // I4: attachments (none in M1) and stack entries are checked here as the model grows.
    for e in s.stack() {
        if let StackKind::Ability { source: _, .. } = e.kind {
            // The source may legitimately be gone (sacrificed as a cost): last-known info is used.
        }
        if e.controller.idx() > 1 {
            return v("I6", "stack entry with invalid controller".into());
        }
    }
    Ok(())
}

fn check_counters(s: &State) -> Result<(), Violation> {
    for &r in s.battlefield() {
        let c = s.obj_counters_raw(r.slot);
        let p = c.iter().find(|x| x.0 == CounterKind::PlusOne).map(|x| x.1).unwrap_or(0);
        let m = c.iter().find(|x| x.0 == CounterKind::MinusOne).map(|x| x.1).unwrap_or(0);
        if p > 0 && m > 0 {
            return v("I8", format!("{r:?} has both +1/+1 and -1/-1 counters at a decision point"));
        }
    }
    Ok(())
}

fn check_decision(s: &State) -> Result<(), Violation> {
    if let Some(p) = s.pending() {
        if p.options.is_empty() {
            return v("I13", format!("decision with no options: {:?}", p.kind));
        }
        if matches!(p.kind, mtg_core::decision::DecisionKind::Priority) {
            if s.turn_data().priority != p.seat {
                return v("I6", "priority decision seat differs from priority holder".into());
            }
            if s.frame_depth() != 1 {
                return v("I14", format!("priority decision with frame depth {}", s.frame_depth()));
            }
        }
    }
    Ok(())
}

fn check_sba_stable(s: &State, db: &CardDb) -> Result<(), Violation> {
    if let Some(p) = s.pending() {
        if matches!(p.kind, mtg_core::decision::DecisionKind::Priority) {
            let mut c = s.clone();
            let mut cx = Cx::new(&mut c, db);
            if cx.sba_pass() {
                return v("I5", "SBA would act when a player receives priority".into());
            }
        }
    }
    Ok(())
}

fn check_derived(s: &State, db: &CardDb) -> Result<(), Violation> {
    if s.derived_is_dirty() {
        return Ok(()); // coherence is only defined on a clean cache
    }
    let mut c = s.clone();
    c.mark_derived_dirty();
    Cx::new(&mut c, db).refresh();
    for &r in s.battlefield() {
        let a = s.obj_data(db, r).chars;
        let b = c.obj_data(db, r).chars;
        if a != b {
            return v("I11", format!("derived cache stale for {r:?}: cached {a:?} vs recomputed {b:?}"));
        }
    }
    Ok(())
}

/// Event-stream audit: legal zone transitions (I3) and life-total reconciliation (I9).
pub struct EventAudit {
    pub life: [i32; 2],
}

impl EventAudit {
    pub fn new(start_life: i32) -> EventAudit {
        EventAudit { life: [start_life; 2] }
    }

    pub fn feed(&mut self, events: &[Event]) -> Result<(), Violation> {
        for e in events {
            match e {
                Event::LifeChange { player, delta } => self.life[player.idx()] += *delta,
                Event::ZoneChange { from, to, .. } => {
                    use ZoneKind::*;
                    let ok = !matches!((from, to), (Library, Stack) | (Battlefield, Stack) | (Hand, Hand) | (Stack, Hand) | (Stack, Library));
                    // Casting from graveyard or exile (flashback, escape) is legal; Stack->Hand/Library is not (yet).
                    if !ok {
                        return v("I3", format!("illegal zone transition {from:?} -> {to:?}"));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn check_life(&self, s: &State) -> Result<(), Violation> {
        for p in 0..2 {
            if self.life[p] != s.life(Seat(p as u8)) {
                return v("I9", format!("life of seat {p}: events say {} but state says {}", self.life[p], s.life(Seat(p as u8))));
            }
        }
        Ok(())
    }
}

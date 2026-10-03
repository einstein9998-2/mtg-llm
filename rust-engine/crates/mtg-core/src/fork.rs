//! Determinization support (doc 02 section 5.3): which physical cards the observer cannot
//! identify, what it does know about the rest, and filling the hidden slots with a sampled
//! assignment. The sampling itself (the belief model) lives in `mtg-view`.
//!
//! The fork is built from the true state by *overwriting* every identity the observer is not
//! entitled to know and every position it does not know, so nothing hidden survives in the
//! result. Property (tested in `mtg-debug`): two true states that agree on everything the
//! observer may know give identical forks for the same seed and assignment.

use crate::card::CardDb;
use crate::ids::*;
use crate::ir::ObjFilter;
use crate::rng::Pcg64;
use crate::state::*;
use crate::types::*;

/// What a belief model needs to know. Built only from the observer's knowledge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenRequest {
    pub observer: Seat,
    /// Per owner: number of physical cards whose identity the observer does not know.
    pub unknown: [usize; 2],
    /// Per owner: front-face definitions of the physical cards the observer does know (any zone),
    /// sorted. Tokens and copies are not physical cards and are not listed.
    pub known: [Vec<CardDefId>; 2],
}

/// Identities for the unknown cards of each owner, in ascending slot order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenAssignment {
    pub defs: [Vec<CardDefId>; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForkError {
    /// Options of a pending decision can depend on the decider's hidden cards (Force of Will's
    /// "exile a blue card", castable spells), so only the decider's own decision point can be
    /// forked: the observer must be the seat to act.
    NotObserversDecision,
    /// The assignment does not fit the request (wrong count) or the decklist cannot explain what
    /// the observer has seen.
    Inconsistent(String),
}

impl State {
    fn unknown_slots(&self, observer: Seat) -> [Vec<u16>; 2] {
        let bit = 1u8 << observer.idx();
        let mut out: [Vec<u16>; 2] = [Vec::new(), Vec::new()];
        for (slot, o) in self.objs.iter().enumerate() {
            if o.kind == ObjKind::Card && o.zone != ZoneKind::Gone && self.knowledge[slot].known_to & bit == 0 {
                out[o.owner.idx()].push(slot as u16);
            }
        }
        out
    }

    /// The part of the state a belief model may condition on.
    pub fn hidden_request(&self, db: &CardDb, observer: Seat) -> HiddenRequest {
        let bit = 1u8 << observer.idx();
        let unknown = self.unknown_slots(observer);
        let mut known: [Vec<CardDefId>; 2] = [Vec::new(), Vec::new()];
        for (slot, o) in self.objs.iter().enumerate() {
            if o.kind == ObjKind::Card && o.zone != ZoneKind::Gone && self.knowledge[slot].known_to & bit != 0 {
                let d = db.def(o.def);
                known[o.owner.idx()].push(if d.is_back { d.front_id.unwrap_or(o.def) } else { o.def });
            }
        }
        known[0].sort();
        known[1].sort();
        HiddenRequest { observer, unknown: [unknown[0].len(), unknown[1].len()], known }
    }

    /// Fills every hidden identity from `asg`, shuffles the library positions the observer does
    /// not know, forgets the opponent's sideboard and reseeds the RNG.
    pub fn determinize(&mut self, db: &CardDb, observer: Seat, asg: &HiddenAssignment, seed: u64) -> Result<(), ForkError> {
        let unknown = self.unknown_slots(observer);
        for p in 0..2 {
            if asg.defs[p].len() != unknown[p].len() {
                return Err(ForkError::Inconsistent(format!("seat {p}: {} hidden cards but {} identities", unknown[p].len(), asg.defs[p].len())));
            }
        }
        // Position-based dealing. Nothing about which slot (object) sits where may influence what
        // the fork shows: known-identity objects with unknown position are re-placed at random
        // free positions, and sampled identities are dealt by position within zone (hand order,
        // then library order), never by slot number.
        let bit = 1u8 << observer.idx();
        let mut rng = Pcg64::from_seed(seed ^ 0x6A09_E667_F3BC_C908);
        for p in 0..2 {
            let lib = self.players[p].library.clone();
            let free: Vec<usize> = (0..lib.len()).filter(|&i| self.knowledge[lib[i].slot as usize].pos_known_to & bit == 0).collect();
            let is_known = |s: &State, r: ObjRef| s.knowledge[r.slot as usize].known_to & bit != 0;
            let mut kn: Vec<ObjRef> = free.iter().map(|&i| lib[i]).filter(|&r| is_known(self, r)).collect();
            kn.sort_by_key(|r| (self.objs[r.slot as usize].def, self.vids[r.slot as usize][observer.idx()]));
            let un: Vec<ObjRef> = free.iter().map(|&i| lib[i]).filter(|&r| !is_known(self, r)).collect();
            let mut pos: Vec<usize> = free.clone();
            rng.shuffle(&mut pos);
            let mut kpos: Vec<usize> = pos[..kn.len()].to_vec();
            kpos.sort();
            let mut l = self.players[p].library.clone();
            let mut ui = un.iter();
            let kset: std::collections::BTreeSet<usize> = kpos.iter().copied().collect();
            let mut ki = kn.iter();
            for &i in &free {
                l[i] = if kset.contains(&i) { *ki.next().unwrap() } else { *ui.next().unwrap() };
            }
            self.players[p].library = l;
        }
        for p in 0..2 {
            let mut order: Vec<u16> = Vec::new();
            let hand = self.players[p].hand.clone();
            for r in hand.iter().filter(|r| self.knowledge[r.slot as usize].known_to & bit == 0) {
                order.push(r.slot);
            }
            let lib = self.players[p].library.clone();
            for r in lib.iter().filter(|r| self.knowledge[r.slot as usize].known_to & bit == 0) {
                order.push(r.slot);
            }
            let mut rest: Vec<u16> = unknown[p].iter().copied().filter(|s| !order.contains(s)).collect();
            rest.sort();
            order.extend(rest);
            assert_eq!(order.len(), asg.defs[p].len());
            for (&slot, &def) in order.iter().zip(asg.defs[p].iter()) {
                let o = &mut self.objs[slot as usize];
                o.def = def;
                o.card = Some(CardId(slot));
            }
        }
        // What the other seat privately remembers about its own library (scried positions, cards it
        // looked at) is hidden state too: forget it.
        for p in 0..2 {
            for r in self.players[p].library.clone() {
                let k = &mut self.knowledge[r.slot as usize];
                k.pos_known_to &= bit;
                if k.known_to & bit == 0 {
                    k.known_to = 0;
                }
            }
        }
        self.players[observer.other().idx()].sideboard.clear();
        self.rng = Pcg64::from_seed(seed);
        self.derived_dirty = true;
        self.resample_secret_picks(db, observer, seed);
        Ok(())
    }

    /// Secret choices the opponent has already made are hidden state: the pick stored in a frame
    /// refers to a card whose identity was just re-sampled, so it may no longer satisfy the
    /// effect's filter. Show and Tell is the only such effect in the pool: when the first chooser
    /// is the opponent, their pick becomes a uniform choice among the candidates of the sampled
    /// hand or "nothing".
    fn resample_secret_picks(&mut self, db: &CardDb, observer: Seat, seed: u64) {
        use crate::compile::Instr;
        use crate::frame::Frame;
        use crate::ir::Effect;
        // (frame index, filter for Show and Tell or None for Stronghold Gambit)
        let mut todo: Vec<(usize, Option<ObjFilter>)> = Vec::new();
        for (i, fr) in self.frames.iter().enumerate() {
            if let Frame::Resolve(f) = fr {
                if f.aux == 1 && self.turn.active != observer {
                    match crate::resolve::entry_code(db, &f.entry).get(f.pc as usize) {
                        Some(Instr::Leaf(Effect::EachPutFromHand { filter })) => todo.push((i, Some(filter.clone()))),
                        Some(Instr::Leaf(Effect::GambitReveal)) => todo.push((i, None)),
                        _ => {}
                    }
                }
            }
        }
        if todo.is_empty() {
            return;
        }
        let mut rng = Pcg64::from_seed(seed ^ 0xBB67_AE85_84CA_A73B);
        let seat = self.turn.active;
        for (i, filter) in todo {
            let cands: Vec<ObjRef> = {
                let mut cx = crate::cx::Cx::new(self, db);
                cx.refresh();
                let hand = cx.s.players[seat.idx()].hand.clone();
                hand.into_iter()
                    .filter(|&r| match &filter {
                        Some(filter) => {
                            let c = cx.chars(r);
                            cx.filter_match(filter, r, &c, false, seat)
                        }
                        None => true,
                    })
                    .collect()
            };
            // Always resampled, whatever the true pick was (including "nothing"): the fork must not
            // depend on the secret choice. Gambit has no "nothing" unless the hand is empty.
            let k = if filter.is_some() { rng.below(cands.len() as u64 + 1) as usize } else if cands.is_empty() { 0 } else { 1 + rng.below(cands.len() as u64) as usize };
            if let Frame::Resolve(f) = &mut self.frames[i] {
                f.sc[0] = if k == 0 { 0 } else { cands[k - 1].slot + 1 };
            }
        }
    }
}

//! Read-only access to `State` for `mtg-view` (projection) and `mtg-debug` (full dumps).
//!
//! State fields are `pub(crate)`; everything leaves the crate through here. Accessors that expose
//! hidden information (library contents and order, the other seat's hand, the RNG) exist only with
//! the `diff-harness` feature (doc 02 section 3). The projection code in `mtg-view` calls the
//! always-available accessors only for the observer's own seat or for public zones; the
//! non-interference test (doc 04 section 7.2) is the real guard.

use crate::card::CardDb;
use crate::decision::*;
use crate::frame::{CombatStage, Frame};
use crate::hash::Fx64;
#[cfg(feature = "diff-harness")]
use crate::rng::Pcg64;
use crate::ids::*;
use crate::state::*;
use crate::types::*;
use std::hash::{Hash, Hasher};

/// Raw object data, including both seats' view ids.
#[derive(Clone, Debug)]
pub struct ObjData {
    pub kind: ObjKind,
    pub def: CardDefId,
    pub owner: Seat,
    pub controller: Seat,
    pub zone: ZoneKind,
    pub vid: [ViewId; 2],
    pub tapped: bool,
    pub damage: u16,
    pub counters: Vec<(CounterKind, u16)>,
    pub chars: Chars,
    pub summoning_sick: bool,
    pub attached_to: Option<ObjRef>,
}

#[derive(Clone, Debug)]
pub struct TurnData {
    pub turn: u16,
    pub step: Step,
    pub active: Seat,
    pub priority: Seat,
}

impl State {
    pub fn turn_data(&self) -> TurnData {
        TurnData { turn: self.turn.number, step: self.turn.step, active: self.turn.active, priority: self.turn.priority }
    }

    pub fn pending(&self) -> Option<&Pending> {
        self.pending.as_ref()
    }

    pub fn life(&self, s: Seat) -> i32 {
        self.players[s.idx()].life
    }
    /// The dungeon card in the player's command zone and the room their marker is in.
    pub fn dungeon_of(&self, s: Seat) -> Option<(CardDefId, u8)> {
        self.players[s.idx()].dungeon.map(|d| (self.obj(d).def, self.players[s.idx()].room))
    }

    /// The definitions of the emblems the player has.
    pub fn dungeon_obj(&self, s: Seat) -> Option<ObjRef> {
        self.players[s.idx()].dungeon
    }

    pub fn emblem_objs(&self, s: Seat) -> &[ObjRef] {
        &self.players[s.idx()].emblems
    }

    pub fn emblems_of(&self, s: Seat) -> Vec<CardDefId> {
        self.players[s.idx()].emblems.iter().map(|&r| self.obj(r).def).collect()
    }

    /// Times the Ring has tempted the player.
    pub fn ring_level(&self, s: Seat) -> u8 {
        self.players[s.idx()].ring
    }

    pub fn ring_bearer(&self, s: Seat) -> Option<ObjRef> {
        self.players[s.idx()].ring_bearer
    }

    pub fn has_blessing(&self, s: Seat) -> bool {
        self.players[s.idx()].blessing
    }

    pub fn attack_target_of(&self, r: ObjRef) -> Option<crate::decision::AttackTarget> {
        self.combat.as_ref().and_then(|c| c.attackers.iter().find(|a| a.obj == r).map(|a| a.target))
    }

    pub fn completed_dungeons(&self, s: Seat) -> &[CardDefId] {
        &self.players[s.idx()].completed
    }

    /// Cards of `owner`'s library whose identity and position `observer` knows: the known run
    /// from the top (top first) and the known run from the bottom (bottom-most last).
    pub fn known_library(&self, owner: Seat, observer: Seat) -> (Vec<ObjRef>, Vec<ObjRef>) {
        let lib = &self.players[owner.idx()].library;
        let bit = 1u8 << observer.idx();
        let known = |r: &ObjRef| {
            let k = &self.knowledge[r.slot as usize];
            k.known_to & bit != 0 && k.pos_known_to & bit != 0
        };
        let top: Vec<ObjRef> = lib.iter().rev().take_while(|r| known(r)).copied().collect();
        let mut bottom: Vec<ObjRef> = lib.iter().take(lib.len() - top.len()).take_while(|r| known(r)).copied().collect();
        bottom.reverse();
        (top, bottom)
    }

    /// Cards in `owner`'s hand whose identity `observer` knows.
    pub fn known_hand(&self, owner: Seat, observer: Seat) -> Vec<ObjRef> {
        let bit = 1u8 << observer.idx();
        self.players[owner.idx()].hand.iter().filter(|r| self.knowledge[r.slot as usize].known_to & bit != 0).copied().collect()
    }

    pub fn energy(&self, s: Seat) -> u32 {
        self.players[s.idx()].energy
    }
    pub fn pool(&self, s: Seat) -> [u8; 6] {
        self.players[s.idx()].pool.0
    }
    pub fn hand_count(&self, s: Seat) -> usize {
        self.players[s.idx()].hand.len()
    }
    pub fn library_count(&self, s: Seat) -> usize {
        self.players[s.idx()].library.len()
    }
    /// Own hand identities. Callers must only use this for the observer's own seat.
    pub fn hand(&self, s: Seat) -> &[ObjRef] {
        &self.players[s.idx()].hand
    }
    pub fn graveyard(&self, s: Seat) -> &[ObjRef] {
        &self.players[s.idx()].graveyard
    }
    pub fn owner_of(&self, r: ObjRef) -> Seat {
        self.objs[r.slot as usize].owner
    }
    pub fn exile(&self) -> &[ObjRef] {
        &self.exile
    }
    pub fn battlefield(&self) -> &[ObjRef] {
        &self.battlefield
    }
    /// Name of the frame on top of the continuation stack (harness use).
    pub fn top_frame_kind(&self) -> &'static str {
        match self.frames.last() {
            Some(Frame::Turn) => "turn",
            Some(Frame::Pregame(_)) => "pregame",
            Some(Frame::Stabilize(_)) => "stabilize",
            Some(Frame::Cast(_)) => "cast",
            Some(Frame::Resolve(_)) => "resolve",
            Some(Frame::Combat(_)) => "combat",
            Some(Frame::Cleanup(_)) => "cleanup",
            None => "none",
        }
    }

    /// A trigger waiting to be put on the stack (for labeling ordering decisions).
    pub fn pending_trigger(&self, i: usize) -> Option<&PendingTrigger> {
        self.pending_triggers.get(i)
    }

    pub fn stack(&self) -> &[StackEntry] {
        &self.stack
    }
    pub fn combat(&self) -> Option<&CombatState> {
        self.combat.as_ref()
    }
    /// While attackers or blockers are being declared: every creature the declaration is about
    /// (the engine asks about them one by one).
    pub fn declaration_candidates(&self) -> Vec<ObjRef> {
        match self.frames.last() {
            Some(Frame::Combat(f)) => match &f.stage {
                CombatStage::Attackers { cands, .. } | CombatStage::Blockers { cands, .. } => cands.to_vec(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
    pub fn spells_cast(&self, s: Seat) -> u8 {
        self.players[s.idx()].turn.spells_cast
    }
    pub fn lands_played(&self, s: Seat) -> u8 {
        self.players[s.idx()].turn.lands_played
    }
    pub fn is_live_ref(&self, r: ObjRef) -> bool {
        self.is_live(r)
    }
    pub fn view_id(&self, r: ObjRef, seat: Seat) -> ViewId {
        // A stale reference (the object left and the slot was reused) has no view id: what the
        // slot holds now is a different object and must not show through.
        if !self.is_live(r) {
            return ViewId::NONE;
        }
        self.vids[r.slot as usize][seat.idx()]
    }
    pub fn zone_of(&self, r: ObjRef) -> ZoneKind {
        self.objs[r.slot as usize].zone
    }
    pub fn def_of(&self, r: ObjRef) -> CardDefId {
        self.objs[r.slot as usize].def
    }
    pub fn cur_ref_of_slot(&self, slot: u16) -> ObjRef {
        self.cur_ref(slot)
    }
    pub fn num_slots(&self) -> usize {
        self.objs.len()
    }
    pub fn decision_counter(&self, seat: Seat) -> u32 {
        self.next_decision[seat.idx()]
    }

    /// Object data. Battlefield characteristics require a clean derived cache (the owning `Game`
    /// refreshes before projecting).
    pub fn obj_data(&self, db: &CardDb, r: ObjRef) -> ObjData {
        let o = &self.objs[r.slot as usize];
        let chars = if o.zone == ZoneKind::Battlefield { self.derived[r.slot as usize] } else { crate::derive::base_chars(db.def(o.def)) };
        // Raw: came under its controller's control this turn (haste does not change that fact).
        let sick = chars.types.contains(Types::CREATURE) && o.entered_turn >= self.turn.own_turn[o.controller.idx()];
        ObjData {
            kind: o.kind,
            def: o.def,
            owner: o.owner,
            controller: o.controller,
            zone: o.zone,
            vid: self.vids[r.slot as usize],
            tapped: o.tapped,
            damage: o.damage,
            counters: o.counters.to_vec(),
            chars,
            summoning_sick: sick,
            attached_to: o.attached_to,
        }
    }

    /// Takes the buffered events (the owning `Game` converts them to per-seat view events).
    pub fn take_events(&mut self) -> Vec<crate::event::Event> {
        std::mem::take(&mut self.events)
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    // ---- diff-harness only: hidden information ------------------------------------------------

    #[cfg(feature = "diff-harness")]
    pub fn library(&self, s: Seat) -> &[ObjRef] {
        &self.players[s.idx()].library
    }
    #[cfg(feature = "diff-harness")]
    pub fn card_known(&self, slot: u16) -> CardKnow {
        self.knowledge[slot as usize]
    }

    /// Builds the "other world" for the non-interference test: re-randomizes everything the
    /// observer is not entitled to know, keeping counts, zones and the observer's knowledge.
    /// Objects whose identity the observer does not know (`known_to` lacks its bit) are permuted
    /// (def and card id together) within the opponent's hand+library pool, and, if
    /// `own_library`, within the observer's library; then the RNG is reseeded.
    #[cfg(feature = "diff-harness")]
    pub fn rerandomize_hidden(&mut self, observer: Seat, seed: u64, own_library: bool) {
        let mut rng = Pcg64::from_seed(seed);
        let opp = observer.other();
        let unknown = |s: &State, r: &ObjRef| s.knowledge[r.slot as usize].known_to & (1 << observer.idx()) == 0;
        let mut pools: Vec<Vec<u16>> = Vec::new();
        let mut p: Vec<u16> = Vec::new();
        for r in self.players[opp.idx()].hand.iter().chain(self.players[opp.idx()].library.iter()) {
            if unknown(self, r) {
                p.push(r.slot);
            }
        }
        pools.push(p);
        if own_library {
            let p: Vec<u16> = self.players[observer.idx()].library.iter().filter(|r| unknown(self, r)).map(|r| r.slot).collect();
            pools.push(p);
        }
        for pool in pools {
            let mut ids: Vec<(CardDefId, Option<CardId>)> = pool.iter().map(|&s| (self.objs[s as usize].def, self.objs[s as usize].card)).collect();
            rng.shuffle(&mut ids);
            for (s, (d, c)) in pool.iter().zip(ids) {
                self.objs[*s as usize].def = d;
                self.objs[*s as usize].card = c;
            }
        }
        self.rng = Pcg64::from_seed(seed ^ 0xDEAD_BEEF);
        self.derived_dirty = true;
    }

    /// Harness: an observer-indistinguishable twin. Shuffles the true order of `owner`'s library
    /// positions the observer does not know (what a different shuffle would have produced).
    #[cfg(feature = "diff-harness")]
    pub fn shuffle_unknown_library_order(&mut self, observer: Seat, owner: Seat, seed: u64) {
        let mut rng = Pcg64::from_seed(seed);
        let bit = 1u8 << observer.idx();
        let lib = self.players[owner.idx()].library.clone();
        let free: Vec<usize> = (0..lib.len()).filter(|&i| self.knowledge[lib[i].slot as usize].pos_known_to & bit == 0).collect();
        let mut objs: Vec<ObjRef> = free.iter().map(|&i| lib[i]).collect();
        rng.shuffle(&mut objs);
        let l = &mut self.players[owner.idx()].library;
        for (&i, r) in free.iter().zip(objs) {
            l[i] = r;
        }
    }

    /// Harness: perturbs one piece of internal state that could in principle affect the future
    /// (`what` 0..6). The hash must change with it, or the future must be unaffected. Returns false
    /// if the perturbation does not apply to this state.
    #[cfg(feature = "diff-harness")]
    pub fn poke_for_hash_test(&mut self, what: u8) -> bool {
        match what {
            0 => {
                let n = self.free.len();
                if n >= 2 {
                    self.free.swap(n - 1, n - 2);
                    true
                } else {
                    false
                }
            }
            1 => {
                self.ts_counter += 1;
                true
            }
            2 => match self.battlefield.first().copied() {
                Some(r) => {
                    self.moved.push(r);
                    true
                }
                None => false,
            },
            3 => {
                self.cfg.starting_life += 1;
                true
            }
            4 => {
                self.steps += 1;
                true
            }
            5 => {
                self.derived_dirty = !self.derived_dirty;
                true
            }
            _ => false,
        }
    }

    /// Harness: allocate objects until `live` are live (arena pressure tests).
    #[cfg(feature = "diff-harness")]
    pub fn fill_arena_for_test(&mut self, live: usize) {
        while self.live_objects() < live {
            self.alloc_slot(ObjKind::Token, CardDefId(0), Seat(0));
        }
        self.derived_dirty = true;
    }

    #[cfg(feature = "diff-harness")]
    pub fn frame_depth(&self) -> usize {
        self.frames.len()
    }
    #[cfg(feature = "diff-harness")]
    pub fn mark_derived_dirty(&mut self) {
        self.derived_dirty = true;
    }
    #[cfg(feature = "diff-harness")]
    pub fn derived_is_dirty(&self) -> bool {
        self.derived_dirty
    }
    #[cfg(feature = "diff-harness")]
    pub fn turn_mode_debug(&self) -> String {
        format!("{:?}", self.turn.mode)
    }
    #[cfg(feature = "diff-harness")]
    pub fn free_slots(&self) -> &[u16] {
        &self.free
    }
    #[cfg(feature = "diff-harness")]
    pub fn obj_raw_zone(&self, slot: u16) -> (ZoneKind, ObjKind, u16, Seat) {
        let o = &self.objs[slot as usize];
        (o.zone, o.kind, o.gen, o.owner)
    }
    #[cfg(feature = "diff-harness")]
    pub fn obj_counters_raw(&self, slot: u16) -> Vec<(CounterKind, u16)> {
        self.objs[slot as usize].counters.to_vec()
    }

    // ---- hashing (doc 01 section 4.4) ------------------------------------------------------------

    /// Hash over rules-relevant state. Excludes knowledge, view ids and the RNG (separate hashes),
    /// and scratch buffers.
    pub fn hash_rules(&self) -> u64 {
        // Every field is named so that a new `State` field is a compile error here: it must be
        // hashed or explicitly listed as not rules state.
        let State {
            objs, free, players, battlefield, stack, pending_triggers, delayed, attack_watch, links, until_links, cur_x, each_player, scripted, used,
            exile_plays, moved, name_choice, enter_choices, batch, effects, player_fx, exile, turn, combat, frames, pending, next_decision,
            ts_counter, result, cfg, pay_hint,
            // Not rules state: caches, knowledge (own hash), RNG (own hash), bookkeeping.
            derived: _, derived_dirty: _, knowledge: _, vids: _, next_vid: _, rng: _, n_cards: _, events: _, steps: _,
        } = self;
        debug_assert!(batch.depth == 0, "a simultaneous-change batch must not be open at a hash point");
        let mut h = Fx64::default();
        objs.hash(&mut h);
        free.hash(&mut h); // the order decides which slot the next object gets
        players.hash(&mut h);
        battlefield.hash(&mut h);
        stack.hash(&mut h);
        pending_triggers.hash(&mut h);
        enter_choices.hash(&mut h);
        name_choice.hash(&mut h);
        delayed.hash(&mut h);
        attack_watch.hash(&mut h);
        links.hash(&mut h);
        until_links.hash(&mut h);
        exile_plays.hash(&mut h);
        used.hash(&mut h);
        for p in players.iter() {
            p.dungeon.hash(&mut h);
            p.room.hash(&mut h);
            p.completed.hash(&mut h);
            p.emblems.hash(&mut h);
            p.ring.hash(&mut h);
            p.ring_bearer.hash(&mut h);
        }
        effects.hash(&mut h);
        player_fx.hash(&mut h);
        exile.hash(&mut h);
        turn.hash(&mut h);
        combat.hash(&mut h);
        frames.hash(&mut h);
        pending.hash(&mut h);
        next_decision.hash(&mut h);
        result.hash(&mut h);
        ts_counter.hash(&mut h);
        moved.hash(&mut h);
        cur_x.hash(&mut h);
        each_player.hash(&mut h);
        pay_hint.hash(&mut h);
        scripted.as_ref().map(|s| (s.queue.len(), s.violations)).hash(&mut h);
        (cfg.first_player, cfg.starting_life, cfg.hand_size, cfg.explicit_mana).hash(&mut h);
        h.finish()
    }

    pub fn hash_knowledge(&self) -> u64 {
        let mut h = Fx64::default();
        self.knowledge.hash(&mut h);
        self.vids.hash(&mut h);
        self.next_vid.hash(&mut h);
        h.finish()
    }

    pub fn hash_rng(&self) -> u64 {
        let mut h = Fx64::default();
        self.rng.hash(&mut h);
        h.finish()
    }

    /// Everything above combined: the golden-replay checkpoint hash.
    pub fn hash_full(&self) -> u64 {
        let mut h = Fx64::default();
        self.hash_rules().hash(&mut h);
        self.hash_knowledge().hash(&mut h);
        self.hash_rng().hash(&mut h);
        h.finish()
    }
}

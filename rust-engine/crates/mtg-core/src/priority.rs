//! Priority: enumerating legal actions and feeding the chosen one (doc 01 section 5.2, doc 02
//! section 2.2). Options are canonically ordered using only information visible to the decider
//! (doc 02 section 4.3), and equivalent options (identical cards in hand, interchangeable
//! permanents) are collapsed.

use crate::card::*;
use crate::cost::*;
use crate::cx::Cx;
use crate::decision::*;
use crate::engine::Next;
use crate::event::Event;
use crate::frame::*;
use crate::ids::*;
use crate::legal::*;
use crate::ops::MoveOpts;
use crate::state::*;
use crate::types::*;

impl<'a> Cx<'a> {
    fn can_play_sorcery_speed(&self, seat: Seat) -> bool {
        self.s.turn.active == seat && self.s.turn.step.is_main() && self.s.stack.is_empty()
    }

    /// Enumerates the legal priority options for `seat`, canonically ordered.
    pub fn priority_options(&mut self, seat: Seat) -> Vec<Opt> {
        self.refresh();
        let mut opts: Vec<(u8, u32, u64, Opt)> = vec![(0, 0, 0, Opt::Pass)];
        let sorcery_ok = self.can_play_sorcery_speed(seat);

        // Cards in hand, collapsed by definition (identical cards are interchangeable).
        let mut hand: Vec<ObjRef> = self.s.players[seat.idx()].hand.clone();
        hand.sort_by_key(|&r| self.obj_key(seat, r));
        let mut last_def: Option<CardDefId> = None;
        for r in hand {
            let def_id = self.s.obj(r).def;
            if last_def == Some(def_id) {
                continue;
            }
            last_def = Some(def_id);
            let def = self.db.def(def_id);
            for (i, a) in def.abilities.iter().enumerate() {
                if let AbilityDef::Activated(ad) = a {
                    if !ad.is_mana && ad.zone == ZoneKind::Hand && self.can_activate(seat, r, i as u8) {
                        let k = self.obj_key(seat, r);
                        opts.push((3, k.0, k.1 as u64 * 256 + i as u64, Opt::Activate { src: r, ability: i as u8 }));
                    }
                }
            }
            if def.types.contains(Types::LAND) {
                if sorcery_ok && self.s.players[seat.idx()].turn.lands_played < 1 {
                    let k = self.obj_key(seat, r);
                    opts.push((1, k.0, k.1 as u64, Opt::PlayLand(r)));
                }
                continue;
            }
            // A modal double-faced card whose back face is a land (pay 3 life or enter tapped).
            if def.back_kind == Some(FaceKind::Modal) && sorcery_ok && self.s.players[seat.idx()].turn.lands_played < 1 {
                if let Some(b) = def.back_id {
                    if self.db.def(b).types.contains(Types::LAND) {
                        let k = self.obj_key(seat, r);
                        opts.push((1, k.0, k.1 as u64 * 4 + 1, Opt::PlayLandBack(r, false)));
                        if self.s.players[seat.idx()].life >= 3 {
                            opts.push((1, k.0, k.1 as u64 * 4 + 2, Opt::PlayLandBack(r, true)));
                        }
                    }
                }
            }
            for way in self.castable_ways(seat, r) {
                let k = self.obj_key(seat, r);
                opts.push((2, k.0, k.1 as u64 * 256 + way as u64, Opt::Cast(r, way)));
            }
        }
        // Cards in other zones that have an alternative cost usable from there (flashback,
        // escape, warp recasts), collapsed by definition.
        for zone in [ZoneKind::Graveyard, ZoneKind::Exile] {
            let list: Vec<ObjRef> = match zone {
                ZoneKind::Graveyard => self.s.players[seat.idx()].graveyard.clone(),
                _ => self.s.exile.iter().copied().filter(|&r| self.s.obj(r).owner == seat).collect(),
            };
            let mut list = list;
            list.sort_by_key(|&r| self.obj_key(seat, r));
            let mut last: Option<CardDefId> = None;
            for r in list {
                let d = self.s.obj(r).def;
                if last == Some(d) {
                    continue;
                }
                last = Some(d);
                if !self.db.def(d).abilities.iter().any(|a| matches!(a, AbilityDef::Alt(x) if x.zone == zone)) {
                    continue;
                }
                for way in self.castable_ways(seat, r) {
                    let k = self.obj_key(seat, r);
                    opts.push((2, k.0, k.1 as u64 * 256 + way as u64, Opt::Cast(r, way)));
                }
            }
        }

        // Exiled cards the player may play (Runestone Caverns).
        let plays: Vec<ObjRef> = self.s.exile_plays.iter().filter(|(r, s)| *s == seat && self.s.is_live(*r) && self.s.obj(*r).zone == ZoneKind::Exile).map(|(r, _)| *r).collect();
        for r in plays {
            let def = self.db.def(self.s.obj(r).def);
            let k = self.obj_key(seat, r);
            if def.types.contains(Types::LAND) {
                if sorcery_ok && self.s.players[seat.idx()].turn.lands_played < 1 {
                    opts.push((1, k.0, k.1 as u64, Opt::PlayLand(r)));
                }
                continue;
            }
            for way in self.castable_ways(seat, r) {
                opts.push((2, k.0, k.1 as u64 * 256 + way as u64, Opt::Cast(r, way)));
            }
        }

        // Activated abilities of permanents, collapsed by (signature, ability).
        let bf: Vec<ObjRef> = self.s.battlefield.clone();
        let mut seen: Vec<(u64, u8)> = Vec::new();
        let mut cands: Vec<(ObjRef, u8)> = Vec::new();
        for r in bf {
            let o = self.s.obj(r);
            if o.controller != seat || o.phased {
                continue;
            }
            let def = self.db.def(o.def);
            for (i, a) in def.abilities.iter().enumerate() {
                if let AbilityDef::Activated(ad) = a {
                    if !ad.is_mana && ad.zone == ZoneKind::Battlefield {
                        cands.push((r, i as u8));
                    }
                }
            }
        }
        cands.sort_by_key(|&(r, i)| (self.obj_key(seat, r), i));
        for (r, i) in cands {
            let sig = self.obj_sig(r);
            if seen.contains(&(sig, i)) {
                continue;
            }
            if self.can_activate(seat, r, i) {
                seen.push((sig, i));
                let k = self.obj_key(seat, r);
                opts.push((3, k.0, k.1 as u64 * 256 + i as u64, Opt::Activate { src: r, ability: i }));
            }
        }
        for (r, i, col, pick) in self.mana_ability_options(seat) {
            let k = self.obj_key(seat, r);
            let pk = pick.map(|p| self.obj_key(seat, p).1 as u32 + 1).unwrap_or(0);
            opts.push((4, k.0, (k.1 as u64 * 4096 + i as u64 * 8 + col as u64) * 65536 + pk as u64, Opt::Mana { src: r, ability: i, color: col, pick }));
        }
        opts.sort_by_key(|&(a, b, c, _)| (a, b, c));
        opts.into_iter().map(|x| x.3).collect()
    }

    /// May `seat` play the exiled card `card` (an effect let them)?
    pub(crate) fn has_exile_play(&self, seat: Seat, card: ObjRef) -> bool {
        self.s.exile_plays.iter().any(|&(r, s)| r == card && s == seat)
    }

    /// Ways (0 = normal, i = i-th alternative cost) in which `seat` can cast `card` right now.
    pub fn castable_ways(&mut self, seat: Seat, card: ObjRef) -> Vec<u8> {
        let o = self.s.obj(card);
        let (zone, owner, def_id) = (o.zone, o.owner, o.def);
        if owner != seat {
            return Vec::new();
        }
        let db = self.db;
        let def = db.def(def_id);
        if def.types.contains(Types::LAND) || (def.cost.is_none() && alt_count(def) == 0) {
            return Vec::new();
        }
        let mut ways = Vec::new();
        if self.cast_restricted(seat, card) {
            return ways;
        }
        for way in 0..=alt_count(def) {
            let ok = match alt_of(def, way) {
                None => (zone == ZoneKind::Hand || (zone == ZoneKind::Exile && self.has_exile_play(seat, card))) && def.cost.is_some(),
                Some(a) => zone == a.zone,
            };
            if ok && self.way_feasible(seat, card, way) {
                ways.push(way);
            }
        }
        for way in self.permit_ways(seat, card) {
            if self.way_feasible(seat, card, way) {
                ways.push(way);
            }
        }
        // The Adventure half: judged as the instant or sorcery it is (its characteristics are
        // swapped in for the checks).
        if zone == ZoneKind::Hand && def.back_kind == Some(FaceKind::Adventure) {
            if let Some(b) = def.back_id {
                self.s.objs[card.slot as usize].def = b;
                let ok = !self.cast_restricted(seat, card) && self.way_feasible(seat, card, crate::restrict::ADVENTURE_WAY);
                self.s.objs[card.slot as usize].def = def_id;
                if ok {
                    ways.push(crate::restrict::ADVENTURE_WAY);
                }
            }
        }
        ways
    }

    /// Is every part of casting `card` by `way` (timing, condition, modes/targets, payment)
    /// completable, for some choice of the remaining decisions?
    pub fn way_feasible(&mut self, seat: Seat, card: ObjRef, way: u8) -> bool {
        self.way_feasible_x(seat, card, way, true)
    }

    /// `check_timing` false: an effect lets the card be cast now, whatever the phase.
    pub fn way_feasible_x(&mut self, seat: Seat, card: ObjRef, way: u8, check_timing: bool) -> bool {
        self.way_feasible_inner(seat, card, way, check_timing, true)
    }

    /// `check_cond` false: the way's own condition is not part of the question (a miracle's alternative
    /// cost carries a condition that only exists to keep the card from being cast normally this way).
    pub fn way_feasible_inner(&mut self, seat: Seat, card: ObjRef, way: u8, check_timing: bool, check_cond: bool) -> bool {
        let db = self.db;
        let def = db.def(self.s.obj(card).def);
        let instant_speed = !check_timing || def.types.contains(Types::INSTANT) || def.keywords.contains(Keywords::FLASH) || (way >= crate::restrict::VIRTUAL_WAY_BASE && way != crate::restrict::FREE_WAY && self.permit_flash(way));
        if !instant_speed && !self.can_play_sorcery_speed(seat) {
            return false;
        }
        if let Some(a) = alt_of(def, way) {
            if let Some(c) = a.cond.as_ref().filter(|_| check_cond) {
                let cap = Captured::default();
                let env = crate::eval::Env { controller: seat, source: Some(card), source_lki: None, targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
                if !self.eval_cond(&env, c) {
                    return false;
                }
            }
        }
        if let Some((_, sd)) = def.spell_def() {
            let feasible_modes = sd
                .modes
                .iter()
                .filter(|m| m.targets.iter().all(|spec| spec.optional || spec.x_count || self.legal_targets(spec, seat, Some(card)).into_iter().any(|t| t != Target::Obj(card))))
                .count();
            if feasible_modes < sd.choose.0.max(1) as usize {
                return false;
            }
        }
        // Payment: some additional-cost option (or none) must be affordable.
        let ctx0 = CastCtx { way, not_hand: self.s.obj(card).zone != ZoneKind::Hand, ..Default::default() };
        match add_cost_of(def) {
            None => self.affordable_with_best_phy(seat, card, None, &ctx0, [0; 6]),
            Some(ac) => {
                if ac.optional && self.affordable_with_best_phy(seat, card, None, &ctx0, [0; 6]) {
                    return true;
                }
                (0..ac.options.len()).any(|i| {
                    let mut c = ctx0.clone();
                    c.add_choice = i as u8 + 1;
                    self.affordable_with_best_phy(seat, card, None, &c, [0; 6])
                })
            }
        }
    }

    pub fn can_activate(&mut self, seat: Seat, src: ObjRef, idx: u8) -> bool {
        let db = self.db;
        let def = db.def(self.s.obj(src).def);
        let ad = match def.activated(idx) {
            Some(a) => a,
            None => return false,
        };
        if self.s.obj(src).zone == ZoneKind::Battlefield {
            self.refresh();
            if self.s.derived[src.slot as usize].lost {
                return false;
            }
        }
        if ad.timing == Timing::Sorcery && !self.can_play_sorcery_speed(seat) {
            return false;
        }
        if ad.cost.iter().any(|c| matches!(c, CostItem::Loyalty(_))) && (!self.can_play_sorcery_speed(seat) || self.s.used.contains(&(src, crate::cost::LOYALTY_USED))) {
            return false;
        }
        if self.s.obj(src).zone != ad.zone {
            return false;
        }
        let tap = ad.cost.iter().any(|c| matches!(c, CostItem::TapSelf));
        if tap && (self.s.obj(src).tapped || self.is_summoning_sick(src)) {
            return false;
        }
        if self.activation_blocked(src, ad.is_mana) {
            return false;
        }
        for spec in &ad.targets {
            if !spec.optional && !spec.x_count && self.legal_targets(spec, seat, Some(src)).is_empty() {
                return false;
            }
        }
        // "mana value X" targets: some affordable X must have a target.
        if ad.targets.iter().any(|s| !s.optional && s.objects.as_ref().map_or(false, |o| o.cmc_x)) {
            let mut ok = false;
            for x in 0..=20u8 {
                let mut c = CastCtx::default();
                c.x = x;
                if !self.affordable_with_best_phy(seat, src, Some(idx), &c, [0; 6]) {
                    break;
                }
                self.s.cur_x = Some(x);
                let has = ad.targets.iter().filter(|s| !s.optional && s.objects.as_ref().map_or(false, |o| o.cmc_x)).all(|sp| !self.legal_targets(sp, seat, Some(src)).is_empty());
                self.s.cur_x = None;
                if has {
                    ok = true;
                    break;
                }
            }
            return ok;
        }
        self.affordable_with_best_phy(seat, src, Some(idx), &CastCtx::default(), [0; 6])
    }

    /// Plays the back face of a modal double-faced card as a land.
    pub fn play_land_back(&mut self, seat: Seat, card: ObjRef, pay_life: bool) {
        if pay_life {
            self.lose_life(seat, 3);
        }
        let nr = self.move_zone(card, ZoneKind::Battlefield, MoveOpts { controller: Some(seat), face: true, tapped: !pay_life, ..Default::default() }).expect("land enters");
        self.s.players[seat.idx()].turn.lands_played += 1;
        self.emit(Event::LandPlayed { obj: nr, controller: seat });
    }

    pub fn play_land(&mut self, seat: Seat, card: ObjRef) {
        let nr = self.move_zone(card, ZoneKind::Battlefield, MoveOpts { controller: Some(seat), ..Default::default() }).expect("land enters");
        self.s.players[seat.idx()].turn.lands_played += 1;
        self.emit(Event::LandPlayed { obj: nr, controller: seat });
    }
}

/// Feed a priority-decision answer (the `Turn` frame is on top).
pub fn feed(cx: &mut Cx, p: &Pending, opt: Opt) -> Next {
    let seat = p.seat;
    debug_assert_eq!(cx.s.turn.mode, TurnMode::AwaitPriority);
    match opt {
        Opt::Pass => {
            cx.s.turn.passes += 1;
            if cx.s.turn.passes >= 2 {
                cx.s.turn.passes = 0;
                if cx.s.stack.is_empty() {
                    cx.s.turn.mode = TurnMode::End;
                    Next::Stay
                } else {
                    let top = cx.s.stack.last().unwrap().clone();
                    cx.s.turn.priority = cx.s.turn.active;
                    cx.s.turn.mode = TurnMode::Stabilize;
                    Next::Push(Frame::Resolve(ResolveFrame::new(top)))
                }
            } else {
                cx.s.turn.priority = seat.other();
                cx.s.turn.mode = TurnMode::Stabilize;
                Next::Stay
            }
        }
        Opt::PlayLand(card) => {
            cx.play_land(seat, card);
            cx.s.turn.passes = 0;
            cx.s.turn.mode = TurnMode::Stabilize;
            Next::Stay
        }
        Opt::PlayLandBack(card, pay_life) => {
            cx.play_land_back(seat, card, pay_life);
            cx.s.turn.passes = 0;
            cx.s.turn.mode = TurnMode::Stabilize;
            Next::Stay
        }
        Opt::Cast(card, way) => {
            cx.s.turn.passes = 0;
            cx.s.turn.mode = TurnMode::Stabilize;
            let from = cx.s.obj(card).zone;
            Next::Push(Frame::Cast(CastFrame::spell(card, seat, way, from)))
        }
        Opt::Activate { src, ability } => {
            cx.s.turn.passes = 0;
            cx.s.turn.mode = TurnMode::Stabilize;
            Next::Push(Frame::Cast(CastFrame::activation(src, seat, ability)))
        }
        Opt::Mana { src, ability, color, pick } => {
            // Any action, a mana ability included, restarts the "all players pass in succession"
            // count (CR 117.4).
            cx.run_mana_ability(seat, src, ability, color, pick);
            cx.s.turn.passes = 0;
            cx.s.turn.mode = TurnMode::Stabilize;
            Next::Stay
        }
        other => unreachable!("not a priority option: {other:?}"),
    }
}

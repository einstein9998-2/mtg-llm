//! Trigger detection (doc 01 section 7.2): events are matched against the triggered abilities of
//! objects in the zone each ability listens from, and matches wait in `pending_triggers` until the
//! next stabilization point puts them on the stack (`stabilize.rs`).

use crate::cx::Cx;
use crate::decision::{AttackTarget, Target, Targets};
use crate::eval::Env;
use crate::event::{Event, Lki};
use crate::frame::*;
use crate::ids::*;
use crate::ir::*;
use crate::state::*;
use crate::types::*;

impl<'a> Cx<'a> {
    /// Objects in `zone`, in a deterministic order.
    fn objs_in(&self, zone: ZoneKind) -> Vec<ObjRef> {
        match zone {
            ZoneKind::Battlefield => self.s.battlefield.clone(),
            ZoneKind::Stack => self.s.stack.iter().map(|e| e.obj).collect(),
            ZoneKind::Exile => self.s.exile.clone(),
            ZoneKind::Graveyard => (0..2).flat_map(|p| self.s.players[p].graveyard.iter().copied()).collect(),
            ZoneKind::Hand => (0..2).flat_map(|p| self.s.players[p].hand.iter().copied()).collect(),
            ZoneKind::Command => (0..2).flat_map(|p| self.s.players[p].emblems.iter().copied()).collect(),
            ZoneKind::Library | ZoneKind::Gone => Vec::new(),
        }
    }

    /// Matches `e` against every triggered ability that could care and queues the hits.
    pub(crate) fn match_triggers(&mut self, e: &Event) {
        // Only event kinds some pattern can name.
        if let Event::StepBegan { step } = e {
            let step = *step;
            let mut i = 0;
            while i < self.s.delayed.len() {
                let d = &self.s.delayed[i];
                let fire = match d.when {
                    DelayWhen::NextEndStep => step == Step::End,
                    DelayWhen::NextUpkeep => step == Step::Upkeep,
                    DelayWhen::EndCombat => step == Step::EndCombat,
                };
                if fire {
                    let d = self.s.delayed.remove(i);
                    self.s.pending_triggers.push(PendingTrigger { source: d.source, def: d.def, ability: d.ability, controller: d.controller, cap: d.cap });
                } else {
                    i += 1;
                }
            }
        }
        if let Event::Attacks { obj } = e {
            let (atk_ctrl, target) = (self.s.obj(*obj).controller, self.s.combat.as_ref().and_then(|cb| cb.attackers.iter().find(|a| a.obj == *obj).map(|a| a.target)));
            if let Some(target) = target {
                let defender = match target {
                    AttackTarget::Player(p) => p,
                    AttackTarget::Walker(w) => self.s.obj(w).controller,
                };
                let hits: Vec<AttackWatch> = self.s.attack_watch.iter().filter(|w| w.controller == defender && w.controller != atk_ctrl).cloned().collect();
                for w in hits {
                    let cap = Captured { obj: Some(*obj), player: Some(atk_ctrl), ..Captured::default() };
                    self.s.pending_triggers.push(PendingTrigger { source: w.source, def: w.def, ability: w.ability, controller: w.controller, cap });
                }
            }
        }
        let relevant = matches!(e, Event::ZoneChange { .. } | Event::SpellCast { .. } | Event::Drew { .. } | Event::StepBegan { .. } | Event::LandPlayed { .. } | Event::BecameTarget { .. } | Event::Blocked { .. } | Event::Attacks { .. } | Event::AttackersDeclared { .. } | Event::Damage { combat: true, .. });
        if !relevant || self.db.trig_zones == 0 {
            return;
        }
        self.refresh();
        for z in [ZoneKind::Battlefield, ZoneKind::Graveyard, ZoneKind::Hand, ZoneKind::Exile, ZoneKind::Stack] {
            if self.db.trig_zones & (1 << (z as u8)) == 0 {
                continue;
            }
            let mut cands = self.objs_in(z);
            // Simultaneous departures look back at one another (603.10a); permanents that entered
            // later in the same batch were not there to see it.
            let leave_batch = z == ZoneKind::Battlefield && matches!(e, Event::ZoneChange { from: ZoneKind::Battlefield, .. }) && !self.s.batch.departed.is_empty();
            if leave_batch {
                let pre = &self.s.batch.pre_bf;
                cands.retain(|r| pre.contains(r));
                for d in self.s.batch.departed.iter() {
                    if !cands.contains(&d.0) {
                        cands.push(d.0);
                    }
                }
            }
            // The object that just left `z` still gets to see its own departure (CR 603.10a).
            if let Event::ZoneChange { obj, from, .. } = e {
                if *from == z && !cands.contains(obj) {
                    cands.push(*obj);
                }
            }
            for src in cands {
                let departed = if leave_batch { self.s.batch.departed.iter().find(|d| d.0 == src).map(|d| (d.1, d.2.controller)) } else { None };
                let (def, live) = match self.s.objs.get(src.slot as usize) {
                    Some(o) if o.gen == src.gen => (departed.map(|d| d.0).unwrap_or(o.def), true),
                    _ => match (e, departed) {
                        (Event::ZoneChange { obj, def, .. }, _) if *obj == src => (*def, false),
                        (_, Some((def, _))) => (def, false),
                        _ => continue,
                    },
                };
                let cd = self.db.def(def);
                if live && z == ZoneKind::Battlefield && self.s.derived[src.slot as usize].lost {
                    continue;
                }
                for &ai in cd.trig.iter() {
                    let t = match &cd.abilities[ai as usize] {
                        AbilityDef::Triggered(t) => t,
                        _ => continue,
                    };
                    if t.zone != z {
                        continue;
                    }
                    if t.once_per_turn && self.s.used.contains(&(src, ai)) {
                        continue;
                    }
                    if t.batch && self.s.pending_triggers.iter().any(|p| p.source == src && p.ability == ai) {
                        continue;
                    }
                    if let Some((ctrl, cap)) = self.pattern_match(t, e, src, live, departed.map(|d| d.1)) {
                        if t.once_per_turn {
                            self.s.used.push((src, ai));
                        }
                        self.s.pending_triggers.push(PendingTrigger { source: src, def, ability: ai, controller: ctrl, cap });
                    }
                }
            }
        }
    }

    fn rel_ok(rel: Rel, who: Seat, ctrl: Seat) -> bool {
        match rel {
            Rel::Any => true,
            Rel::You => who == ctrl,
            Rel::Opp => who != ctrl,
        }
    }

    /// Does `e` trigger `t` on `src`? Returns the controller and captured event data.
    fn pattern_match(&mut self, t: &TriggeredDef, e: &Event, src: ObjRef, live: bool, src_ctrl: Option<Seat>) -> Option<(Seat, Captured)> {
        let moved_self = matches!(e, Event::ZoneChange { obj, .. } if *obj == src);
        let ctrl = match e {
            Event::ZoneChange { lki: Some(l), from, .. } if moved_self && *from == t.zone => l.controller,
            _ if src_ctrl.is_some() => src_ctrl.unwrap(),
            _ if live => self.s.obj(src).controller,
            _ => return None,
        };
        let mut cap = Captured::default();
        match (&t.on, e) {
            (EventPat::Enters { filter }, Event::ZoneChange { obj, to: ZoneKind::Battlefield, controller, .. }) => {
                if !self.s.is_live(*obj) || !self.matches_filter(filter, *obj, Some(src), ctrl) {
                    return None;
                }
                cap.obj = Some(*obj);
                cap.player = Some(*controller);
            }
            (EventPat::ZoneChange { filter, from, to }, Event::ZoneChange { obj, from: f, to: tz, controller, lki, .. }) => {
                if from.map(|x| x != *f).unwrap_or(false) || to.map(|x| x != *tz).unwrap_or(false) {
                    return None;
                }
                let ok = match lki {
                    Some(l) => self.lki_matches(filter, l, ctrl, *obj == src),
                    None => {
                        if !self.s.is_live(*obj) {
                            return None;
                        }
                        let c = self.chars(*obj);
                        self.filter_match(filter, *obj, &c, *obj == src, ctrl)
                    }
                };
                if !ok {
                    return None;
                }
                cap.obj = Some(*obj);
                cap.player = Some(*controller);
                cap.lki = lki.clone();
            }
            (EventPat::SpellCast { caster, filter, nth }, Event::SpellCast { obj, controller, .. }) => {
                if !Self::rel_ok(*caster, *controller, ctrl) || !self.s.is_live(*obj) {
                    return None;
                }
                if let Some(n) = nth {
                    if self.s.players[controller.idx()].turn.spells_cast != *n {
                        return None;
                    }
                }
                let c = self.chars(*obj);
                if !self.filter_match(filter, *obj, &c, *obj == src, ctrl) {
                    return None;
                }
                cap.obj = Some(*obj);
                cap.player = Some(*controller);
                // "That much" = the spells cast before this one this turn by anyone (storm).
                cap.amount = self.s.players.iter().map(|p| p.turn.spells_cast as i32).sum::<i32>() - 1;
            }
            (EventPat::LandPlayed { who, filter }, Event::LandPlayed { obj, controller }) => {
                if !Self::rel_ok(*who, *controller, ctrl) || !self.s.is_live(*obj) {
                    return None;
                }
                let c = self.chars(*obj);
                if !self.filter_match(filter, *obj, &c, *obj == src, ctrl) {
                    return None;
                }
                cap.obj = Some(*obj);
                cap.player = Some(*controller);
            }
            (EventPat::Attacks { filter }, Event::Attacks { obj }) => {
                if !self.s.is_live(*obj) {
                    return None;
                }
                let c = self.chars(*obj);
                if !self.filter_match(filter, *obj, &c, *obj == src, ctrl) {
                    return None;
                }
                cap.obj = Some(*obj);
                cap.player = Some(self.s.obj(*obj).controller);
            }
            (EventPat::CombatDamageToPlayer { filter }, Event::Damage { source: Some(s), to: Target::Player(p), amount, combat: true }) => {
                if *amount == 0 || !self.s.is_live(*s) {
                    return None;
                }
                let c = self.chars(*s);
                if !self.filter_match(filter, *s, &c, *s == src, ctrl) {
                    return None;
                }
                cap.obj = Some(*s);
                cap.player = Some(*p);
                cap.amount = *amount as i32;
            }
            (EventPat::BecomesBlocked { filter }, Event::Blocked { attacker, blocker }) => {
                if !self.s.is_live(*attacker) || !self.s.is_live(*blocker) {
                    return None;
                }
                let c = self.chars(*attacker);
                if !self.filter_match(filter, *attacker, &c, *attacker == src, ctrl) {
                    return None;
                }
                cap.obj = Some(*blocker);
                cap.player = Some(self.s.obj(*blocker).controller);
            }
            (EventPat::BecomesTarget { by }, Event::BecameTarget { obj, by: spell, by_ctrl }) => {
                if *obj != src || !Self::rel_ok(*by, *by_ctrl, ctrl) {
                    return None;
                }
                cap.obj = Some(*spell);
                cap.player = Some(*by_ctrl);
            }
            (EventPat::YouAttack, Event::AttackersDeclared { count }) => {
                if *count == 0 || self.s.turn.active != ctrl {
                    return None;
                }
                cap.player = Some(ctrl);
            }
            (EventPat::Draw { who, not_first_in_draw_step, self_only, first_this_turn }, Event::Drew { player, obj, .. }) => {
                if !Self::rel_ok(*who, *player, ctrl) {
                    return None;
                }
                if *self_only && *obj != src {
                    return None;
                }
                if *first_this_turn && self.s.players[player.idx()].turn.cards_drawn != 1 {
                    return None;
                }
                cap.obj = Some(*obj);
                if *not_first_in_draw_step && self.s.turn.step == Step::Draw && self.s.turn.active == *player && self.s.players[player.idx()].turn.draw_step_draws <= 1 {
                    return None;
                }
                cap.player = Some(*player);
            }
            (EventPat::BeginStep { step, whose }, Event::StepBegan { step: st }) => {
                let active = self.s.turn.active;
                if st != step || !Self::rel_ok(*whose, active, ctrl) {
                    return None;
                }
                cap.player = Some(active);
            }
            _ => return None,
        }
        if !live {
            // A token that ceased to exist: the ability looks back at what it was.
            if let Event::ZoneChange { lki: Some(l), .. } = e {
                cap.lki = Some(l.clone());
            }
        }
        if let Some(c) = &t.cond {
            let env = Env { controller: ctrl, source: Some(src), source_lki: cap.lki.as_ref().filter(|_| !live), targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
            if !self.eval_cond(&env, c) {
                return None;
            }
        }
        Some((ctrl, cap))
    }

    /// Registers a delayed trigger for the ability being resolved (CR 603.7).
    pub(crate) fn add_delayed(&mut self, env: &Env, when: DelayWhen, ability: u8, cap: Captured) {
        let source = match env.source {
            Some(s) => s,
            None => return,
        };
        // The resolving object is on top of the stack; its definition is the source card's even if
        // the source itself is gone (sacrificed as a cost, for instance).
        let def = self.s.stack.last().map(|e| e.def).unwrap_or(self.s.objs[source.slot as usize].def);
        self.s.delayed.push(DelayedTrig { when, source, def, ability, controller: env.controller, cap });
    }

    /// Puts an ability on the stack as a new object (CR 603.3, 602.2).
    pub(crate) fn push_ability(&mut self, source: ObjRef, ability: u8, def: CardDefId, controller: Seat, targets: Targets, cap: Captured) -> ObjRef {
        let nr = self.s.alloc_slot(ObjKind::Ability, def, controller);
        {
            let o = self.s.obj_mut_raw(nr);
            o.zone = ZoneKind::Stack;
            o.controller = controller;
        }
        let mut vid = [ViewId::NONE; 2];
        for v in 0..2 {
            vid[v] = ViewId(self.s.next_vid[v]);
            self.s.next_vid[v] += 1;
        }
        self.s.vids[nr.slot as usize] = vid;
        self.s.stack.push(StackEntry {
            obj: nr,
            controller,
            kind: StackKind::Ability { source, ability },
            targets,
            x: 0,
            def,
            modes: 1,
            cap,
            cast: CastInfo::default(),
        });
        self.emit_targeted_by(nr, controller);
        nr
    }

    fn emit_targeted_by(&mut self, by: ObjRef, ctrl: Seat) {
        crate::cast::emit_targeted(self, by, ctrl);
    }

    /// Does the triggered ability read anything about the event that caused it (the object, the
    /// player, the amount, the objects a delayed trigger captured)? If not, what the event was
    /// cannot change the outcome.
    fn ability_uses_event(&self, def: CardDefId, ability: u8) -> bool {
        match self.db.def(def).abilities.get(ability as usize) {
            Some(AbilityDef::Triggered(t)) => {
                let text = format!("{:?} {:?} {:?}", t.effect, t.targets, t.cond);
                ["EventObj", "EventPlayer", "EventAmount", "Captured"].iter().any(|k| text.contains(k))
            }
            _ => true,
        }
    }

    /// Two pending triggers are interchangeable if choosing between them can change nothing
    /// observable (doc 01 section 7.3).
    pub(crate) fn interchangeable(&self, a: &PendingTrigger, b: &PendingTrigger) -> bool {
        if a.def != b.def || a.ability != b.ability || a.controller != b.controller {
            return false;
        }
        if a.cap != b.cap && self.ability_uses_event(a.def, a.ability) {
            return false;
        }
        if a.source == b.source {
            return true;
        }
        if !self.s.is_live(a.source) || !self.s.is_live(b.source) {
            return false;
        }
        let (x, y) = (self.s.obj(a.source), self.s.obj(b.source));
        if x.zone != y.zone || x.chosen != y.chosen || self.keyed(a.source) || self.keyed(b.source) {
            return false;
        }
        if x.zone == ZoneKind::Battlefield {
            // Same rules-relevant state, derived characteristics included.
            return self.obj_sig(a.source) == self.obj_sig(b.source);
        }
        x.controller == y.controller && x.tapped == y.tapped && x.damage == y.damage && x.counters == y.counters && x.attached_to.is_none() && y.attached_to.is_none()
    }
}

#[allow(dead_code)]
fn _unused(_: &Lki, _: &StabFrame) {}

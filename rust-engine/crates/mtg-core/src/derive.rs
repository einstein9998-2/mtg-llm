//! Derived characteristics: layers and timestamps (doc 01 section 10).
//!
//! Layers needed by the pool: 4, 6, 7a, 7b, 7c. Dependency ordering (CR 613.8) is a scope cut
//! guarded by `deps-scan` (doc 01 section 10.4). Within a layer, effects apply in timestamp order.

use crate::card::*;
use crate::cx::Cx;
use crate::ids::*;
use crate::state::*;
use crate::types::*;

pub fn base_chars(def: &CardDef) -> Chars {
    let (p, t) = def.pt.map(|(p, t)| (p as i32, t as i32)).unwrap_or((0, 0));
    Chars {
        types: def.types,
        supertypes: def.supertypes,
        subtypes: def.subtypes,
        colors: def.colors,
        keywords: def.keywords,
        hexproof_from: Colors::empty(),
        power: p,
        toughness: t,
        lost: false,
        cant_attack: false,
    }
}

fn apply_cont(d: &mut Chars, e: &ContEffect) {
    match e {
        ContEffect::ModifyPT(p, t) => {
            d.power += *p as i32;
            d.toughness += *t as i32;
        }
        ContEffect::SetPT(p, t) => {
            d.power = *p as i32;
            d.toughness = *t as i32;
        }
        ContEffect::GrantKeywords(k) => d.keywords |= *k,
        ContEffect::AddTypes(ty) => d.types |= *ty,
        ContEffect::AddSubtypes(s) => d.subtypes = d.subtypes.union(*s),
        ContEffect::HexproofFrom(c) => d.hexproof_from |= *c,
        ContEffect::SetColors(c) => d.colors = *c,
        ContEffect::SetCreatureType(t) => {
            let keep = (1u128 << SUB_FIRST_CREATURE_TYPE) - 1;
            d.subtypes = SubtypeSet(d.subtypes.0 & keep).union(SubtypeSet::single(*t));
        }
        ContEffect::RemoveTypes(ty) => d.types.remove(*ty),
        ContEffect::SetTypes(ty, subs) => {
            d.types = *ty;
            d.subtypes = *subs;
        }
        ContEffect::BecomeBasicLand(sub) => {
            // Land types are the first seven entries of the subtype table.
            d.subtypes = SubtypeSet(d.subtypes.0 & !((1u128 << 7) - 1)).union(SubtypeSet::single(*sub));
            d.lost = true;
        }
        // Restrictions are read where the restricted action is offered, not in characteristics.
        ContEffect::CantAttack => d.cant_attack = true,
        ContEffect::SetPTExpr { .. } => {}
    }
}

impl<'a> Cx<'a> {
    /// Recomputes derived characteristics of all battlefield objects if the cache is dirty.
    pub fn refresh(&mut self) {
        if !self.s.derived_dirty {
            return;
        }
        self.recompute();
        self.s.derived_dirty = false;
    }

    fn recompute(&mut self) {
        let n = self.s.battlefield.len();
        // Base characteristics.
        for i in 0..n {
            let r = self.s.battlefield[i];
            let def = self.db.def(self.s.objs[r.slot as usize].def);
            self.s.derived[r.slot as usize] = base_chars(def);
        }
        // Collect statics: (layer, timestamp, source, def index within source def).
        let mut statics: smallvec::SmallVec<[(Layer, u32, ObjRef, usize); 8]> = smallvec::SmallVec::new();
        for i in 0..n {
            let r = self.s.battlefield[i];
            let o = &self.s.objs[r.slot as usize];
            if o.phased {
                continue;
            }
            let def = self.db.def(o.def);
            for (ai, a) in def.abilities.iter().enumerate() {
                if let AbilityDef::Static(sd) = a {
                    statics.push((sd.layer, o.timestamp, r, ai));
                }
            }
        }
        for p in 0..2usize {
            for &r in self.s.players[p].emblems.iter() {
                let o = &self.s.objs[r.slot as usize];
                let def = self.db.def(o.def);
                for (ai, a) in def.abilities.iter().enumerate() {
                    if let AbilityDef::Static(sd) = a {
                        statics.push((sd.layer, o.timestamp, r, ai));
                    }
                }
            }
        }
        statics.sort_by_key(|&(l, ts, r, ai)| (l, ts, r.slot, ai));
        let mut insts: Vec<usize> = (0..self.s.effects.len()).collect();
        insts.sort_by_key(|&i| (self.s.effects[i].layer, self.s.effects[i].ts));
        for layer in [Layer::L4, Layer::L6, Layer::L7a, Layer::L7b, Layer::L7c] {
            if layer == Layer::L4 {
                // The Ring's first level: your Ring-bearer is legendary.
                for p in 0..2usize {
                    if let Some(b) = self.s.players[p].ring_bearer {
                        if self.s.is_live(b) && self.s.objs[b.slot as usize].zone == ZoneKind::Battlefield && self.s.objs[b.slot as usize].controller.idx() == p {
                            self.s.derived[b.slot as usize].supertypes |= Supertypes::LEGENDARY;
                        }
                    }
                }
            }
            if layer == Layer::L6 {
                for i in 0..n {
                    let r = self.s.battlefield[i];
                    if self.s.objs[r.slot as usize].counter(CounterKind::Flying) > 0 {
                        self.s.derived[r.slot as usize].keywords |= Keywords::FLYING;
                    }
                }
            }
            if layer == Layer::L7c {
                // Counters are applied in 7c (their relative order is irrelevant: all additive).
                for i in 0..n {
                    let r = self.s.battlefield[i];
                    let o = &self.s.objs[r.slot as usize];
                    let plus = o.counter(CounterKind::PlusOne) as i32;
                    let minus = o.counter(CounterKind::MinusOne) as i32;
                    let d = &mut self.s.derived[r.slot as usize];
                    d.power += plus - minus;
                    d.toughness += plus - minus;
                }
            }
            for &ii in insts.iter() {
                let inst = self.s.effects[ii].clone();
                if inst.layer != layer {
                    continue;
                }
                for &t in inst.objs.iter() {
                    if !self.s.is_live(t) || self.s.objs[t.slot as usize].zone != ZoneKind::Battlefield {
                        continue;
                    }
                    apply_cont(&mut self.s.derived[t.slot as usize], &inst.effect);
                }
            }
            for &(l, _ts, src, ai) in statics.iter() {
                if l != layer {
                    continue;
                }
                let sd = match &self.db.def(self.s.objs[src.slot as usize].def).abilities[ai] {
                    AbilityDef::Static(sd) => sd.clone(),
                    _ => unreachable!(),
                };
                let src_ctrl = self.s.objs[src.slot as usize].controller;
                if self.s.objs[src.slot as usize].zone == ZoneKind::Battlefield && self.s.derived[src.slot as usize].lost {
                    continue;
                }
                if let Some(c) = &sd.cond {
                    let cap = Captured::default();
                    let env = crate::eval::Env { controller: src_ctrl, source: Some(src), source_lki: None, targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
                    if !self.eval_cond_static(&env, c) {
                        continue;
                    }
                }
                let eff = match &sd.effect {
                    ContEffect::SetPTExpr { x, t_plus } => {
                        let cap = Captured::default();
                        let env = crate::eval::Env { controller: src_ctrl, source: Some(src), source_lki: None, targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
                        let v = self.static_expr(&env, x) as i16;
                        ContEffect::SetPT(v, v + *t_plus)
                    }
                    other => other.clone(),
                };
                for i in 0..n {
                    let t = self.s.battlefield[i];
                    if self.matches_filter_bf(&sd.applies_to, t, src, src_ctrl) {
                        apply_cont(&mut self.s.derived[t.slot as usize], &eff);
                    }
                }
            }
        }
    }

    /// Would `obj` (about to enter the battlefield) have lost its abilities as it enters, given the
    /// continuous effects already on the battlefield (614.12)? Only type-changing statics matter.
    pub(crate) fn would_lose_abilities(&mut self, obj: ObjRef, ctrl: Seat) -> bool {
        self.refresh();
        let base = base_chars(self.db.def(self.s.objs[obj.slot as usize].def));
        let bf = self.s.battlefield.clone();
        for src in bf {
            let o = &self.s.objs[src.slot as usize];
            if o.phased || self.s.derived[src.slot as usize].lost {
                continue;
            }
            let src_ctrl = o.controller;
            for a in &self.db.def(o.def).abilities {
                if let AbilityDef::Static(sd) = a {
                    if matches!(sd.effect, ContEffect::BecomeBasicLand(_)) {
                        let mut f = sd.applies_to.clone();
                        f.controller = Rel::Any;
                        let _ = ctrl;
                        if self.filter_match(&f, obj, &base, false, src_ctrl) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Conditions of static abilities: only facts that do not depend on derived characteristics
    /// (graveyards, hand sizes, turn), since this runs inside `recompute`.
    fn eval_cond_static(&mut self, env: &crate::eval::Env, c: &Cond) -> bool {
        match c {
            Cond::True => true,
            Cond::Not(c) => !self.eval_cond_static(env, c),
            Cond::And(v) => v.iter().all(|c| self.eval_cond_static(env, c)),
            Cond::Or(v) => v.iter().any(|c| self.eval_cond_static(env, c)),
            Cond::IsActive(p) => self.eval_p(env, *p) == Some(self.s.turn.active),
            Cond::HasCounter(ORef::This, k) => env.source.map(|s| self.s.is_live(s) && self.s.obj(s).counter(*k) > 0).unwrap_or(false),
            Cond::Delirium(p) => match self.eval_p(env, *p) {
                Some(s) => self.graveyard_card_types(&[s]).bits().count_ones() >= 4,
                None => false,
            },
            Cond::Cmp(a, op, b) => {
                let (x, y) = (self.static_expr(env, a), self.static_expr(env, b));
                op.test(x, y)
            }
            other => panic!("condition {other:?} is not allowed in a static ability"),
        }
    }

    fn static_expr(&mut self, env: &crate::eval::Env, e: &Expr) -> i32 {
        match e {
            Expr::Const(n) => *n,
            Expr::GraveyardTypes(Some(p)) => self.eval_p(env, *p).map(|s| self.graveyard_card_types(&[s]).bits().count_ones() as i32).unwrap_or(0),
            Expr::GraveyardTypes(None) => self.graveyard_card_types(&[Seat(0), Seat(1)]).bits().count_ones() as i32,
            Expr::CardsInGraveyard(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].graveyard.len() as i32).unwrap_or(0),
            Expr::CardsInHand(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].hand.len() as i32).unwrap_or(0),
            Expr::Plus(a, b) => self.static_expr(env, a) + self.static_expr(env, b),
            other => panic!("expression {other:?} is not allowed in a static ability"),
        }
    }

    /// Filter match for a battlefield object using the (partially computed) derived chars.
    /// Filter match for a battlefield object using the (partially computed) derived chars.
    fn matches_filter_bf(&self, f: &ObjFilter, t: ObjRef, src: ObjRef, src_ctrl: Seat) -> bool {
        if f.equipped_by_source && self.s.objs[src.slot as usize].attached_to != Some(t) {
            return false;
        }
        let c = self.s.derived[t.slot as usize];
        self.filter_match(f, t, &c, t == src, src_ctrl)
    }

    /// Characteristics of any live object. Battlefield objects read the derived cache, which must
    /// be clean (call `refresh` first); other zones use printed characteristics.
    pub fn chars(&self, r: ObjRef) -> Chars {
        let o = self.s.obj(r);
        if o.zone == ZoneKind::Battlefield {
            debug_assert!(!self.s.derived_dirty, "stale derived cache read for {r:?}");
            self.s.derived[r.slot as usize]
        } else {
            base_chars(self.db.def(o.def))
        }
    }

    /// Does `t` match `f` from the viewpoint of `ctrl` (the controller of the spell/ability)?
    pub fn matches_filter(&self, f: &ObjFilter, t: ObjRef, source: Option<ObjRef>, ctrl: Seat) -> bool {
        if !self.s.is_live(t) {
            return false;
        }
        if self.s.obj(t).zone != f.zone {
            return false;
        }
        if f.equipped_by_source && source.map(|s| self.s.is_live(s) && self.s.obj(s).attached_to != Some(t)).unwrap_or(true) {
            return false;
        }
        let c = self.chars(t);
        self.filter_match(f, t, &c, Some(t) == source, ctrl)
    }

    /// The zone-independent part of filter matching, given characteristics.
    pub fn filter_match(&self, f: &ObjFilter, t: ObjRef, c: &Chars, is_source: bool, ctrl: Seat) -> bool {
        self.filter_match_one(f, t, c, is_source, ctrl) || f.alt.iter().any(|a| self.filter_match_one(a, t, c, is_source, ctrl))
    }

    fn filter_match_one(&self, f: &ObjFilter, t: ObjRef, c: &Chars, is_source: bool, ctrl: Seat) -> bool {
        let o = &self.s.objs[t.slot as usize];
        if (f.not_self && is_source) || (f.self_only && !is_source) {
            return false;
        }
        if !f.types_any.is_empty() && !c.types.intersects(f.types_any) {
            return false;
        }
        if c.types.intersects(f.types_not) {
            return false;
        }
        if !f.supertypes_any.is_empty() && !c.supertypes.intersects(f.supertypes_any) {
            return false;
        }
        if c.supertypes.intersects(f.supertypes_not) {
            return false;
        }
        if !f.colors_any.is_empty() && !c.colors.intersects(f.colors_any) {
            return false;
        }
        if c.colors.intersects(f.colors_not) {
            return false;
        }
        if let Some(cl) = f.colorless {
            if c.colors.is_empty() != cl {
                return false;
            }
        }
        if !f.keywords_any.is_empty() && !c.keywords.intersects(f.keywords_any) {
            return false;
        }
        match f.controller {
            Rel::Any => {}
            Rel::You => {
                if o.controller != ctrl {
                    return false;
                }
            }
            Rel::Opp => {
                if o.controller == ctrl {
                    return false;
                }
            }
        }
        match f.owner {
            Rel::Any => {}
            Rel::You => {
                if o.owner != ctrl {
                    return false;
                }
            }
            Rel::Opp => {
                if o.owner == ctrl {
                    return false;
                }
            }
        }
        if f.subtypes_any != SubtypeSet::EMPTY && !c.subtypes.intersects(f.subtypes_any) {
            return false;
        }
        if c.subtypes.intersects(f.subtypes_not) {
            return false;
        }
        if let Some(e) = f.entered_this_turn {
            if (o.entered_turn == self.s.turn.number) != e {
                return false;
            }
        }
        if let Some(tok) = f.token {
            if (o.kind == ObjKind::Token) != tok {
                return false;
            }
        }
        if let Some(tp) = f.tapped {
            if o.tapped != tp {
                return false;
            }
        }
        if let Some(x) = f.has_x {
            if self.db.def(o.def).cost.map(|c| c.x > 0).unwrap_or(false) != x {
                return false;
            }
        }
        if f.cmc_x {
            if let Some(x) = self.s.cur_x {
                if self.db.def(o.def).mana_value() as i32 != x as i32 {
                    return false;
                }
            }
        }
        if let Some((cmp, n)) = f.cmc {
            if !cmp.test(self.db.def(o.def).mana_value() as i32, n) {
                return false;
            }
        }
        if let Some((cmp, n)) = f.power {
            if !cmp.test(c.power, n) {
                return false;
            }
        }
        if let Some((cmp, n)) = f.toughness {
            if !cmp.test(c.toughness, n) {
                return false;
            }
        }
        if !f.names.is_empty() {
            let name = &self.db.def(o.def).name;
            if !f.names.iter().any(|n| n == name) {
                return false;
            }
        }
        if let Some(a) = f.attacking {
            let is_att = self.s.combat.as_ref().map(|cb| cb.attackers.iter().any(|x| x.obj == t)).unwrap_or(false);
            if is_att != a {
                return false;
            }
        }
        if let Some(b) = f.from_bf_this_turn {
            if (o.entered_turn == self.s.turn.number && o.prev_zone == ZoneKind::Battlefield) != b {
                return false;
            }
        }
        if f.ring_bearer && self.s.players[o.controller.idx()].ring_bearer != Some(t) {
            return false;
        }
        if let Some(u) = f.unblocked {
            let declared = matches!(self.s.turn.step, Step::DeclareBlockers | Step::FirstStrikeDamage | Step::CombatDamage | Step::EndCombat);
            let is_unb = declared && self.s.combat.as_ref().map(|cb| cb.attackers.iter().any(|x| x.obj == t && !x.blocked)).unwrap_or(false);
            if is_unb != u {
                return false;
            }
        }
        if let Some(sel) = f.stack {
            let is_spell = matches!(o.kind, ObjKind::Card | ObjKind::SpellCopy | ObjKind::Token);
            let ok = match sel {
                StackSel::Spell => is_spell,
                StackSel::Ability => !is_spell,
                StackSel::Triggered => !is_spell && self.s.stack.iter().any(|e| e.obj == t && matches!(e.kind, StackKind::Ability { ability, .. } if matches!(self.db.def(e.def).abilities.get(ability as usize), Some(AbilityDef::Triggered(_))))),
            };
            if !ok {
                return false;
            }
        }
        if let Some(k) = f.has_counter {
            if o.counter(k) == 0 {
                return false;
            }
        }
        true
    }
}

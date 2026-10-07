//! Evaluation of references, values, conditions and selectors against the game state (the
//! read-only half of the effect VM).

use crate::cx::Cx;
use crate::decision::Target;
use crate::event::Lki;
use crate::ids::*;
use crate::ir::*;
use crate::state::*;
use crate::types::*;

/// Everything an expression needs to know about "the ability being evaluated".
#[derive(Clone, Debug)]
pub struct Env<'a> {
    pub controller: Seat,
    /// The source object: the spell itself, or the permanent that the ability came from (may be
    /// stale if it left the battlefield).
    pub source: Option<ObjRef>,
    pub source_lki: Option<&'a Lki>,
    pub targets: &'a [Target],
    pub legal: &'a [bool],
    /// Offset of the running mode's slots in `targets`/`legal`.
    pub base: usize,
    pub x: i32,
    pub cap: &'a Captured,
    pub each: [Option<ObjRef>; 2],
    /// Last-known information of the object targets (parallel to `targets`).
    pub tlki: &'a [Option<Lki>],
}

impl<'a> Env<'a> {
    pub fn target(&self, slot: u8) -> Option<Target> {
        let i = self.base + slot as usize;
        if self.legal.get(i).copied().unwrap_or(false) {
            self.targets.get(i).copied()
        } else {
            None
        }
    }
}

impl<'a> Cx<'a> {
    pub fn eval_p(&self, env: &Env, p: PRef) -> Option<Seat> {
        match p {
            PRef::You => Some(env.controller),
            PRef::Opp => Some(env.controller.other()),
            PRef::Active => Some(self.s.turn.active),
            PRef::Target(i) => match env.target(i) {
                Some(Target::Player(s)) => Some(s),
                _ => None,
            },
            PRef::ControllerOf(o) => self.obj_ctrl_owner(env, o, true),
            PRef::OwnerOf(o) => self.obj_ctrl_owner(env, o, false),
            PRef::EventPlayer => env.cap.player,
            PRef::Each => self.s.each_player,
        }
    }

    fn obj_ctrl_owner(&self, env: &Env, o: ORef, ctrl: bool) -> Option<Seat> {
        if let Some(r) = self.eval_o(env, o) {
            let ob = self.s.obj(r);
            return Some(if ctrl { ob.controller } else { ob.owner });
        }
        let lki = self.lki_of(env, o)?;
        Some(if ctrl { lki.controller } else { lki.owner })
    }

    /// Live object for a reference, if it still exists in the zone it had.
    pub fn eval_o(&self, env: &Env, o: ORef) -> Option<ObjRef> {
        let r = match o {
            ORef::This => env.source,
            ORef::Target(i) => match env.target(i) {
                Some(Target::Obj(r)) => Some(r),
                _ => None,
            },
            ORef::Each(d) => env.each.get(d as usize).copied().flatten(),
            ORef::EventObj => env.cap.obj,
            ORef::Moved(i) => self.s.moved.get(i as usize).copied(),
            ORef::AttachedTo => match env.source {
                Some(s) if self.s.is_live(s) && self.s.obj(s).zone == ZoneKind::Battlefield => self.s.obj(s).attached_to,
                _ => env.source_lki.and_then(|l| l.attached),
            },
        }?;
        self.s.is_live(r).then_some(r)
    }

    /// Last-known information for a reference whose object may have left.
    pub fn lki_of(&self, env: &Env, o: ORef) -> Option<Lki> {
        match o {
            ORef::This => env.source_lki.cloned(),
            ORef::EventObj => env.cap.lki.clone(),
            ORef::Target(i) => env.tlki.get(env.base + i as usize).cloned().flatten(),
            _ => None,
        }
    }

    pub fn eval_expr(&mut self, env: &Env, e: &Expr) -> i32 {
        match e {
            Expr::Const(n) => *n,
            Expr::X => env.x,
            Expr::EventAmount => env.cap.amount,
            Expr::Count(f) => self.select(env, f).len() as i32,
            Expr::Life(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].life).unwrap_or(0),
            Expr::CardsInHand(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].hand.len() as i32).unwrap_or(0),
            Expr::CardsInLibrary(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].library.len() as i32).unwrap_or(0),
            Expr::CardsInGraveyard(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].graveyard.len() as i32).unwrap_or(0),
            Expr::SpellsCastThisTurn(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].turn.spells_cast as i32).unwrap_or(0),
            Expr::LifeLostThisTurn(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].turn.life_lost as i32).unwrap_or(0),
            Expr::CardsDrawnThisTurn(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].turn.cards_drawn as i32).unwrap_or(0),
            Expr::RingLevel(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].ring as i32).unwrap_or(0),
            Expr::LifeGainedThisTurn(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].turn.life_gained as i32).unwrap_or(0),
            Expr::PowerOf(o) => {
                self.refresh();
                match self.eval_o(env, *o) {
                    Some(r) => self.chars(r).power,
                    None => self.lki_of(env, *o).map(|l| l.power).unwrap_or(0),
                }
            }
            Expr::ToughnessOf(o) => {
                self.refresh();
                match self.eval_o(env, *o) {
                    Some(r) => self.chars(r).toughness,
                    None => self.lki_of(env, *o).map(|l| l.toughness).unwrap_or(0),
                }
            }
            Expr::CmcOf(o) => match self.eval_o(env, *o) {
                Some(r) => self.db.def(self.s.obj(r).def).mana_value() as i32,
                None => self.lki_of(env, *o).map(|l| self.db.def(l.def).mana_value() as i32).unwrap_or(0),
            },
            Expr::ManaSpent(o) => match self.eval_o(env, *o) {
                Some(r) => self.s.stack.iter().find(|e| e.obj == r).map(|e| e.cast.mana_spent as i32).unwrap_or(-1),
                None => -1,
            },
            Expr::ColorsSpent(o) => match self.eval_o(env, *o) {
                Some(r) => self.s.stack.iter().find(|e| e.obj == r).map(|e| e.cast.colors_spent.count_ones() as i32).unwrap_or(0),
                None => 0,
            },
            Expr::Energy(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].energy as i32).unwrap_or(0),
            Expr::Devotion(colors, p) => {
                let seat = match self.eval_p(env, *p) {
                    Some(s) => s,
                    None => return 0,
                };
                let mut n = 0i32;
                for &r in self.s.battlefield.iter() {
                    let o = self.s.obj(r);
                    if o.controller != seat {
                        continue;
                    }
                    if let Some(c) = self.db.def(o.def).cost {
                        for (ci, col) in Colors::ALL.iter().enumerate() {
                            if !colors.intersects(*col) {
                                continue;
                            }
                            n += c.pips[ci] as i32 + c.phy[ci] as i32;
                            for (hi, &(a, b)) in crate::mana::HYBRID_PAIRS.iter().enumerate() {
                                if a == ci || b == ci {
                                    n += c.hyb[hi] as i32;
                                }
                            }
                        }
                    }
                }
                n
            }
            Expr::Replicated(o) => match self.eval_o(env, *o) {
                Some(r) => self.s.stack.iter().find(|e| e.obj == r).map(|e| e.cast.replicate as i32).unwrap_or(0),
                None => 0,
            },
            Expr::Counters(o, k) => match self.eval_o(env, *o) {
                Some(r) => self.s.obj(r).counter(*k) as i32,
                None => self.lki_of(env, *o).and_then(|l| l.counters.iter().find(|c| c.0 == *k).map(|c| c.1 as i32)).unwrap_or(0),
            },
            Expr::GraveyardTypes(p) => {
                let mut seats: Vec<Seat> = Vec::new();
                match p {
                    Some(p) => seats.extend(self.eval_p(env, *p)),
                    None => seats.extend([Seat(0), Seat(1)]),
                }
                self.graveyard_card_types(&seats).bits().count_ones() as i32
            }
            Expr::Plus(a, b) => self.eval_expr(env, a) + self.eval_expr(env, b),
            Expr::Minus(a, b) => self.eval_expr(env, a) - self.eval_expr(env, b),
            Expr::Times(a, b) => self.eval_expr(env, a) * self.eval_expr(env, b),
            Expr::Max(a, b) => self.eval_expr(env, a).max(self.eval_expr(env, b)),
            Expr::Min(a, b) => self.eval_expr(env, a).min(self.eval_expr(env, b)),
            Expr::HalfDown(a) => self.eval_expr(env, a).div_euclid(2),
            Expr::HalfUp(a) => (self.eval_expr(env, a) + 1).div_euclid(2),
        }
    }

    /// Union of the card types of the cards in the given players' graveyards.
    /// Card types among `cards` (the types that count for escape and the */1+* creatures).
    pub fn types_of_cards(&self, cards: &[ObjRef]) -> Types {
        let mut t = Types::empty();
        for &r in cards {
            t |= self.db.def(self.s.obj(r).def).types;
        }
        t & (Types::LAND | Types::CREATURE | Types::ARTIFACT | Types::ENCHANTMENT | Types::PLANESWALKER | Types::INSTANT | Types::SORCERY | Types::KINDRED)
    }

    pub fn graveyard_card_types(&self, seats: &[Seat]) -> Types {
        let mut t = Types::empty();
        for s in seats {
            for &r in &self.s.players[s.idx()].graveyard {
                t |= self.db.def(self.s.obj(r).def).types;
            }
        }
        t & (Types::LAND | Types::CREATURE | Types::ARTIFACT | Types::ENCHANTMENT | Types::PLANESWALKER | Types::INSTANT | Types::SORCERY | Types::KINDRED)
    }

    pub fn eval_cond(&mut self, env: &Env, c: &Cond) -> bool {
        match c {
            Cond::True => true,
            Cond::Not(c) => !self.eval_cond(env, c),
            Cond::And(v) => v.iter().all(|c| self.eval_cond(env, c)),
            Cond::Or(v) => v.iter().any(|c| self.eval_cond(env, c)),
            Cond::Cmp(a, op, b) => {
                let (x, y) = (self.eval_expr(env, a), self.eval_expr(env, b));
                op.test(x, y)
            }
            Cond::IsActive(p) => self.eval_p(env, *p) == Some(self.s.turn.active),
            Cond::UsedThisTurn(i) => match env.source {
                Some(src) => self.s.used.contains(&(src, *i)),
                None => false,
            },
            Cond::HasCounter(o, k) => match self.eval_o(env, *o) {
                Some(r) => self.s.obj(r).counter(*k) > 0,
                None => false,
            },
            Cond::Completed { who, dungeon } => match (self.eval_p(env, *who), self.db.id(dungeon)) {
                (Some(p), Some(d)) => self.s.players[p.idx()].completed.contains(&d),
                _ => false,
            },
            Cond::Controls { who, filter, at_least } => {
                let who = match self.eval_p(env, *who) {
                    Some(w) => w,
                    None => return false,
                };
                let mut f = filter.clone();
                f.controller = Rel::Any;
                let objs = self.select(env, &f);
                objs.iter().filter(|&&r| self.s.obj(r).controller == who).count() >= *at_least as usize
            }
            Cond::Matches(o, f) => {
                self.refresh();
                match self.eval_o(env, *o) {
                    Some(r) => {
                        let ctrl = env.controller;
                        let c = self.chars(r);
                        let mut f2 = f.clone();
                        f2.zone = self.s.obj(r).zone;
                        self.filter_match(&f2, r, &c, Some(r) == env.source, ctrl)
                    }
                    None => match self.lki_of(env, *o) {
                        Some(l) => self.lki_matches(f, &l, env.controller, *o == ORef::This),
                        None => false,
                    },
                }
            }
            Cond::WasCast { from, key } => {
                let r = match env.source {
                    Some(r) if self.s.is_live(r) => r,
                    _ => return false,
                };
                let o = self.s.obj(r);
                // A spell still on the stack (an instant or sorcery resolving: Orim's Chant's kicker,
                // RFC 0005) keeps how it was cast on its stack entry; a permanent keeps it on itself.
                let (cast_from, cast_way) = match self.s.stack.iter().find(|e| e.obj == r) {
                    Some(e) => (e.cast.from, e.cast.way),
                    None => (o.cast_from, o.cast_way),
                };
                if cast_from == ZoneKind::Gone {
                    return false;
                }
                if from.map(|z| z != cast_from).unwrap_or(false) {
                    return false;
                }
                match key {
                    None => true,
                    Some(k) => {
                        let def = self.db.def(o.def);
                        match crate::cost::alt_of(def, cast_way) {
                            None => k.is_empty(),
                            Some(a) => &a.key == k,
                        }
                    }
                }
            }
            Cond::ControlledBy(o, p) => match (self.eval_o(env, *o), self.eval_p(env, *p)) {
                (Some(r), Some(s)) => self.s.obj(r).zone == ZoneKind::Battlefield && self.s.obj(r).controller == s,
                _ => false,
            },
            Cond::CastColor { who, colors } => self.eval_p(env, *who).map(|s| (self.s.players[s.idx()].turn.cast_colors as u32) & colors.bits() as u32 != 0).unwrap_or(false),
            Cond::Blessing(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].blessing).unwrap_or(false),
            Cond::Revolt(p) => self.eval_p(env, *p).map(|s| self.s.players[s.idx()].turn.perm_left).unwrap_or(false),
            Cond::Delirium(p) => match self.eval_p(env, *p) {
                Some(s) => self.graveyard_card_types(&[s]).bits().count_ones() >= 4,
                None => false,
            },
        }
    }

    /// Filter match against last-known information (only the characteristics LKI carries).
    pub fn lki_matches(&self, f: &ObjFilter, l: &Lki, ctrl: Seat, is_self: bool) -> bool {
        self.lki_match_one(f, l, ctrl, is_self) || f.alt.iter().any(|a| self.lki_match_one(a, l, ctrl, is_self))
    }

    /// Every characteristic test of `filter_match_one` that a last-known-information snapshot can
    /// answer (the snapshot has no tapped/attacking/counter state of a live object).
    fn lki_match_one(&self, f: &ObjFilter, l: &Lki, ctrl: Seat, is_self: bool) -> bool {
        if (f.not_self && is_self) || (f.self_only && !is_self) {
            return false;
        }
        if !f.types_any.is_empty() && !l.types.intersects(f.types_any) {
            return false;
        }
        if l.types.intersects(f.types_not) {
            return false;
        }
        if !f.supertypes_any.is_empty() && !l.supertypes.intersects(f.supertypes_any) {
            return false;
        }
        if l.supertypes.intersects(f.supertypes_not) {
            return false;
        }
        if f.subtypes_any != SubtypeSet::EMPTY && !l.subtypes.intersects(f.subtypes_any) {
            return false;
        }
        if l.subtypes.intersects(f.subtypes_not) {
            return false;
        }
        if !f.colors_any.is_empty() && !l.colors.intersects(f.colors_any) {
            return false;
        }
        if l.colors.intersects(f.colors_not) {
            return false;
        }
        if let Some(cl) = f.colorless {
            if l.colors.is_empty() != cl {
                return false;
            }
        }
        if !f.keywords_any.is_empty() && !l.keywords.intersects(f.keywords_any) {
            return false;
        }
        if let Some((cmp, n)) = f.cmc {
            if !cmp.test(self.db.def(l.def).mana_value() as i32, n) {
                return false;
            }
        }
        if let Some((cmp, n)) = f.power {
            if !cmp.test(l.power, n) {
                return false;
            }
        }
        if let Some((cmp, n)) = f.toughness {
            if !cmp.test(l.toughness, n) {
                return false;
            }
        }
        match f.controller {
            Rel::Any => {}
            Rel::You => {
                if l.controller != ctrl {
                    return false;
                }
            }
            Rel::Opp => {
                if l.controller == ctrl {
                    return false;
                }
            }
        }
        match f.owner {
            Rel::Any => {}
            Rel::You => {
                if l.owner != ctrl {
                    return false;
                }
            }
            Rel::Opp => {
                if l.owner == ctrl {
                    return false;
                }
            }
        }
        if let Some(tok) = f.token {
            if l.was_token != tok {
                return false;
            }
        }
        if !f.names.is_empty() && !f.names.iter().any(|n| *n == self.db.def(l.def).name) {
            return false;
        }
        true
    }

    /// All objects currently matching `f` (zone taken from the filter), in a deterministic order:
    /// zone list order (battlefield order = entry order).
    pub fn select(&mut self, env: &Env, f: &ObjFilter) -> Vec<ObjRef> {
        self.refresh();
        let mut cands: Vec<ObjRef> = Vec::new();
        match f.zone {
            ZoneKind::Battlefield => cands.extend(self.s.battlefield.iter().copied()),
            ZoneKind::Stack => cands.extend(self.s.stack.iter().map(|e| e.obj)),
            ZoneKind::Graveyard => {
                for p in 0..2 {
                    cands.extend(self.s.players[p].graveyard.iter().copied());
                }
            }
            ZoneKind::Hand => {
                for p in 0..2 {
                    cands.extend(self.s.players[p].hand.iter().copied());
                }
            }
            ZoneKind::Library => {
                for p in 0..2 {
                    cands.extend(self.s.players[p].library.iter().copied());
                }
            }
            ZoneKind::Exile => cands.extend(self.s.exile.iter().copied()),
            ZoneKind::Command | ZoneKind::Gone => {}
        }
        cands.retain(|&r| self.matches_filter(f, r, env.source, env.controller));
        cands
    }
}

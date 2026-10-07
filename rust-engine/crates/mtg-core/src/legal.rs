//! Legality helpers shared by the enumerator and the procedures: mana sources, targets, timing.
//! Legality is decided in one place and re-checked on apply (doc 01 R1).

use crate::card::*;
use crate::cx::Cx;
use crate::decision::*;
use crate::hash::Fx64;
use crate::ids::*;
use crate::mana::*;
use crate::ops::MoveOpts;
use crate::state::ObjKind;
use crate::types::*;
use smallvec::SmallVec;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct SrcInfo {
    pub obj: ObjRef,
    pub colors: u8,
    pub pref: u8,
    /// Mana produced by one tap (Ancient Tomb: 2).
    pub n: u8,
    /// Index of the mana ability used (255 = the intrinsic ability of a basic land type).
    pub ability: u8,
    /// The ability also sacrifices a permanent (Lazotep Quarry); automatic payment picks the
    /// cheapest one that is not itself a mana source of this payment (see `sac_pick`).
    pub sac: bool,
}

impl<'a> Cx<'a> {
    /// Summoning sickness (CR 302.6): controlled continuously since the controller's most recent
    /// turn began. `entered_turn` is the turn number control was gained.
    pub fn is_summoning_sick(&self, r: ObjRef) -> bool {
        let o = self.s.obj(r);
        let c = self.chars(r);
        c.types.contains(Types::CREATURE) && !c.keywords.contains(Keywords::HASTE) && o.entered_turn >= self.s.turn.own_turn[o.controller.idx()]
    }

    /// Permanents `seat` could tap for mana right now (excluding `except`), as sources for the
    /// automatic payment: basic-type lands and simple mana abilities (tap, optionally sacrifice).
    pub fn mana_sources(&mut self, seat: Seat, except: Option<ObjRef>) -> SmallVec<[SrcInfo; 16]> {
        self.mana_sources_ex(seat, except, 0)
    }

    /// The first `sac_count` permanents with a sacrifice mana ability (Lazotep Quarry) offer it
    /// (any color, one tap, costs a creature) instead of their other modes; `plan_cost` raises
    /// the count only when fewer sacrifices cannot pay.
    pub fn mana_sources_ex(&mut self, seat: Seat, except: Option<ObjRef>, sac_count: usize) -> SmallVec<[SrcInfo; 16]> {
        self.refresh();
        let mut sac_left = sac_count;
        let mut out: SmallVec<[SrcInfo; 16]> = SmallVec::new();
        let bf: SmallVec<[ObjRef; 24]> = self.s.battlefield.iter().copied().collect();
        for r in bf {
            if Some(r) == except {
                continue;
            }
            {
                let o = &self.s.objs[r.slot as usize];
                if o.controller != seat || o.tapped || o.phased {
                    continue;
                }
            }
            if self.db.has_restrict && self.activation_blocked(r, true) {
                continue;
            }
            let o = &self.s.objs[r.slot as usize];
            let c = self.s.derived[r.slot as usize];
            let mut mask = 0u8;
            let mut n = 1u8;
            let mut ability = 255u8;
            let mut consumes = false;
            let mut sac = false;
            let mut sac_ab: Option<u8> = None;
            if c.types.contains(Types::LAND) {
                for (st, mc) in [(SUB_PLAINS, ManaColor::W), (SUB_ISLAND, ManaColor::U), (SUB_SWAMP, ManaColor::B), (SUB_MOUNTAIN, ManaColor::R), (SUB_FOREST, ManaColor::G)] {
                    if c.subtypes.has(st) {
                        mask |= 1 << mc.idx();
                    }
                }
            }
            let def = self.db.def(o.def);
            let mut extra = 0u8;
            for (i, a) in def.abilities.iter().enumerate().filter(|_| !c.lost) {
                if let AbilityDef::Activated(ad) = a {
                    if ad.is_mana {
                        if let Some(m) = simple_mana_ability(ad) {
                            mask |= m.colors;
                            if m.n > n || ability == 255 {
                                n = m.n.max(n);
                                ability = i as u8;
                            }
                            consumes |= m.consumes;
                        } else if sac_left > 0 && sac_mana_ability(ad) {
                            sac_ab = Some(i as u8);
                        }
                    } else {
                        extra += 1;
                    }
                }
            }
            if let Some(i) = sac_ab {
                sac_left -= 1;
                // The sacrifice mode replaces the permanent's other modes in this variant (one
                // tap per permanent); it makes any color but costs a creature.
                mask = 0b11111;
                n = 1;
                ability = i;
                sac = true;
                consumes = true;
            }
            if mask == 0 {
                continue;
            }
            if c.types.contains(Types::CREATURE) && o.entered_turn >= self.s.turn.own_turn[seat.idx()] && !c.keywords.contains(Keywords::HASTE) {
                continue;
            }
            let mut pref = extra + if c.supertypes.contains(Supertypes::BASIC) { 0 } else { 1 } + if consumes { 8 } else { 0 };
            if !self.s.pay_hint.is_empty() {
                pref = if self.s.pay_hint.contains(&r) { 0 } else { 100 + pref };
            }
            out.push(SrcInfo { obj: r, colors: mask, pref, n, ability, sac });
        }
        // A sacrifice source needs its own victim: a permanent that is not itself providing mana
        // to this payment. Keep as many such sources as there are victims.
        if out.iter().any(|s| s.sac) {
            let mut free = self.sac_victims(seat, &out);
            let mut kept: SmallVec<[SrcInfo; 16]> = SmallVec::new();
            for s in out {
                if s.sac {
                    if free == 0 {
                        continue;
                    }
                    free -= 1;
                }
                kept.push(s);
            }
            return kept;
        }
        out
    }

    /// Creatures `seat` could sacrifice to a mana ability without losing a mana source of the
    /// payment `srcs` (a creature that is itself a source is not a victim).
    fn sac_victims(&mut self, seat: Seat, srcs: &[SrcInfo]) -> usize {
        self.sac_candidates(seat, srcs).len()
    }

    fn sac_candidates(&mut self, seat: Seat, srcs: &[SrcInfo]) -> Vec<ObjRef> {
        // Reads derived characteristics (types, subtypes) of permanents; make sure they are current.
        self.refresh();
        let Some(src) = srcs.iter().find(|s| s.sac) else { return Vec::new() };
        let def = self.db.def(self.s.obj(src.obj).def);
        let Some(item) = def.activated(src.ability).and_then(|ad| ad.cost.iter().find(|x| matches!(x, CostItem::Sacrifice(_)))) else { return Vec::new() };
        let f = match item {
            CostItem::Sacrifice(f) => f.clone(),
            _ => return Vec::new(),
        };
        let all: Vec<ObjRef> = self.s.battlefield.iter().copied().filter(|&r| self.s.obj(r).controller == seat).collect();
        all.into_iter()
            .filter(|&r| {
                let c = self.chars(r);
                self.filter_match(&f, r, &c, false, seat)
            })
            .filter(|&r| !srcs.iter().any(|s| s.obj == r))
            .collect()
    }

    /// The canonical victim for an automatic sacrifice payment: never a target of the spell or
    /// ability being paid for unless nothing else can be sacrificed (`avoid`), then tokens first, then the lowest
    /// mana value, then the usual option order. A player who wants a different one activates the
    /// mana ability explicitly before casting.
    fn sac_pick(&mut self, seat: Seat, srcs: &[SrcInfo], avoid: &[ObjRef]) -> Option<ObjRef> {
        let mut v = self.sac_candidates(seat, srcs);
        v.sort_by_key(|&r| {
            let o = self.s.obj(r);
            let mv = if o.kind == ObjKind::Token { 0 } else { self.db.def(o.def).cost.as_ref().map_or(0, |c| c.mana_value()) };
            (avoid.contains(&r), o.kind != ObjKind::Token, mv, self.obj_key(seat, r))
        });
        v.first().copied()
    }

    /// Plans the canonical payment for a cost, or None if it cannot be paid.
    pub fn plan_cost(&mut self, seat: Seat, cost: &ManaCost, x: u8, except: Option<ObjRef>, spell: Option<ObjRef>) -> Option<(Payment, SmallVec<[SrcInfo; 16]>)> {
        if let Some(r) = self.plan_cost_with(seat, cost, x, except, spell, 0) {
            return Some(r);
        }
        for k in 1..=self.sac_mana_count(seat) {
            if let Some(r) = self.plan_cost_with(seat, cost, x, except, spell, k) {
                return Some(r);
            }
        }
        None
    }

    fn sac_mana_count(&self, seat: Seat) -> usize {
        self.s
            .battlefield
            .iter()
            .filter(|&&r| {
                let o = self.s.obj(r);
                o.controller == seat && !o.tapped && self.db.def(o.def).abilities.iter().any(|a| matches!(a, AbilityDef::Activated(ad) if ad.is_mana && sac_mana_ability(ad)))
            })
            .count()
    }

    fn plan_cost_with(&mut self, seat: Seat, cost: &ManaCost, x: u8, except: Option<ObjRef>, spell: Option<ObjRef>, sac_count: usize) -> Option<(Payment, SmallVec<[SrcInfo; 16]>)> {
        let srcs = self.mana_sources_ex(seat, except, sac_count);
        let mut ms: SmallVec<[ManaSource; 16]> = SmallVec::new();
        for (i, s) in srcs.iter().enumerate() {
            for _ in 0..s.n {
                ms.push(ManaSource { tag: i as u32, colors: s.colors, pref: s.pref });
            }
        }
        let generic = cost.generic as u32 + (cost.x as u32) * (x as u32);
        let mut pool = self.s.players[seat.idx()].pool;
        // Restricted mana that this spell may use (creature spell of the chosen type).
        let mut usable = [0u8; 6];
        if let Some(sp) = spell {
            if !self.s.players[seat.idx()].rpool.is_empty() {
                let c = self.chars(sp);
                if c.types.contains(Types::CREATURE) {
                    for &(col, t) in &self.s.players[seat.idx()].rpool {
                        if c.subtypes.has(t) {
                            usable[col as usize] += 1;
                        }
                    }
                }
            }
        }
        for c in 0..6 {
            pool.0[c] = pool.0[c].saturating_add(usable[c]);
        }
        // A named payment source (601.2h: the player chooses what to pay with) is honored before
        // floating mana: plan from the named sources alone, and fall back to the pool only if
        // they cannot pay.
        let hinted = if self.s.pay_hint.is_empty() {
            None
        } else {
            let mut bare = pool;
            for c in 0..6 {
                bare.0[c] = usable[c];
            }
            plan_payment(cost.pips, cost.hyb, generic, &bare, &ms)
        };
        let mut pay = match hinted {
            Some(p) => p,
            None => plan_payment(cost.pips, cost.hyb, generic, &pool, &ms)?,
        };
        // Restricted mana is spent first (it is useless otherwise).
        for c in 0..6 {
            pay.restricted[c] = usable[c].min(pay.pool_used[c]);
        }
        Some((pay, srcs))
    }

    /// Executes a planned payment: spends pool mana and taps the chosen sources. A source that
    /// makes more mana than the payment used puts the rest in the pool.
    pub fn apply_payment(&mut self, seat: Seat, pay: &Payment, srcs: &[SrcInfo], avoid: &[ObjRef]) {
        for c in 0..6 {
            let r = pay.restricted[c];
            let n = pay.pool_used[c] - r;
            let pool = &mut self.s.players[seat.idx()].pool;
            pool.0[c] -= n;
            for _ in 0..r {
                // Spend a restricted mana of this color (the first that fits the spell; all usable
                // entries of a color are interchangeable for the payment).
                let rp = &mut self.s.players[seat.idx()].rpool;
                if let Some(i) = rp.iter().position(|e| e.0 as usize == c) {
                    rp.remove(i);
                }
            }
        }
        let mut i = 0;
        while i < pay.taps.len() {
            let tag = pay.taps[i].0;
            let mut used = 0u8;
            while i < pay.taps.len() && pay.taps[i].0 == tag {
                used += 1;
                i += 1;
            }
            self.tap_for_mana(seat, &srcs[tag as usize], used, srcs, avoid);
        }
    }

    /// Pays the cost of a source's mana ability when it is used by automatic payment, and does
    /// its non-mana effects. `used` of its `n` mana went to the payment; the rest floats.
    fn tap_for_mana(&mut self, seat: Seat, s: &SrcInfo, used: u8, srcs: &[SrcInfo], avoid: &[ObjRef]) {
        let r = s.obj;
        if s.ability == 255 {
            self.tap(r);
            return;
        }
        let db = self.db;
        let def = db.def(self.s.obj(r).def);
        let ad = def.activated(s.ability).expect("mana source ability");
        let mut sac = false;
        for item in &ad.cost {
            match item {
                CostItem::TapSelf => self.tap(r),
                CostItem::SacrificeSelf => sac = true,
                CostItem::Sacrifice(_) => {
                    if let Some(v) = self.sac_pick(seat, srcs, avoid) {
                        self.move_zone(v, ZoneKind::Graveyard, MoveOpts::default());
                    }
                }
                _ => {}
            }
        }
        if let Some(e) = &ad.effect {
            self.exec_mana_effect(seat, r, e, 0, false);
        }
        let left = s.n.saturating_sub(used);
        if left > 0 {
            let color = ManaColor::ALL[(s.colors.trailing_zeros() as usize).min(5)];
            self.s.players[seat.idx()].pool.add(color, left);
        }
        if sac && self.s.is_live(r) {
            self.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
        }
    }

    /// Runs the effect of a mana ability. `add` = also add the mana to the pool (explicit
    /// activation); automatic payment adds the mana itself and runs only the side effects.
    pub fn exec_mana_effect(&mut self, seat: Seat, src: ObjRef, e: &Effect, color: u8, add: bool) {
        match e {
            Effect::Seq(v) => {
                for x in v {
                    self.exec_mana_effect(seat, src, x, color, add);
                }
            }
            Effect::AddMana { color: c, n } => {
                if add {
                    self.add_mana(seat, *c, *n);
                }
            }
            Effect::AddManaAny { n: Expr::Const(n) } => {
                if add {
                    self.add_mana(seat, ManaColor::ALL[color as usize], *n as u8);
                }
            }
            Effect::AddManaRestricted => {
                if add {
                    let t = self.s.obj(src).chosen.saturating_sub(1) as u8;
                    self.s.players[seat.idx()].rpool.push((color, t));
                }
            }
            Effect::Damage { amount: Expr::Const(n), to: Rcpt::Player(PRef::You) } => {
                self.deal_damage(Some(src), Target::Player(seat), *n as u32, false);
            }
            Effect::LoseLife { who: PRef::You, n: Expr::Const(n) } => self.lose_life(seat, *n as u32),
            Effect::GainLife { who: PRef::You, n: Expr::Const(n) } => self.gain_life(seat, *n as u32),
            other => debug_assert!(false, "unsupported effect in a mana ability: {other:?}"),
        }
    }

    pub fn add_mana(&mut self, seat: Seat, color: ManaColor, n: u8) {
        self.s.players[seat.idx()].pool.add(color, n);
        self.emit(crate::event::Event::ManaAdded { player: seat, color, n });
    }

    /// The explicit mana-ability options of `seat`: (permanent, ability, color index), in
    /// canonical order, collapsed for interchangeable permanents. Ordinary sources (lands, simple
    /// mana creatures) are listed only when `explicit_mana` is on; abilities with real costs
    /// (Lion's Eye Diamond) are always explicit.
    pub fn mana_ability_options(&mut self, seat: Seat) -> Vec<(ObjRef, u8, u8, Option<ObjRef>)> {
        self.refresh();
        let explicit = self.s.cfg.explicit_mana;
        let bf: Vec<ObjRef> = self.s.battlefield.clone();
        let mut cands: Vec<(ObjRef, u8, u8, Option<ObjRef>)> = Vec::new();
        for r in bf {
            let o = self.s.obj(r);
            if o.controller != seat || o.phased {
                continue;
            }
            if self.activation_blocked(r, true) {
                continue;
            }
            let o = self.s.obj(r);
            let c = self.s.derived[r.slot as usize];
            let def = self.db.def(o.def);
            let tapped = o.tapped;
            let sick = c.types.contains(Types::CREATURE) && o.entered_turn >= self.s.turn.own_turn[seat.idx()] && !c.keywords.contains(Keywords::HASTE);
            if explicit && c.types.contains(Types::LAND) && !tapped {
                for (st, mc) in [(SUB_PLAINS, ManaColor::W), (SUB_ISLAND, ManaColor::U), (SUB_SWAMP, ManaColor::B), (SUB_MOUNTAIN, ManaColor::R), (SUB_FOREST, ManaColor::G)] {
                    if c.subtypes.has(st) {
                        cands.push((r, 255, mc.idx() as u8, None));
                    }
                }
            }
            for (i, a) in def.abilities.iter().enumerate().filter(|_| !c.lost) {
                let ad = match a {
                    AbilityDef::Activated(ad) if ad.is_mana => ad,
                    _ => continue,
                };
                let simple = simple_mana_ability(ad).is_some();
                if simple && !explicit {
                    continue;
                }
                let tap = ad.cost.iter().any(|x| matches!(x, CostItem::TapSelf));
                if tap && (tapped || sick) {
                    continue;
                }
                let life_ok = ad.cost.iter().all(|x| match x {
                    CostItem::PayLife(n) => self.s.players[seat.idx()].life >= *n as i32,
                    _ => true,
                });
                if !life_ok {
                    continue;
                }
                let (colors, _) = ad.effect.as_ref().and_then(mana_leaf).unwrap_or((0, 0));
                // A sacrifice cost needs a chosen permanent: one option per candidate.
                let mut picks: Vec<Option<ObjRef>> = vec![None];
                if let Some(item) = ad.cost.iter().find(|x| matches!(x, CostItem::Sacrifice(_))) {
                    picks = self.pick_candidates(seat, r, item, &[]).into_iter().map(Some).collect();
                }
                for col in 0..6u8 {
                    if colors & (1 << col) != 0 {
                        for &p in &picks {
                            cands.push((r, i as u8, col, p));
                        }
                    }
                }
            }
        }
        cands.sort_by_key(|&(r, i, col, p)| (self.obj_key(seat, r), i, col, p.map(|p| self.obj_key(seat, p))));
        let mut seen: Vec<(u64, u8, u8, Option<u64>)> = Vec::new();
        let mut out = Vec::new();
        for (r, i, col, p) in cands {
            let sig = self.obj_sig(r);
            let psig = p.map(|p| self.obj_sig(p));
            if seen.contains(&(sig, i, col, psig)) {
                continue;
            }
            seen.push((sig, i, col, psig));
            out.push((r, i, col, p));
        }
        out
    }

    /// Activates a mana ability (CR 605): costs, then its effect, no stack.
    pub fn run_mana_ability(&mut self, seat: Seat, src: ObjRef, ability: u8, color: u8, pick: Option<ObjRef>) {
        if ability == 255 {
            self.tap(src);
            self.add_mana(seat, ManaColor::ALL[color as usize], 1);
            return;
        }
        let db = self.db;
        let def = db.def(self.s.obj(src).def);
        let ad = def.activated(ability).expect("mana ability");
        let mut sac = false;
        for item in &ad.cost {
            match item {
                CostItem::TapSelf => self.tap(src),
                CostItem::SacrificeSelf => sac = true,
                CostItem::PayLife(n) => self.lose_life(seat, *n as u32),
                CostItem::Sacrifice(_) => {
                    if let Some(p) = pick {
                        self.move_zone(p, ZoneKind::Graveyard, MoveOpts::default());
                    }
                }
                CostItem::DiscardHand => {
                    let hand: Vec<ObjRef> = self.s.players[seat.idx()].hand.clone();
                    for h in hand {
                        self.move_zone(h, ZoneKind::Graveyard, MoveOpts::default());
                    }
                }
                _ => debug_assert!(false, "unsupported mana ability cost"),
            }
        }
        if let Some(e) = &ad.effect {
            self.exec_mana_effect(seat, src, e, color, true);
        }
        if sac && self.s.is_live(src) {
            self.move_zone(src, ZoneKind::Graveyard, MoveOpts::default());
        }
    }

    // ---- targets ---------------------------------------------------------------------------

    /// Can `ctrl`'s spell or ability target `t`? Hexproof, shroud, hexproof-from-colors effects.
    pub fn can_target(&self, ctrl: Seat, t: Target) -> bool {
        self.can_target_from(ctrl, t, None)
    }

    pub fn can_target_from(&self, ctrl: Seat, t: Target, source: Option<ObjRef>) -> bool {
        let src_colors = source.filter(|&s| self.s.is_live(s)).map(|s| self.chars(s).colors).unwrap_or(Colors::empty());
        let hexproof_from = |owner: Seat| -> bool {
            owner != ctrl
                && self.s.player_fx.iter().any(|f| f.player == owner && matches!(f.fx, PlayerFx::HexproofFrom(c) if c.intersects(src_colors)))
        };
        match t {
            Target::None => false,
            Target::Player(p) => !hexproof_from(p),
            Target::Obj(r) => {
                if !self.s.is_live(r) {
                    return false;
                }
                let o = self.s.obj(r);
                if o.zone == ZoneKind::Battlefield {
                    let k = self.chars(r).keywords;
                    if k.contains(Keywords::SHROUD) {
                        return false;
                    }
                    if k.contains(Keywords::HEXPROOF) && o.controller != ctrl {
                        return false;
                    }
                    if o.controller != ctrl && self.chars(r).hexproof_from.intersects(src_colors) {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Can the spell `r` on the stack be countered (CR 101.2)?
    pub fn can_be_countered(&self, r: ObjRef) -> bool {
        let ctrl = self.s.obj(r).controller;
        let own = self.chars(r).keywords.contains(Keywords::UNCOUNTERABLE);
        let by_mana = self.s.stack.iter().any(|e| e.obj == r && e.cast.uncounterable);
        !own && !by_mana && !self.s.player_fx.iter().any(|f| f.player == ctrl && f.fx == PlayerFx::SpellsUncounterable)
    }

    /// All legal targets for a spec, in canonical order (built from info visible to `ctrl`).
    pub fn legal_targets(&mut self, spec: &TargetSpec, ctrl: Seat, source: Option<ObjRef>) -> SmallVec<[Target; 8]> {
        self.refresh();
        let mut out: SmallVec<[Target; 8]> = SmallVec::new();
        if let Some(rel) = spec.players {
            for s in 0..2u8 {
                let seat = Seat(s);
                let ok = match rel {
                    Rel::Any => true,
                    Rel::You => seat == ctrl,
                    Rel::Opp => seat != ctrl,
                };
                if ok && self.can_target_from(ctrl, Target::Player(seat), source) {
                    out.push(Target::Player(seat));
                }
            }
        }
        if let Some(f) = &spec.objects {
            let mut cands: SmallVec<[ObjRef; 16]> = SmallVec::new();
            match f.zone {
                ZoneKind::Battlefield => cands.extend(self.s.battlefield.iter().copied()),
                ZoneKind::Stack => cands.extend(self.s.stack.iter().map(|e| e.obj)),
                ZoneKind::Graveyard => {
                    for p in 0..2 {
                        cands.extend(self.s.players[p].graveyard.iter().copied());
                    }
                }
                ZoneKind::Exile => cands.extend(self.s.exile.iter().copied()),
                _ => {}
            }
            for r in cands {
                if self.matches_filter(f, r, source, ctrl) && self.can_target_from(ctrl, Target::Obj(r), source) {
                    out.push(Target::Obj(r));
                }
            }
        }
        let me = ctrl;
        out.sort_by_key(|t| self.target_key(me, *t));
        out
    }

    /// Canonical sort key built only from information visible to `viewer`.
    pub fn target_key(&self, viewer: Seat, t: Target) -> (u8, u32, u32) {
        match t {
            Target::None => (2, 0, 0),
            Target::Player(s) => (0, if s == viewer { 0 } else { 1 }, 0),
            Target::Obj(r) => (1, self.s.obj(r).def.0 as u32, self.s.vids[r.slot as usize][viewer.idx()].0),
        }
    }

    /// Canonical option order key. Note: for permanents on the battlefield the per-seat view ids
    /// are assigned in entry order for both seats, so this order is the same whichever seat asks;
    /// `sac_pick`, `attack_candidates` and `target_key` rely on it (RFC 0004 documents it).
    pub fn obj_key(&self, viewer: Seat, r: ObjRef) -> (u32, u32) {
        if crate::canary::on(crate::canary::OPTION_ORDER_BY_CARDID) {
            return (self.s.obj(r).card.map(|c| c.0 as u32).unwrap_or(0), 0);
        }
        // Among identical cards the one the other seat already knows comes first, so which copy an
        // option stands for never depends on whether an unknown copy exists.
        let known_to_other = self.s.knowledge[r.slot as usize].known_to & (1 << viewer.other().idx()) != 0;
        (self.s.obj(r).def.0 as u32 * 2 + (!known_to_other) as u32, self.s.vids[r.slot as usize][viewer.idx()].0)
    }

    /// Hash of everything observable about an object's rules state: two objects with equal
    /// signatures are interchangeable (used to collapse equivalent options).
    pub fn obj_sig(&self, r: ObjRef) -> u64 {
        let o = self.s.obj(r);
        let c = self.chars(r);
        let mut h = Fx64::default();
        o.def.hash(&mut h);
        o.controller.hash(&mut h);
        o.tapped.hash(&mut h);
        o.damage.hash(&mut h);
        o.dt_damage.hash(&mut h);
        o.counters.hash(&mut h);
        // What it is attached to matters (it is also `keyed`, so hosts are never interchangeable):
        // two Equipment on different creatures are not the same choice (603.3b trigger order).
        o.attached_to.hash(&mut h);
        o.chosen.hash(&mut h);
        (self.s.players[o.controller.idx()].ring_bearer == Some(r)).hash(&mut h);
        c.hash(&mut h);
        self.is_summoning_sick_safe(r).hash(&mut h);
        // An object other state refers to by identity (a Skyclave Apparition's exiled card, an
        // "until leaves" link, a delayed trigger source, something attached to it, an exile-play
        // permission) is never interchangeable with another.
        if self.keyed(r) {
            r.hash(&mut h);
        }
        h.finish()
    }

    /// Does anything in the state refer to `r` by identity?
    pub(crate) fn keyed(&self, r: ObjRef) -> bool {
        self.s.links.iter().any(|l| l.src == r)
            || self.s.until_links.iter().any(|l| l.src == r || l.card == r)
            || self.s.exile_plays.iter().any(|&(x, _)| x == r)
            || self.s.delayed.iter().any(|d| d.source == r || d.cap.obj == Some(r) || d.cap.objs.contains(&r))
            || self.s.attack_watch.iter().any(|w| w.source == r)
            || self.s.battlefield.iter().any(|&b| self.s.obj(b).attached_to == Some(r))
    }

    fn is_summoning_sick_safe(&self, r: ObjRef) -> bool {
        let o = self.s.obj(r);
        o.entered_turn >= self.s.turn.own_turn[o.controller.idx()]
    }
}


/// A mana ability that automatic payment can use: it costs only a tap (and maybe a sacrifice of
/// itself) and its effect adds mana (plus fixed side effects such as Ancient Tomb's damage).
pub struct SimpleMana {
    pub colors: u8,
    pub n: u8,
    pub consumes: bool,
}

pub fn simple_mana_ability(ad: &ActivatedDef) -> Option<SimpleMana> {
    let mut consumes = false;
    for c in &ad.cost {
        match c {
            CostItem::TapSelf => {}
            CostItem::SacrificeSelf => consumes = true,
            _ => return None,
        }
    }
    if matches!(ad.effect, Some(Effect::AddManaRestricted)) {
        return None;
    }
    let (colors, n) = ad.effect.as_ref().and_then(mana_leaf)?;
    Some(SimpleMana { colors, n, consumes })
}

/// "{T}, Sacrifice a <permanent>: Add one mana of any color" (Lazotep Quarry): usable by automatic
/// payment with a canonical victim. Always also an explicit option (it is not `simple`).
pub fn sac_mana_ability(ad: &ActivatedDef) -> bool {
    let mut tap = false;
    let mut sac = false;
    for c in &ad.cost {
        match c {
            CostItem::TapSelf => tap = true,
            CostItem::Sacrifice(_) if !sac => sac = true,
            _ => return false,
        }
    }
    tap && sac && matches!(ad.effect, Some(Effect::AddManaAny { n: Expr::Const(1) }))
}

/// (color mask, amount) of the mana-producing leaf of a mana ability's effect.
pub fn mana_leaf(e: &Effect) -> Option<(u8, u8)> {
    match e {
        Effect::AddMana { color, n } => Some((1 << color.idx(), *n)),
        Effect::AddManaAny { n: Expr::Const(n) } => Some((0b11111, *n as u8)),
        Effect::AddManaRestricted => Some((0b11111, 1)),
        Effect::Seq(v) => v.iter().find_map(mana_leaf),
        _ => None,
    }
}

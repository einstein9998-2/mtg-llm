//! Total-cost computation, affordability and payment (CR 601.2b-h, 602.2). Everything the cast
//! frame needs to know about "what would this cost" lives here, so the enumerator and the frame
//! agree by construction.

use crate::card::*;
use crate::cx::Cx;
use crate::eval::Env;
use crate::event::Event;
use crate::ids::*;
use crate::mana::ManaCost;
use crate::ops::MoveOpts;
use crate::state::Captured;
use crate::types::*;
use smallvec::SmallVec;

/// Choices made so far while casting/activating.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CastCtx {
    /// 0 = normal cost, i (>= 1) = the i-th `Alt` ability of the card.
    pub way: u8,
    /// 0 = none chosen, i = `AddCost.options[i - 1]` (the first `AddCost` ability).
    pub add_choice: u8,
    /// Phyrexian symbols paid with life, per color.
    pub phy_life: [u8; 6],
    pub x: u8,
    /// Objects chosen for the pick-requiring items of the plan, in plan order.
    pub picks: SmallVec<[ObjRef; 4]>,
    /// Graveyard cards exiled with delve (each pays for one generic mana).
    pub delve: SmallVec<[ObjRef; 8]>,
    /// The delve choice has been made (until then, payability assumes the best delve).
    pub delve_done: bool,
    /// Times the replicate cost is paid.
    pub replicate: u8,
    /// Modes chosen beyond the first (escalate payments).
    pub escalate: u8,
    /// The spell is cast from a zone other than the hand.
    pub not_hand: bool,
}

/// The fully resolved cost of a cast or activation.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Plan {
    pub mana: ManaCost,
    /// Phyrexian symbols of the plan per color, before they were resolved into mana or life.
    pub phy_total: [u8; 6],
    pub life: u32,
    pub energy: u32,
    pub tap_self: bool,
    /// A loyalty cost (see `CostItem::Loyalty`).
    pub loyalty: Option<i8>,
    pub sac_self: bool,
    pub discard_self: bool,
    /// The plan is for casting a spell (restricted mana may apply).
    pub is_spell: bool,
    /// Escape-style collective exile: other graveyard cards with at least this many card types.
    pub collective: Option<u8>,
    /// Items that need a chosen object, in order.
    pub picks: SmallVec<[CostItem; 3]>,
}

/// The `(source, LOYALTY_USED)` entry of `State::used` records that the permanent's loyalty
/// ability for the turn was activated.
pub const LOYALTY_USED: u8 = 255;

/// Index of the i-th (1-based) `Alt` ability of a def.
pub fn alt_of(def: &CardDef, way: u8) -> Option<&AltCost> {
    if way == 0 || way >= crate::restrict::VIRTUAL_WAY_BASE {
        return None;
    }
    def.abilities.iter().filter_map(|a| if let AbilityDef::Alt(x) = a { Some(x) } else { None }).nth(way as usize - 1)
}

pub fn alt_count(def: &CardDef) -> u8 {
    def.abilities.iter().filter(|a| matches!(a, AbilityDef::Alt(_))).count() as u8
}

pub fn has_delve(def: &CardDef) -> bool {
    def.abilities.iter().any(|a| matches!(a, AbilityDef::Delve))
}

pub fn replicate_of(def: &CardDef) -> Option<ManaCost> {
    def.abilities.iter().find_map(|a| if let AbilityDef::Replicate(c) = a { Some(*c) } else { None })
}

pub fn escalate_of(def: &CardDef) -> Option<&Vec<CostItem>> {
    def.abilities.iter().find_map(|a| if let AbilityDef::Escalate(x) = a { Some(x) } else { None })
}

pub fn add_cost_of(def: &CardDef) -> Option<&AddCostDef> {
    def.abilities.iter().find_map(|a| if let AbilityDef::AddCost(x) = a { Some(x) } else { None })
}

fn add_items(plan: &mut Plan, items: &[CostItem]) {
    for it in items {
        match it {
            CostItem::Mana(m) => {
                plan.mana.generic = plan.mana.generic.saturating_add(m.generic);
                for i in 0..6 {
                    plan.mana.pips[i] += m.pips[i];
                    plan.mana.phy[i] += m.phy[i];
                }
                for i in 0..10 {
                    plan.mana.hyb[i] += m.hyb[i];
                }
                plan.mana.x += m.x;
            }
            CostItem::PayLife(n) => plan.life += *n as u32,
            CostItem::PayEnergy(n) => plan.energy += *n as u32,
            CostItem::TapSelf => plan.tap_self = true,
            CostItem::Loyalty(n) => plan.loyalty = Some(*n),
            CostItem::SacrificeSelf => plan.sac_self = true,
            CostItem::DiscardSelf => plan.discard_self = true,
            CostItem::ExileTypes(n) => plan.collective = Some(*n),
            other => plan.picks.push(other.clone()),
        }
    }
}

impl<'a> Cx<'a> {
    /// The plan for casting the spell `card` (in whatever zone it is) with the given choices.
    pub fn spell_plan(&mut self, seat: Seat, card: ObjRef, ctx: &CastCtx) -> Plan {
        let def = self.db.def(self.s.obj(card).def);
        let mut plan = Plan { is_spell: true, ..Plan::default() };
        match alt_of(def, ctx.way) {
            // A cast permission (Aluren): no cost of its own, taxes still apply.
            None if ctx.way == crate::restrict::ENERGY_WAY => {
                add_items(&mut plan, &[CostItem::PayEnergy(def.mana_value() as u8)]);
            }
            None if ctx.way == crate::restrict::GY_WAY || ctx.way == crate::restrict::ADVENTURE_WAY => {
                if let Some(c) = def.cost {
                    add_items(&mut plan, &[CostItem::Mana(c)]);
                }
            }
            None if ctx.way >= crate::restrict::VIRTUAL_WAY_BASE => {}
            None => {
                if let Some(c) = def.cost {
                    add_items(&mut plan, &[CostItem::Mana(c)]);
                }
            }
            Some(a) => add_items(&mut plan, &a.cost),
        }
        if ctx.replicate > 0 {
            if let Some(rc) = replicate_of(def) {
                for _ in 0..ctx.replicate {
                    add_items(&mut plan, &[CostItem::Mana(rc)]);
                }
            }
        }
        if ctx.escalate > 0 {
            if let Some(items) = escalate_of(def) {
                for _ in 0..ctx.escalate {
                    add_items(&mut plan, items);
                }
            }
        }
        if ctx.add_choice > 0 {
            if let Some(ac) = add_cost_of(def) {
                if let Some(opt) = ac.options.get(ctx.add_choice as usize - 1) {
                    add_items(&mut plan, opt);
                }
            }
        }
        self.finish_plan(seat, Some(card), ctx, &mut plan);
        plan
    }

    /// The plan for activating ability `idx` of `src`.
    pub fn ability_plan(&mut self, seat: Seat, src: ObjRef, idx: u8, ctx: &CastCtx) -> Plan {
        let def = self.db.def(self.s.obj(src).def);
        let mut plan = Plan::default();
        if let Some(ad) = def.activated(idx) {
            add_items(&mut plan, &ad.cost);
            if let Some(f) = &ad.cost_reduce {
                let n = self.count_matching(seat, f);
                plan.mana.generic = plan.mana.generic.saturating_sub(n.min(255) as u8);
            }
        }
        self.finish_plan(seat, None, ctx, &mut plan);
        plan
    }

    fn finish_plan(&mut self, seat: Seat, spell: Option<ObjRef>, ctx: &CastCtx, plan: &mut Plan) {
        // Phyrexian symbols: paid with the color or 2 life, as chosen.
        for c in 0..6 {
            let n = plan.mana.phy[c];
            plan.phy_total[c] = n;
            if n > 0 {
                let life = ctx.phy_life[c].min(n);
                plan.mana.pips[c] += n - life;
                plan.life += 2 * life as u32;
                plan.mana.phy[c] = 0;
            }
        }
        // Cost increases and reductions from static abilities (generic only).
        if let Some(card) = spell {
            let delta = self.cost_mod_total(seat, card, ctx.not_hand);
            if delta >= 0 {
                plan.mana.generic = plan.mana.generic.saturating_add(delta as u8);
            } else {
                plan.mana.generic = plan.mana.generic.saturating_sub((-delta) as u8);
            }
            // Delve pays for generic mana of the total cost (702.66a).
            if has_delve(self.db.def(self.s.obj(card).def)) {
                let n = ctx.delve.len().min(plan.mana.generic as usize) as u8;
                plan.mana.generic -= n;
            }
        }
    }

    /// Battlefield objects matching `f` from `seat`'s point of view.
    pub fn count_matching(&mut self, seat: Seat, f: &ObjFilter) -> usize {
        self.refresh();
        let bf: SmallVec<[ObjRef; 24]> = self.s.battlefield.iter().copied().collect();
        bf.into_iter()
            .filter(|&r| {
                let c = self.chars(r);
                self.filter_match(f, r, &c, false, seat)
            })
            .count()
    }

    /// Net generic-mana change from `CostMod` abilities of permanents on the battlefield.
    pub fn cost_mod_total(&mut self, caster: Seat, spell: ObjRef, not_hand: bool) -> i32 {
        self.refresh();
        let mut total = 0i32;
        let bf: SmallVec<[ObjRef; 24]> = self.s.battlefield.iter().copied().collect();
        for src in bf {
            let o = self.s.obj(src);
            if o.phased {
                continue;
            }
            let (ctrl, def_id) = (o.controller, o.def);
            let def = self.db.def(def_id);
            for a in &def.abilities {
                if let AbilityDef::CostMod(m) = a {
                    let who_ok = match m.who {
                        Rel::Any => true,
                        Rel::You => caster == ctrl,
                        Rel::Opp => caster != ctrl,
                    };
                    if !who_ok {
                        continue;
                    }
                    if m.not_casters_turn && self.s.turn.active == caster {
                        continue;
                    }
                    if m.not_from_hand && !not_hand {
                        continue;
                    }
                    if m.applies.chosen_name {
                        let chosen = self.s.obj(src).chosen;
                        if chosen == 0 || self.s.obj(spell).def.0 + 1 != chosen {
                            continue;
                        }
                    }
                    // The spell is matched on characteristics only (it is not on the battlefield).
                    let c = self.chars(spell);
                    let mut f = m.applies.clone();
                    f.controller = Rel::Any;
                    f.owner = Rel::Any;
                    if !self.filter_match(&f, spell, &c, false, ctrl) {
                        continue;
                    }
                    if let Some(cond) = &m.cond {
                        let cap = Captured::default();
                        let env = Env { controller: ctrl, source: Some(src), source_lki: None, targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
                        if !self.eval_cond(&env, cond) {
                            continue;
                        }
                    }
                    total += m.generic as i32;
                }
            }
        }
        total
    }

    /// Candidates for one pick-requiring cost item, canonically ordered and collapsed so that
    /// interchangeable objects appear once.
    pub fn pick_candidates(&mut self, seat: Seat, source: ObjRef, item: &CostItem, taken: &[ObjRef]) -> Vec<ObjRef> {
        self.refresh();
        let (list, f): (Vec<ObjRef>, &ObjFilter) = match item {
            CostItem::Sacrifice(f) | CostItem::ReturnToHand(f) => (self.s.battlefield.iter().copied().filter(|&r| self.s.obj(r).controller == seat).collect(), f),
            CostItem::Discard(f) | CostItem::ExileFromHand(f) => (self.s.players[seat.idx()].hand.clone(), f),
            CostItem::ExileFromGraveyard(f) => (self.s.players[seat.idx()].graveyard.clone(), f),
            _ => return Vec::new(),
        };
        let mut v: Vec<ObjRef> = list
            .into_iter()
            .filter(|&r| (r != source || matches!(item, CostItem::Sacrifice(_) | CostItem::ReturnToHand(_))) && !taken.contains(&r))
            .filter(|&r| {
                let c = self.chars(r);
                self.filter_match(f, r, &c, false, seat)
            })
            .collect();
        v.sort_by_key(|&r| self.obj_key(seat, r));
        // Collapse interchangeable candidates (identical cards in hand/graveyard, identical
        // permanents on the battlefield).
        let mut out: Vec<ObjRef> = Vec::new();
        let mut seen: SmallVec<[u64; 8]> = SmallVec::new();
        for r in v {
            let sig = self.obj_sig(r);
            if seen.contains(&sig) {
                continue;
            }
            seen.push(sig);
            out.push(r);
        }
        out
    }

    /// Can `seat` pay `plan` right now? `source` is the spell being cast or the permanent whose
    /// ability is activated; `tap_exempt` is the permanent being tapped as a cost.
    pub fn can_afford(&mut self, seat: Seat, source: ObjRef, plan: &Plan, x: u8) -> bool {
        if plan.life > 0 && self.s.players[seat.idx()].life < plan.life as i32 {
            return false;
        }
        if (self.s.players[seat.idx()].energy as u32) < plan.energy {
            return false;
        }
        if let Some(n) = plan.loyalty {
            if n < 0 && self.s.is_live(source) && (self.s.obj(source).counter(CounterKind::Loyalty) as i32) < -(n as i32) {
                return false;
            }
        }
        let mut taken: SmallVec<[ObjRef; 3]> = SmallVec::new();
        for item in plan.picks.iter() {
            let cands = self.pick_candidates(seat, source, item, &taken);
            match cands.first() {
                None => return false,
                Some(&c) => taken.push(c),
            }
        }
        if let Some(n) = plan.collective {
            let gy: Vec<ObjRef> = self.s.players[seat.idx()].graveyard.iter().copied().filter(|&r| r != source).collect();
            if (self.types_of_cards(&gy).bits().count_ones() as u8) < n {
                return false;
            }
        }
        if plan.sac_self && !self.s.is_live(source) {
            return false;
        }
        if plan.discard_self && !(self.s.is_live(source) && self.s.obj(source).zone == ZoneKind::Hand) {
            return false;
        }
        if !plan.mana.is_zero() {
            let except = if plan.tap_self { Some(source) } else { None };
            return self.plan_cost(seat, &plan.mana, x, except, plan.is_spell.then_some(source)).is_some();
        }
        true
    }

    /// Pays `plan` in full: mana (auto payment), life, energy, then the chosen objects, then the
    /// self tap/sacrifice. Returns the mana spent (total, color mask).
    pub fn pay_plan(&mut self, seat: Seat, source: ObjRef, plan: &Plan, ctx: &CastCtx) -> (u8, u8, bool) {
        let mut spent = 0u8;
        let mut colors = 0u8;
        let mut restricted = false;
        if !plan.mana.is_zero() {
            let except = if plan.tap_self { Some(source) } else { None };
            let (pay, srcs) = self.plan_cost(seat, &plan.mana, ctx.x, except, plan.is_spell.then_some(source)).expect("pre-validated payment must succeed (engine bug)");
            for c in 0..6 {
                if pay.pool_used[c] > 0 {
                    spent += pay.pool_used[c];
                    if c < 5 {
                        colors |= 1 << c;
                    }
                }
            }
            for &(_, color) in pay.taps.iter() {
                spent += 1;
                if (color as usize) < 5 {
                    colors |= 1 << (color as usize);
                }
            }
            restricted = pay.restricted.iter().any(|&n| n > 0);
            // The spell or ability being paid for is on top of the stack with its targets chosen;
            // an automatic sacrifice must not take one of them.
            let avoid: SmallVec<[ObjRef; 4]> = self.s.stack.last().map(|e| e.targets.iter().filter_map(|t| if let crate::decision::Target::Obj(r) = t { Some(*r) } else { None }).collect()).unwrap_or_default();
            self.apply_payment(seat, &pay, &srcs, &avoid);
        }
        if plan.life > 0 {
            self.lose_life(seat, plan.life);
        }
        if plan.energy > 0 {
            self.s.players[seat.idx()].energy -= plan.energy;
        }
        if let Some(n) = plan.loyalty {
            // 606.3: one loyalty ability per permanent each turn (even if it is countered).
            self.s.used.push((source, LOYALTY_USED));
            self.add_counters(source, CounterKind::Loyalty, n as i32);
        }
        if plan.tap_self && self.s.is_live(source) {
            self.tap(source);
        }
        for &d in ctx.delve.iter() {
            if self.s.is_live(d) {
                self.move_zone(d, ZoneKind::Exile, MoveOpts::default());
            }
        }
        for (item, &pick) in plan.picks.iter().zip(ctx.picks.iter()) {
            if !self.s.is_live(pick) {
                continue;
            }
            match item {
                CostItem::Sacrifice(_) | CostItem::Discard(_) => {
                    self.move_zone(pick, ZoneKind::Graveyard, MoveOpts::default());
                }
                CostItem::ExileFromHand(_) | CostItem::ExileFromGraveyard(_) => {
                    self.move_zone(pick, ZoneKind::Exile, MoveOpts::default());
                }
                CostItem::ReturnToHand(_) => {
                    self.move_zone(pick, ZoneKind::Hand, MoveOpts::default());
                }
                _ => {}
            }
        }
        if plan.sac_self && self.s.is_live(source) {
            self.move_zone(source, ZoneKind::Graveyard, MoveOpts::default());
        }
        if plan.discard_self && self.s.is_live(source) {
            self.move_zone(source, ZoneKind::Graveyard, MoveOpts::default());
        }
        let _ = Event::GameEnded;
        (spent, colors, restricted)
    }
}

impl<'a> Cx<'a> {
    /// Can the plan implied by `ctx` be paid, assuming every Phyrexian symbol not yet decided is
    /// paid in the way that is easiest to afford (life, as far as life allows)?
    pub fn affordable_with_best_phy(&mut self, seat: Seat, source: ObjRef, ability: Option<u8>, ctx: &CastCtx, decided: [u8; 6]) -> bool {
        let mut c = ctx.clone();
        if ability.is_none() && !c.delve_done && has_delve(self.db.def(self.s.obj(source).def)) {
            // Assume the most generous delve until the player has chosen.
            c.delve.clear();
            let g = self.spell_plan(seat, source, &c).mana.generic as usize;
            let gy: SmallVec<[ObjRef; 8]> = self.s.players[seat.idx()].graveyard.iter().copied().filter(|&r| r != source).take(g).collect();
            c.delve = gy;
        }
        let plan0 = match ability {
            None => self.spell_plan(seat, source, &c),
            Some(i) => self.ability_plan(seat, source, i, &c),
        };
        // Symbols that can still be paid with life.
        let mut budget = (self.s.players[seat.idx()].life - plan0.life as i32).max(0) / 2;
        for col in 0..6 {
            let undecided = plan0.phy_total[col].saturating_sub(decided[col]);
            let take = (undecided as i32).min(budget).max(0) as u8;
            c.phy_life[col] += take;
            budget -= take as i32;
        }
        let plan = match ability {
            None => self.spell_plan(seat, source, &c),
            Some(i) => self.ability_plan(seat, source, i, &c),
        };
        self.can_afford(seat, source, &plan, c.x)
    }
}

/// Short human/LLM-readable description of cost items (labels only; never parsed).
pub fn describe_cost(items: &[CostItem]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for it in items {
        parts.push(match it {
            CostItem::TapSelf => "tap".into(),
            CostItem::Mana(m) => m.to_string(),
            CostItem::PayLife(n) => format!("pay {n} life"),
            CostItem::SacrificeSelf => "sacrifice it".into(),
            CostItem::Sacrifice(_) => "sacrifice a permanent".into(),
            CostItem::Discard(_) => "discard a card".into(),
            CostItem::ExileFromHand(_) => "exile a card from your hand".into(),
            CostItem::ExileFromGraveyard(_) => "exile a card from your graveyard".into(),
            CostItem::ReturnToHand(_) => "return a permanent to hand".into(),
            CostItem::PayEnergy(n) => format!("pay {n} energy"),
            CostItem::ExileTypes(n) => format!("exile cards with {n} card types from your graveyard"),
            CostItem::DiscardHand => "discard your hand".into(),
            CostItem::DiscardSelf => "discard this card".into(),
            CostItem::Loyalty(n) => format!("loyalty {n:+}"),
        });
    }
    parts.join(", ")
}

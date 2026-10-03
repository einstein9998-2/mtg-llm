//! Casting spells and activating abilities (CR 601.2, 602.2). Legality is guaranteed by
//! pre-validation in the enumerator and in each stage (an option is offered only if a complete
//! legal cast remains possible), so there is no rollback (doc 01 sections 6.3, 6.5).
//!
//! Stages follow the CR order: move to stack, modes, X, targets, additional-cost choice,
//! Phyrexian choices, object picks for costs, payment.

use crate::card::*;
use crate::cost::*;
use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::event::Event;
use crate::frame::*;
use crate::ids::*;
use crate::ops::MoveOpts;
use crate::resolve::{entry_specs, specs_raw_x};
use crate::state::*;
use crate::types::*;

fn entry_mut<'s>(cx: &'s mut Cx, obj: ObjRef) -> &'s mut StackEntry {
    cx.s.stack.iter_mut().rev().find(|e| e.obj == obj).expect("cast frame must have a stack entry")
}

/// The object whose abilities/characteristics define this cast (the spell, or the source).
fn source_of(f: &CastFrame) -> ObjRef {
    match f.ability {
        None => f.obj,
        Some((src, _)) => src,
    }
}

fn plan_of(cx: &mut Cx, f: &CastFrame) -> Plan {
    match f.ability {
        None => cx.spell_plan(f.controller, f.obj, &f.ctx),
        Some((src, idx)) => cx.ability_plan(f.controller, src, idx, &f.ctx),
    }
}

pub fn run(cx: &mut Cx, f: &mut CastFrame) -> Next {
    let seat = f.controller;
    match f.stage {
        CastStage::Start => {
            match f.ability {
                None => {
                    // 601.2a: the card moves to the stack and becomes a new object.
                    let nr = cx.move_zone(f.obj, ZoneKind::Stack, MoveOpts { face: f.ctx.way == crate::restrict::ADVENTURE_WAY, ..Default::default() }).expect("spell moves to stack");
                    f.obj = nr;
                    let def = cx.s.obj(nr).def;
                    cx.s.stack.push(StackEntry {
                        obj: nr,
                        controller: seat,
                        kind: StackKind::Spell,
                        targets: Targets::new(),
                        x: 0,
                        def,
                        modes: 0,
                        cap: Captured::default(),
                        cast: CastInfo { from: f.from, way: f.ctx.way, mana_spent: 0, colors_spent: 0, delve_is: 0, replicate: 0, uncounterable: false },
                    });
                    f.stage = CastStage::Modes;
                }
                Some((src, idx)) => {
                    let def = cx.s.obj(src).def;
                    let nr = cx.s.alloc_slot(ObjKind::Ability, def, seat);
                    {
                        let o = cx.s.obj_mut_raw(nr);
                        o.zone = ZoneKind::Stack;
                        o.controller = seat;
                    }
                    let mut vid = [ViewId::NONE; 2];
                    for v in 0..2 {
                        vid[v] = ViewId(cx.s.next_vid[v]);
                        cx.s.next_vid[v] += 1;
                    }
                    cx.s.vids[nr.slot as usize] = vid;
                    f.obj = nr;
                    cx.s.stack.push(StackEntry {
                        obj: nr,
                        controller: seat,
                        kind: StackKind::Ability { source: src, ability: idx },
                        targets: Targets::new(),
                        x: 0,
                        def,
                        modes: 1,
                        cap: Captured::default(),
                        cast: CastInfo::default(),
                    });
                    f.stage = CastStage::AddCost;
                }
            }
            Next::Stay
        }
        CastStage::Modes => {
            let db = cx.db;
            let def = db.def(cx.s.obj(f.obj).def);
            let (min, max, n) = match def.spell_def() {
                Some((_, sd)) => (sd.choose.0, sd.choose.1, sd.modes.len() as u8),
                None => (1, 1, 1),
            };
            if n <= 1 {
                f.modes = 1;
                entry_mut(cx, f.obj).modes = 1;
                f.stage = CastStage::AddCost;
                return Next::Stay;
            }
            if f.mode_count >= max {
                f.stage = CastStage::AddCost;
                return Next::Stay;
            }
            let sd = def.spell_def().unwrap().1;
            let mut opts: Vec<Opt> = Vec::new();
            for i in f.mode_next..n {
                let ok = sd.modes[i as usize].targets.iter().all(|spec| spec.optional || spec.x_count || cx.legal_targets(spec, seat, Some(f.obj)).into_iter().any(|t| t != Target::Obj(f.obj)));
                // An additional mode must be affordable with its escalate payment.
                let ok = ok && (f.mode_count == 0 || escalate_of(def).is_none() || {
                    let mut c = f.ctx.clone();
                    c.escalate = f.mode_count;
                    cx.affordable_with_best_phy(seat, f.obj, None, &c, [0; 6])
                });
                if ok {
                    opts.push(Opt::Mode(i));
                }
            }
            if f.mode_count >= min && f.mode_count > 0 {
                opts.push(Opt::Done);
            }
            if opts.is_empty() {
                // Pre-validation guarantees this cannot happen.
                debug_assert!(false, "no legal mode for {}", def.name);
                f.stage = CastStage::AddCost;
                return Next::Stay;
            }
            ask(cx, seat, DecisionKind::ChooseMode { spell: f.obj, chosen: f.mode_count }, opts);
            Next::Await
        }
        CastStage::Replicate => {
            let rc = if f.ability.is_none() { replicate_of(cx.db.def(cx.s.obj(f.obj).def)) } else { None };
            if rc.is_none() {
                f.stage = CastStage::X;
                return Next::Stay;
            }
            let mut opts = vec![Opt::Number(0)];
            for n in 1..=8u8 {
                let mut c = f.ctx.clone();
                c.replicate = n;
                if cx.affordable_with_best_phy(seat, f.obj, None, &c, [0; 6]) {
                    opts.push(Opt::Number(n as u32));
                } else {
                    break;
                }
            }
            if opts.len() == 1 {
                f.stage = CastStage::X;
                return Next::Stay;
            }
            ask(cx, seat, DecisionKind::ChooseReplicate, opts);
            Next::Await
        }
        CastStage::X => {
            let plan = plan_of(cx, f);
            if plan.mana.x == 0 {
                f.stage = CastStage::Targets { slot: 0 };
                return Next::Stay;
            }
            let src = source_of(f);
            let mut opts = Vec::new();
            for x in 0..=20u8 {
                let mut c = f.ctx.clone();
                c.x = x;
                let ability = f.ability.map(|a| a.1);
                if f.ability.is_none() && cx.cast_restricted_x(seat, f.obj, Some(x)) {
                    break;
                }
                if cx.affordable_with_best_phy(seat, src, ability, &c, [0; 6]) {
                    // A target whose mana value must equal X has to exist.
                    let entry = cx.s.stack.iter().rev().find(|e| e.obj == f.obj).unwrap().clone();
                    let specs = entry_specs(cx.db, &entry);
                    let mut has_target = true;
                    for spec in specs.iter().filter(|s| !s.optional && s.objects.as_ref().map(|o| o.cmc_x).unwrap_or(false)) {
                        cx.s.cur_x = Some(x);
                        has_target &= !cx.legal_targets(spec, seat, Some(src)).is_empty();
                        cx.s.cur_x = None;
                    }
                    // "X target ...": enough distinct targets must exist.
                    for spec in specs_raw_x(cx.db, &entry) {
                        let n = cx.legal_targets(&spec, seat, Some(src)).into_iter().filter(|t| *t != Target::Obj(f.obj)).count();
                        has_target &= n >= x as usize;
                    }
                    if has_target {
                        opts.push(Opt::Number(x as u32));
                    }
                } else {
                    break;
                }
            }
            if opts.is_empty() {
                opts.push(Opt::Number(0));
            }
            ask(cx, seat, DecisionKind::ChooseX, opts);
            Next::Await
        }
        CastStage::Targets { slot } => {
            let e = cx.s.stack.iter().rev().find(|e| e.obj == f.obj).unwrap().clone();
            let specs = entry_specs(cx.db, &e);
            let src = source_of(f);
            match specs.get(slot as usize) {
                None => {
                    f.stage = CastStage::Delve;
                    Next::Stay
                }
                Some(spec) => {
                    cx.s.cur_x = Some(f.ctx.x);
                    let cands = cx.legal_targets(spec, seat, Some(src));
                    cx.s.cur_x = None;
                    // Exclude the spell itself from stack targets (it is already on the stack).
                    let chosen: Vec<Target> = e.targets.iter().copied().collect();
                    let mut opts: Vec<Opt> = cands.into_iter().filter(|t| *t != Target::Obj(f.obj) && !(spec.distinct && chosen.contains(t))).map(Opt::Target).collect();
                    if spec.optional {
                        opts.push(Opt::Done);
                    }
                    ask(cx, seat, DecisionKind::ChooseTarget { slot }, opts);
                    Next::Await
                }
            }
        }
        CastStage::AddCost => {
            let def = cx.db.def(cx.s.obj(f.obj).def);
            let ac = match add_cost_of(def).filter(|_| f.ability.is_none()) {
                Some(a) => a,
                None => {
                    f.stage = CastStage::Replicate;
                    return Next::Stay;
                }
            };
            let mut opts: Vec<Opt> = Vec::new();
            for i in 0..ac.options.len() {
                let mut c = f.ctx.clone();
                c.add_choice = i as u8 + 1;
                if cx.affordable_with_best_phy(seat, f.obj, None, &c, [0; 6]) {
                    opts.push(Opt::Choice(i as u8));
                }
            }
            if ac.optional {
                opts.push(Opt::No);
            }
            if opts.len() == 1 && !ac.optional {
                if let Opt::Choice(i) = opts[0] {
                    f.ctx.add_choice = i + 1;
                }
                f.stage = CastStage::Replicate;
                return Next::Stay;
            }
            if opts.is_empty() {
                debug_assert!(false, "no affordable additional cost");
                f.stage = CastStage::Replicate;
                return Next::Stay;
            }
            ask(cx, seat, DecisionKind::ChooseAddCost { spell: f.obj }, opts);
            Next::Await
        }
        CastStage::Delve => {
            let delves = f.ability.is_none() && has_delve(cx.db.def(cx.s.obj(f.obj).def));
            let plan = plan_of(cx, f);
            let gy_left = cx.s.players[seat.idx()].graveyard.iter().filter(|r| !f.ctx.delve.contains(r)).count();
            if let Some(n) = plan.collective {
                // Escape: the cards exiled so far are `ctx.delve`; the selection may end once
                // they have enough card types among them.
                let cands = cx.pick_candidates(seat, f.obj, &CostItem::ExileFromGraveyard(ObjFilter::new(ZoneKind::Graveyard)), &f.ctx.delve);
                let mut opts: Vec<Opt> = cands.into_iter().map(Opt::Card).collect();
                if cx.types_of_cards(&f.ctx.delve).bits().count_ones() as u8 >= n {
                    opts.push(Opt::Done);
                }
                ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::Delve, remaining: 1 }, opts);
                return Next::Await;
            }
            if !delves || plan.mana.generic == 0 || gy_left == 0 {
                f.ctx.delve_done = true;
                f.stage = CastStage::Phyrexian { color: 0, left: 0 };
                return Next::Stay;
            }
            let cands = cx.pick_candidates(seat, f.obj, &CostItem::ExileFromGraveyard(ObjFilter::new(ZoneKind::Graveyard)), &f.ctx.delve);
            let mut opts: Vec<Opt> = cands.into_iter().map(Opt::Card).collect();
            let mut c = f.ctx.clone();
            c.delve_done = true;
            if cx.affordable_with_best_phy(seat, f.obj, None, &c, [0; 6]) {
                opts.push(Opt::Done);
            }
            ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::Delve, remaining: 1 }, opts);
            Next::Await
        }
        CastStage::Phyrexian { color, left } => {
            let plan = plan_of(cx, f);
            let mut color = color;
            let mut left = left;
            // Find the next color with undecided symbols.
            while color < 6 && left == 0 {
                left = plan.phy_total[color as usize];
                if left == 0 {
                    color += 1;
                } else {
                    // `left` symbols of this color are undecided; remember decisions so far in ctx.
                    f.ctx.phy_life[color as usize] = 0;
                }
            }
            if color >= 6 {
                f.stage = CastStage::Picks { i: 0 };
                return Next::Stay;
            }
            f.stage = CastStage::Phyrexian { color, left };
            let src = source_of(f);
            let ability = f.ability.map(|a| a.1);
            // Decided so far: all symbols of earlier colors, and (total - left) of this color.
            let mut decided = [0u8; 6];
            for c in 0..color as usize {
                decided[c] = plan.phy_total[c];
            }
            decided[color as usize] = plan.phy_total[color as usize] - left;
            let mut opts = Vec::new();
            // Pay with life.
            let mut c_life = f.ctx.clone();
            c_life.phy_life[color as usize] += 1;
            let mut d1 = decided;
            d1[color as usize] += 1;
            if cx.affordable_with_best_phy(seat, src, ability, &c_life, d1) {
                opts.push(Opt::Yes);
            }
            // Pay with mana.
            if cx.affordable_with_best_phy(seat, src, ability, &f.ctx, d1) {
                opts.push(Opt::No);
            }
            if opts.is_empty() {
                debug_assert!(false, "unaffordable Phyrexian symbol");
                opts.push(Opt::No);
            }
            ask(cx, seat, DecisionKind::PayPhyrexian { color }, opts);
            Next::Await
        }
        CastStage::Picks { i } => {
            let plan = plan_of(cx, f);
            match plan.picks.get(i as usize) {
                None => {
                    f.stage = CastStage::Pay;
                    Next::Stay
                }
                Some(item) => {
                    let src = source_of(f);
                    let cands = cx.pick_candidates(seat, src, item, &f.ctx.picks);
                    let opts: Vec<Opt> = cands.into_iter().map(Opt::Card).collect();
                    ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::PayCost, remaining: 1 }, opts);
                    Next::Await
                }
            }
        }
        CastStage::Pay => {
            pay_and_finish(cx, f);
            Next::Done
        }
    }
}

pub fn feed(cx: &mut Cx, f: &mut CastFrame, p: &Pending, opt: Opt) -> Next {
    match (p.kind, opt) {
        (DecisionKind::ChooseMode { .. }, Opt::Mode(i)) => {
            f.modes |= 1 << i;
            f.mode_count += 1;
            f.ctx.escalate = f.mode_count.saturating_sub(1);
            f.mode_next = i + 1;
            entry_mut(cx, f.obj).modes = f.modes;
            Next::Stay
        }
        (DecisionKind::ChooseMode { .. }, Opt::Done) => {
            f.mode_count = u8::MAX; // stop choosing
            Next::Stay
        }
        (DecisionKind::ChooseReplicate, Opt::Number(n)) => {
            f.ctx.replicate = n as u8;
            f.stage = CastStage::X;
            Next::Stay
        }
        (DecisionKind::ChooseX, Opt::Number(n)) => {
            f.ctx.x = n as u8;
            entry_mut(cx, f.obj).x = n as u8;
            f.stage = CastStage::Targets { slot: 0 };
            Next::Stay
        }
        (DecisionKind::ChooseTarget { slot }, Opt::Target(t)) => {
            entry_mut(cx, f.obj).targets.push(t);
            f.stage = CastStage::Targets { slot: slot + 1 };
            Next::Stay
        }
        (DecisionKind::ChooseTarget { slot }, Opt::Done) => {
            entry_mut(cx, f.obj).targets.push(Target::None);
            f.stage = CastStage::Targets { slot: slot + 1 };
            Next::Stay
        }
        (DecisionKind::ChooseAddCost { .. }, Opt::Choice(i)) => {
            f.ctx.add_choice = i + 1;
            f.stage = CastStage::Replicate;
            Next::Stay
        }
        (DecisionKind::ChooseAddCost { .. }, Opt::No) => {
            f.ctx.add_choice = 0;
            f.stage = CastStage::Replicate;
            Next::Stay
        }
        (DecisionKind::PayPhyrexian { color }, o @ (Opt::Yes | Opt::No)) => {
            if o == Opt::Yes {
                f.ctx.phy_life[color as usize] += 1;
            }
            if let CastStage::Phyrexian { color: c, left } = f.stage {
                f.stage = CastStage::Phyrexian { color: c, left: left - 1 };
                if left - 1 == 0 {
                    f.stage = CastStage::Phyrexian { color: c + 1, left: 0 };
                }
            }
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::Delve, .. }, Opt::Card(r)) => {
            f.ctx.delve.push(r);
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::Delve, .. }, Opt::Done) => {
            f.ctx.delve_done = true;
            f.stage = CastStage::Phyrexian { color: 0, left: 0 };
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::PayCost, .. }, Opt::Card(r)) => {
            f.ctx.picks.push(r);
            if let CastStage::Picks { i } = f.stage {
                f.stage = CastStage::Picks { i: i + 1 };
            }
            Next::Stay
        }
        _ => unreachable!("unexpected decision in cast frame: {:?}", p.kind),
    }
}

fn pay_and_finish(cx: &mut Cx, f: &mut CastFrame) {
    let seat = f.controller;
    let plan = plan_of(cx, f);
    let src = source_of(f);
    let ctx = f.ctx.clone();
    let delve_is = ctx.delve.iter().filter(|&&r| cx.s.is_live(r) && cx.db.def(cx.s.obj(r).def).types.intersects(Types::SPELL_KIND)).count() as u8;
    if f.ability.is_some() {
        // Ninjutsu: remember what the returned attacker was attacking before it leaves combat.
        for (item, &pick) in plan.picks.iter().zip(ctx.picks.iter()) {
            if let CostItem::ReturnToHand(flt) = item {
                if flt.unblocked == Some(true) {
                    let tgt = cx.s.combat.as_ref().and_then(|cb| cb.attackers.iter().find(|a| a.obj == pick).map(|a| a.target));
                    let e = entry_mut(cx, f.obj);
                    match tgt {
                        Some(AttackTarget::Player(p)) => e.cap.player = Some(p),
                        Some(AttackTarget::Walker(w)) => e.cap.obj = Some(w),
                        None => {}
                    }
                }
            }
        }
    }
    let (spent, colors, restricted) = cx.pay_plan(seat, src, &plan, &ctx);
    match f.ability {
        None => {
            {
                let e = entry_mut(cx, f.obj);
                e.x = ctx.x;
                e.cast.mana_spent = spent;
                e.cast.colors_spent = colors;
                e.cast.uncounterable = restricted;
                e.cast.delve_is = delve_is;
                e.cast.replicate = ctx.replicate;
            }
            let d = cx.s.obj(f.obj).def;
            let vid = cx.s.vids[f.obj.slot as usize];
            cx.s.players[seat.idx()].turn.spells_cast += 1;
            let col = cx.chars(f.obj).colors.bits() as u8;
            cx.s.players[seat.idx()].turn.cast_colors |= col;
            if !cx.db.def(d).types.contains(Types::CREATURE) {
                cx.s.players[seat.idx()].turn.noncreature_cast += 1;
            }
            cx.emit(Event::SpellCast { obj: f.obj, def: d, controller: seat, vid });
            emit_targeted(cx, f.obj, seat);
        }
        Some((src, _)) => {
            entry_mut(cx, f.obj).x = ctx.x;
            cx.emit(Event::AbilityActivated { source: src, controller: seat });
            emit_targeted(cx, f.obj, seat);
        }
    }
}

/// Announces every object the spell or ability `by` targets (ward and its kin listen for this).
pub(crate) fn emit_targeted(cx: &mut Cx, by: ObjRef, ctrl: Seat) {
    let ts: Vec<ObjRef> = match cx.s.stack.iter().find(|e| e.obj == by) {
        Some(e) => e.targets.iter().filter_map(|t| if let Target::Obj(r) = t { Some(*r) } else { None }).collect(),
        None => return,
    };
    let mut seen: Vec<ObjRef> = Vec::new();
    for r in ts {
        if seen.contains(&r) || !cx.s.is_live(r) {
            continue;
        }
        seen.push(r);
        cx.emit(Event::BecameTarget { obj: r, by, by_ctrl: ctrl });
    }
}

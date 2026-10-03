//! The stabilize frame: SBA and trigger placement until nothing changes (doc 01 sections 5.2, 7.3).

use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::frame::*;
use crate::resolve::entry_specs_for_trigger;
use crate::ids::ObjRef;
use crate::state::*;
use crate::types::ZoneKind;

pub fn run(cx: &mut Cx, f: &mut StabFrame) -> Next {
    loop {
        if cx.s.result.is_some() {
            return Next::Done;
        }
        match f.stage {
            StabStage::Sba => {
                // "As this enters, choose ...": answered before anything else can happen.
                while let Some(&r) = cx.s.enter_choices.first() {
                    if !cx.s.is_live(r) || cx.s.obj(r).zone != ZoneKind::Battlefield {
                        cx.s.enter_choices.remove(0);
                        continue;
                    }
                    let seat = cx.s.obj(r).controller;
                    let kind = cx.db.def(cx.s.obj(r).def).abilities.iter().find_map(|a| if let crate::ir::AbilityDef::EntersChoice(k) = a { Some(*k) } else { None });
                    match kind {
                        Some(crate::ir::ChoiceKind::CardName) => {
                            ask(cx, seat, DecisionKind::ChooseName { lands: true }, name_options(cx, true));
                        }
                        _ => {
                            ask(cx, seat, DecisionKind::ChooseType, type_options());
                        }
                    }
                    return Next::Await;
                }
                if cx.sba_pass() {
                    f.changed = true;
                    continue;
                }
                if let Some((seat, group)) = cx.legend_violation() {
                    ask(cx, seat, DecisionKind::LegendRule, group.into_iter().map(Opt::Card).collect());
                    return Next::Await;
                }
                f.stage = StabStage::Triggers;
            }
            StabStage::Triggers => {
                if f.cur.is_none() {
                    match next_trigger(cx) {
                        Pick::Ask(seat, opts) => {
                            ask(cx, seat, DecisionKind::OrderTriggers, opts);
                            return Next::Await;
                        }
                        Pick::Take(i) => {
                            f.cur = Some(cx.s.pending_triggers.remove(i));
                            f.slot = 0;
                            f.targets.clear();
                        }
                        Pick::None => {
                            if f.placed {
                                // Something went on the stack: SBA are checked again (CR 603.3b).
                                f.placed = false;
                                f.stage = StabStage::Sba;
                                continue;
                            }
                            break;
                        }
                    }
                }
                let pt = f.cur.clone().unwrap();
                let specs = entry_specs_for_trigger(cx.db, &pt);
                if (f.slot as usize) < specs.len() {
                    cx.refresh();
                    let spec = &specs[f.slot as usize];
                    let cands: Vec<Target> = cx.legal_targets(spec, pt.controller, Some(pt.source)).into_iter().filter(|t| !(spec.distinct && f.targets.contains(t))).collect();
                    if cands.is_empty() {
                        if spec.optional {
                            f.targets.push(Target::None);
                            f.slot += 1;
                            continue;
                        }
                        // No legal target (or a required slot is empty): the ability is removed (603.3d).
                        f.cur = None;
                        continue;
                    }
                    let mut opts: Vec<Opt> = cands.into_iter().map(Opt::Target).collect();
                    if spec.optional {
                        opts.push(Opt::Done);
                    }
                    ask(cx, pt.controller, DecisionKind::ChooseTarget { slot: f.slot }, opts);
                    return Next::Await;
                }
                let targets = std::mem::take(&mut f.targets);
                cx.push_ability(pt.source, pt.ability, pt.def, pt.controller, targets, pt.cap);
                f.cur = None;
                f.placed = true;
                f.changed = true;
            }
        }
    }
    if f.changed {
        cx.s.turn.stabilized_changed = true;
    }
    Next::Done
}

enum Pick {
    /// Index into `pending_triggers` of the one to put on the stack next.
    Take(usize),
    Ask(crate::ids::Seat, Vec<Opt>),
    None,
}

/// APNAP: the active player's triggers go first, then the non-active player's. A player with
/// several non-interchangeable triggers orders them; the last one put on the stack resolves first.
fn next_trigger(cx: &mut Cx) -> Pick {
    let active = cx.s.turn.active;
    for seat in [active, active.other()] {
        let mine: Vec<usize> = cx.s.pending_triggers.iter().enumerate().filter(|(_, t)| t.controller == seat).map(|(i, _)| i).collect();
        if mine.is_empty() {
            continue;
        }
        let mut reps: Vec<usize> = Vec::new();
        for &i in &mine {
            if !reps.iter().any(|&j| cx.interchangeable(&cx.s.pending_triggers[i], &cx.s.pending_triggers[j])) {
                reps.push(i);
            }
        }
        if reps.len() > 1 {
            return Pick::Ask(seat, reps.into_iter().map(|i| Opt::Choice(i as u8)).collect());
        }
        return Pick::Take(reps[0]);
    }
    Pick::None
}

/// The names a "choose a card name" decision offers: every real card of the pool.
pub fn name_options(cx: &Cx, lands: bool) -> Vec<Opt> {
    let mut v: Vec<(String, u16)> = Vec::new();
    for (i, d) in cx.db.defs.iter().enumerate() {
        if d.is_token || (!lands && d.types.contains(crate::types::Types::LAND)) {
            continue;
        }
        v.push((d.name.clone(), i as u16));
    }
    v.sort();
    v.into_iter().map(|(_, i)| Opt::Name(i)).collect()
}

/// The creature types a "choose a creature type" decision offers.
pub fn type_options() -> Vec<Opt> {
    (crate::types::SUB_FIRST_CREATURE_TYPE..crate::types::SUBTYPE_NAMES.len() as u8).map(Opt::Type).collect()
}

pub fn feed(cx: &mut Cx, f: &mut StabFrame, p: &Pending, opt: Opt) -> Next {
    match (p.kind, opt) {
        (DecisionKind::ChooseName { .. }, Opt::Name(d)) => {
            let r = cx.s.enter_choices.remove(0);
            if cx.s.is_live(r) {
                cx.s.obj_mut(r).chosen = d + 1;
            }
            cx.s.derived_dirty = true;
            Next::Stay
        }
        (DecisionKind::ChooseType, Opt::Type(t)) => {
            let r = cx.s.enter_choices.remove(0);
            if cx.s.is_live(r) {
                cx.s.obj_mut(r).chosen = t as u16 + 1;
            }
            cx.s.derived_dirty = true;
            Next::Stay
        }
        (DecisionKind::OrderTriggers, Opt::Choice(i)) => {
            f.cur = Some(cx.s.pending_triggers.remove(i as usize));
            f.slot = 0;
            f.targets.clear();
            Next::Stay
        }
        (DecisionKind::LegendRule, Opt::Card(keep)) => {
            let group: Vec<ObjRef> = p.options.iter().filter_map(|o| if let Opt::Card(r) = o { Some(*r) } else { None }).collect();
            let mut doomed = Vec::new();
            for r in group {
                if r != keep && cx.s.is_live(r) {
                    doomed.push((r, cx.lki(r)));
                }
            }
            for (r, l) in doomed {
                cx.commit_move(r, ZoneKind::Graveyard, crate::ops::MoveOpts::default(), Some(l));
            }
            f.changed = true;
            Next::Stay
        }
        (DecisionKind::ChooseTarget { slot }, Opt::Target(t)) => {
            f.targets.push(t);
            f.slot = slot + 1;
            Next::Stay
        }
        (DecisionKind::ChooseTarget { slot }, Opt::Done) => {
            f.targets.push(Target::None);
            f.slot = slot + 1;
            Next::Stay
        }
        _ => unreachable!("unexpected decision in stabilize frame: {:?}", p.kind),
    }
}

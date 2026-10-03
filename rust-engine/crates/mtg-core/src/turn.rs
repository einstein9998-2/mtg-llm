//! The turn machine: steps, turn-based actions, priority bookkeeping, cleanup (doc 01 section 5).

use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::event::Event;
use crate::frame::*;
use crate::ids::*;
use crate::ir::Until;
use crate::state::*;
use crate::types::*;

fn has_priority(step: Step) -> bool {
    !matches!(step, Step::Untap | Step::Cleanup)
}

pub fn run_turn(cx: &mut Cx) -> Next {
    match cx.s.turn.mode {
        TurnMode::Begin => begin_step(cx),
        TurnMode::AfterTba => {
            cx.s.turn.mode = if has_priority(cx.s.turn.step) || cx.s.turn.step == Step::Cleanup { TurnMode::Stabilize } else { TurnMode::End };
            Next::Stay
        }
        TurnMode::Stabilize => {
            cx.s.turn.mode = TurnMode::AskPriority;
            Next::Push(Frame::Stabilize(StabFrame::new()))
        }
        TurnMode::AskPriority => {
            if cx.s.turn.step == Step::Cleanup && !cx.s.turn.stabilized_changed {
                cx.s.turn.mode = TurnMode::End;
                return Next::Stay;
            }
            let seat = cx.s.turn.priority;
            let opts = cx.priority_options(seat);
            cx.s.turn.mode = TurnMode::AwaitPriority;
            ask(cx, seat, DecisionKind::Priority, opts);
            Next::Await
        }
        TurnMode::AwaitPriority => unreachable!("decision pending"),
        TurnMode::End => end_step(cx),
    }
}

fn start_turn(cx: &mut Cx, active: Seat) {
    cx.s.turn.number += 1;
    cx.s.turn.active = active;
    cx.s.turn.own_turn[active.idx()] = cx.s.turn.number;
    cx.s.turn.step = Step::Untap;
    for p in 0..2 {
        cx.s.players[p].turn = PlayerTurn::default();
        cx.s.used.clear();
    }
    // "Until your next turn" effects end as that player's turn begins.
    cx.s.effects.retain(|e| !(e.until == Until::YourNextTurn && e.controller == active));
    cx.s.player_fx.retain(|e| !(e.until == Until::YourNextTurn && e.controller == active));
    cx.s.attack_watch.retain(|w| !(w.until == Until::YourNextTurn && w.controller == active));
    cx.s.derived_dirty = true;
    cx.emit(Event::TurnBegan { turn: cx.s.turn.number, active });
}

fn begin_step(cx: &mut Cx) -> Next {
    if cx.s.turn.number == 0 {
        let first = cx.s.turn.first;
        start_turn(cx, first);
    }
    let step = cx.s.turn.step;
    cx.emit(Event::StepBegan { step });
    cx.s.turn.stabilized_changed = false;
    cx.s.turn.passes = 0;
    cx.s.turn.priority = cx.s.turn.active;
    cx.s.turn.mode = TurnMode::AfterTba;
    let active = cx.s.turn.active;
    match step {
        Step::Untap => {
            let mine: Vec<ObjRef> = cx.s.battlefield.iter().copied().filter(|&r| cx.s.obj(r).controller == active).collect();
            for r in mine {
                cx.untap(r);
            }
            Next::Stay
        }
        Step::Draw => {
            if cx.s.turn.number > 1 || active != cx.s.turn.first {
                cx.draw_n(active, 1);
            }
            Next::Stay
        }
        Step::BeginCombat => {
            cx.s.combat = Some(CombatState::default());
            Next::Stay
        }
        Step::DeclareAttackers => Next::Push(crate::combat::attackers_frame(cx)),
        Step::DeclareBlockers => Next::Push(crate::combat::blockers_frame(cx)),
        Step::FirstStrikeDamage => {
            if cx.combat_has_first_strike() {
                Next::Push(crate::combat::damage_frame(true))
            } else {
                cx.s.turn.mode = TurnMode::End;
                Next::Stay
            }
        }
        Step::CombatDamage => Next::Push(crate::combat::damage_frame(false)),
        Step::Cleanup => Next::Push(Frame::Cleanup(CleanupFrame { stage: CleanupStage::Discard })),
        Step::Upkeep | Step::Main1 | Step::EndCombat | Step::Main2 | Step::End => Next::Stay,
    }
}

fn end_step(cx: &mut Cx) -> Next {
    // Mana empties at the end of each step and phase (CR 500.5).
    for p in 0..2 {
        cx.s.players[p].pool.clear();
        cx.s.players[p].rpool.clear();
    }
    let step = cx.s.turn.step;
    let attackers_declared = cx.s.combat.as_ref().map(|c| !c.attackers.is_empty()).unwrap_or(false);
    let next = match step {
        Step::Untap => Some(Step::Upkeep),
        // CR 103.8a: the first player skips the draw step of their first turn.
        Step::Upkeep => Some(if cx.s.turn.number == 1 && cx.s.turn.active == cx.s.turn.first { Step::Main1 } else { Step::Draw }),
        Step::Draw => Some(Step::Main1),
        Step::Main1 => Some(Step::BeginCombat),
        Step::BeginCombat => Some(Step::DeclareAttackers),
        // CR 508.8: with no attackers, skip declare blockers and combat damage.
        Step::DeclareAttackers => Some(if attackers_declared { Step::DeclareBlockers } else { Step::EndCombat }),
        Step::DeclareBlockers => Some(Step::FirstStrikeDamage),
        Step::FirstStrikeDamage => Some(Step::CombatDamage),
        Step::CombatDamage => Some(Step::EndCombat),
        Step::EndCombat => Some(Step::Main2),
        Step::Main2 => Some(Step::End),
        Step::End => Some(Step::Cleanup),
        Step::Cleanup => None,
    };
    if step == Step::EndCombat {
        cx.s.combat = None;
    }
    match next {
        Some(n) => {
            cx.s.turn.step = n;
            cx.s.turn.mode = TurnMode::Begin;
        }
        None => {
            if cx.s.turn.stabilized_changed {
                // Cleanup had SBA or triggers: players got priority; another cleanup follows.
                cx.s.turn.mode = TurnMode::Begin;
            } else {
                let next_active = if let Some(s) = cx.s.turn.extra_turns.pop() { s } else { cx.s.turn.active.other() };
                start_turn(cx, next_active);
                cx.s.turn.mode = TurnMode::Begin;
            }
        }
    }
    Next::Stay
}

pub fn run_cleanup(cx: &mut Cx, f: &mut CleanupFrame) -> Next {
    let active = cx.s.turn.active;
    match f.stage {
        CleanupStage::Discard => {
            let hand = cx.s.players[active.idx()].hand.clone();
            let max = cx.s.cfg.hand_size as usize;
            let unlimited = cx.s.players[active.idx()].emblems.iter().any(|&e| cx.db.def(cx.s.obj(e).def).abilities.iter().any(|a| matches!(a, crate::ir::AbilityDef::NoHandLimit)));
            if hand.len() > max && !unlimited {
                let opts = crate::resolve::card_options(cx, active, &hand);
                let left = (hand.len() - max) as u8;
                ask(cx, active, DecisionKind::ChooseCards { purpose: CardsPurpose::DiscardToHandSize, remaining: left }, opts);
                return Next::Await;
            }
            f.stage = CleanupStage::Wear;
            Next::Stay
        }
        CleanupStage::Wear => {
            // Damage wears off and "until end of turn" effects end simultaneously (CR 514.2).
            let bf = cx.s.battlefield.clone();
            for r in bf {
                let o = cx.s.obj_mut(r);
                o.damage = 0;
                o.dt_damage = false;
            }
            cx.s.effects.retain(|e| e.until != Until::EndOfTurn);
            // Permanent changes are only kept while something they apply to is still there.
            let live = |s: &crate::state::State, r: &ObjRef| s.is_live(*r) && s.obj(*r).zone == ZoneKind::Battlefield;
            let mut effs = std::mem::take(&mut cx.s.effects);
            effs.retain(|e| e.until != Until::Forever || e.objs.iter().any(|r| live(cx.s, r)));
            cx.s.effects = effs;
            cx.s.player_fx.retain(|e| e.until != Until::EndOfTurn);
            cx.s.derived_dirty = true;
            Next::Done
        }
    }
}

pub fn feed_cleanup(cx: &mut Cx, f: &mut CleanupFrame, _p: &Pending, opt: Opt) -> Next {
    match opt {
        Opt::Card(r) => {
            cx.move_zone(r, ZoneKind::Graveyard, Default::default());
            let _ = f;
            Next::Stay
        }
        _ => unreachable!(),
    }
}

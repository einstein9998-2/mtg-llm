//! Resolution of the top stack object (CR 608) and the effect VM (doc 03 section 5).
//!
//! The VM state is `(pc, aux, loops)` inside the `ResolveFrame`, so resolution can pause for a
//! decision and be cloned mid-resolution.

use crate::card::*;
use crate::compile::Instr;
use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::eval::Env;
use crate::event::Event;
use crate::frame::*;
use crate::ids::*;
use crate::ops::MoveOpts;
use crate::state::*;
use crate::types::*;
use smallvec::SmallVec;

/// The spec list of a stack entry: the target slots of every chosen mode, in mode order.
pub fn entry_specs(db: &CardDb, e: &StackEntry) -> Vec<TargetSpec> {
    expand_x(entry_specs_raw(db, e), e.x)
}

/// Replaces each "X target" spec by X copies of itself.
fn expand_x(specs: Vec<TargetSpec>, x: u8) -> Vec<TargetSpec> {
    if !specs.iter().any(|s| s.x_count) {
        return specs;
    }
    specs.into_iter().flat_map(|s| if s.x_count { vec![TargetSpec { x_count: false, ..s }; x as usize] } else { vec![s] }).collect()
}

/// The "X target" specs of a stack entry (before expansion).
pub fn specs_raw_x(db: &CardDb, e: &StackEntry) -> Vec<TargetSpec> {
    entry_specs_raw(db, e).into_iter().filter(|s| s.x_count).collect()
}

fn entry_specs_raw(db: &CardDb, e: &StackEntry) -> Vec<TargetSpec> {
    let def = db.def(e.def);
    match e.kind {
        StackKind::Spell => match def.spell_def() {
            Some((_, sd)) => sd.modes.iter().enumerate().filter(|(i, _)| e.modes & (1 << i) != 0).flat_map(|(_, m)| m.targets.iter().cloned()).collect(),
            None => Vec::new(),
        },
        StackKind::Ability { ability, .. } => match def.abilities.get(ability as usize) {
            Some(AbilityDef::Activated(a)) => a.targets.clone(),
            Some(AbilityDef::Triggered(t)) => t.targets.clone(),
            _ => Vec::new(),
        },
    }
}

/// Target slots of a pending trigger.
pub fn entry_specs_for_trigger(db: &CardDb, t: &PendingTrigger) -> Vec<TargetSpec> {
    match db.def(t.def).abilities.get(t.ability as usize) {
        Some(AbilityDef::Triggered(td)) => td.targets.clone(),
        _ => Vec::new(),
    }
}

/// The instructions of a stack entry.
pub fn entry_code<'d>(db: &'d CardDb, e: &StackEntry) -> &'d [Instr] {
    let def = db.def(e.def);
    match e.kind {
        StackKind::Spell => def.spell_def().map(|(i, _)| def.code_of(i as u8)).unwrap_or(&[]),
        StackKind::Ability { ability, .. } => def.code_of(ability),
    }
}

/// Number of target slots in each mode of a spell, indexed by mode.
pub fn mode_slot_counts(db: &CardDb, e: &StackEntry) -> SmallVec<[u8; 4]> {
    let def = db.def(e.def);
    match (e.kind, def.spell_def()) {
        (StackKind::Spell, Some((_, sd))) => sd.modes.iter().map(|m| m.targets.len() as u8).collect(),
        _ => SmallVec::new(),
    }
}

/// Instructions one resolution may execute before the game is declared a draw (a legal script
/// loops at most over the 60,000 objects `RUNAWAY_OBJECTS` allows, a few instructions each).
pub const RESOLVE_STEP_CAP: u32 = 4_000_000;

pub fn run(cx: &mut Cx, f: &mut ResolveFrame) -> Next {
    let db = cx.db;
    loop {
        match f.stage {
            ResolveStage::Start => {
                cx.refresh();
                let specs = entry_specs(db, &f.entry);
                f.legal.clear();
                let ts = f.entry.targets.clone();
                for (i, t) in ts.iter().enumerate() {
                    let ok = match *t {
                        Target::None => false,
                        Target::Player(_) => cx.can_target_from(f.entry.controller, *t, Some(f.entry.obj)),
                        Target::Obj(r) => {
                            cx.s.is_live(r)
                                && cx.can_target_from(f.entry.controller, *t, Some(f.entry.obj))
                                && specs.get(i).map(|sp| target_still_matches(cx, sp, r, f.entry.controller, f.entry.obj)).unwrap_or(false)
                        }
                    };
                    f.legal.push(ok);
                }
                f.tlki.clear();
                for t in ts.iter() {
                    let l = match *t {
                        Target::Obj(r) if cx.s.is_live(r) => Some(cx.lki(r)),
                        _ => None,
                    };
                    f.tlki.push(l);
                }
                if let StackKind::Ability { ability, .. } = f.entry.kind {
                    // Intervening "if" is rechecked on resolution (CR 603.4).
                    if let Some(AbilityDef::Triggered(TriggeredDef { cond: Some(c), event_cond: false, .. })) = db.def(f.entry.def).abilities.get(ability as usize) {
                        let env = Env {
                            controller: f.entry.controller,
                            source: source_of(&f.entry),
                            source_lki: f.entry.cap.lki.as_ref(),
                            targets: &f.entry.targets,
                            legal: &f.legal,
                            base: 0,
                            x: 0,
                            cap: &f.entry.cap,
                            each: [None, None],
                            tlki: &f.tlki,
                        };
                        if !cx.eval_cond(&env, c) {
                            f.stage = ResolveStage::Finish;
                            continue;
                        }
                    }
                }
                if f.entry.targets.iter().any(|t| *t != Target::None) && !f.legal.iter().any(|&b| b) {
                    f.fizzled = true;
                    cx.emit(Event::SpellFizzled { obj: f.entry.obj });
                    f.stage = ResolveStage::Finish;
                } else {
                    f.stage = ResolveStage::Ops;
                }
            }
            ResolveStage::Ops => {
                let code = entry_code(db, &f.entry);
                let mut steps = 0u32;
                while (f.pc as usize) < code.len() {
                    // A resolution is bounded by the objects in the game; far past that it is a
                    // runaway (a loop that cannot end), so the game is declared a draw instead of
                    // looping or exhausting memory (RFC 0003). Counted per run, so a decision
                    // inside the script restarts the count: the cap guards loops, not scripts.
                    steps += 1;
                    if steps > RESOLVE_STEP_CAP {
                        cx.s.result.get_or_insert(crate::decision::GameResult::Draw);
                        return Next::Done;
                    }
                    match step(cx, f, &code[f.pc as usize]) {
                        Flow::Next => {
                            f.pc += 1;
                            f.aux = 0;
                        }
                        Flow::Jump(t) => {
                            f.pc = t;
                            f.aux = 0;
                        }
                        Flow::Await => return Next::Await,
                    }
                    if cx.s.result.is_some() {
                        return Next::Done;
                    }
                }
                f.stage = ResolveStage::Finish;
            }
            ResolveStage::Finish => {
                finish(cx, f);
                return Next::Done;
            }
        }
    }
}

fn target_still_matches(cx: &mut Cx, spec: &TargetSpec, r: ObjRef, ctrl: Seat, source: ObjRef) -> bool {
    match &spec.objects {
        Some(filt) => cx.matches_filter(filt, r, Some(source), ctrl),
        None => false,
    }
}

enum Flow {
    Next,
    Jump(u16),
    Await,
}

fn source_of(e: &StackEntry) -> Option<ObjRef> {
    match e.kind {
        StackKind::Spell => Some(e.obj),
        StackKind::Ability { source, .. } => Some(source),
    }
}

fn step(cx: &mut Cx, f: &mut ResolveFrame, ins: &Instr) -> Flow {
    // The env borrows the entry; loop/aux state is mutated through disjoint fields.
    let each = [f.loops[0].list.get(f.loops[0].idx.saturating_sub(1) as usize).copied(), f.loops[1].list.get(f.loops[1].idx.saturating_sub(1) as usize).copied()];
    let env = Env {
        controller: f.entry.controller,
        source: source_of(&f.entry),
        source_lki: f.entry.cap.lki.as_ref().filter(|_| matches!(f.entry.kind, StackKind::Ability { .. })),
        targets: &f.entry.targets,
        legal: &f.legal,
        base: f.base as usize,
        x: f.entry.x as i32,
        cap: &f.entry.cap,
        each,
        tlki: &f.tlki,
    };
    match ins {
        Instr::Jump(t) => Flow::Jump(*t),
        Instr::JumpUnless(c, t) => {
            if cx.eval_cond(&env, c) {
                Flow::Next
            } else {
                Flow::Jump(*t)
            }
        }
        Instr::Mode { i, skip } => {
            if f.entry.modes & (1 << *i) == 0 {
                return Flow::Jump(*skip);
            }
            let counts = mode_slot_counts(cx.db, &f.entry);
            let base: u8 = counts.iter().enumerate().filter(|(j, _)| *j < *i as usize && f.entry.modes & (1 << *j) != 0).map(|(_, c)| *c).sum();
            f.base = base;
            Flow::Next
        }
        Instr::May { who, skip } => {
            let p = match cx.eval_p(&env, *who) {
                Some(p) => p,
                None => return Flow::Jump(*skip),
            };
            match f.aux {
                0 => {
                    ask(cx, p, DecisionKind::May, vec![Opt::Yes, Opt::No]);
                    Flow::Await
                }
                1 => Flow::Next,
                _ => Flow::Jump(*skip),
            }
        }
        Instr::MayPay { who, pays, skip } => {
            let p = match cx.eval_p(&env, *who) {
                Some(p) => p,
                None => return Flow::Jump(*skip),
            };
            match f.aux {
                0 => {
                    if cx.plan_cost(p, pays, 0, None, None).is_some() {
                        ask(cx, p, DecisionKind::PayUnless, vec![Opt::Yes, Opt::No]);
                        Flow::Await
                    } else {
                        Flow::Jump(*skip)
                    }
                }
                1 => {
                    let (pay, srcs) = cx.plan_cost(p, pays, 0, None, None).expect("payment was offered, so it is possible");
                    cx.apply_payment(p, &pay, &srcs, &[]);
                    Flow::Next
                }
                _ => Flow::Jump(*skip),
            }
        }
        Instr::LoopInit { filter, depth } => {
            let list = cx.select(&env, filter);
            let st = &mut f.loops[*depth as usize];
            st.list = list.into_iter().collect();
            st.idx = 0;
            Flow::Next
        }
        Instr::LoopNext { depth, end } => {
            let st = &mut f.loops[*depth as usize];
            // Skip members that left the zone since the snapshot.
            while (st.idx as usize) < st.list.len() {
                let r = st.list[st.idx as usize];
                st.idx += 1;
                if cx.s.is_live(r) {
                    return Flow::Next;
                }
            }
            Flow::Jump(*end)
        }
        Instr::PlayerLoopInit { depth } => {
            f.loops[*depth as usize].count = 0;
            Flow::Next
        }
        Instr::PlayerLoopNext { depth, end } => {
            let i = f.loops[*depth as usize].count;
            if i >= 2 {
                cx.s.each_player = None;
                return Flow::Jump(*end);
            }
            f.loops[*depth as usize].count += 1;
            let a = cx.s.turn.active;
            cx.s.each_player = Some(if i == 0 { a } else { a.other() });
            Flow::Next
        }
        Instr::RepeatInit { n, depth } => {
            let n = cx.eval_expr(&env, n).max(0);
            f.loops[*depth as usize].count = n;
            Flow::Next
        }
        Instr::RepeatNext { depth, end } => {
            let st = &mut f.loops[*depth as usize];
            if st.count > 0 {
                st.count -= 1;
                Flow::Next
            } else {
                Flow::Jump(*end)
            }
        }
        Instr::Leaf(e) => leaf(cx, &env, &mut f.aux, &mut f.sc, e),
    }
}

fn wait(w: bool) -> Flow {
    if w {
        Flow::Await
    } else {
        Flow::Next
    }
}

fn objs_of(cx: &mut Cx, env: &Env, o: &Objs) -> Vec<ObjRef> {
    match o {
        Objs::One(r) => cx.eval_o(env, *r).into_iter().collect(),
        Objs::All(f) => cx.select(env, f),
        Objs::TargetsFrom(n) => (*n..env.targets.len().saturating_sub(env.base) as u8).filter_map(|i| match env.target(i) {
            Some(Target::Obj(r)) if cx.s.is_live(r) => Some(r),
            _ => None,
        }).collect(),
        Objs::Captured => env.cap.objs.iter().copied().filter(|&r| cx.s.is_live(r)).collect(),
        Objs::TopOfGraveyard { who, filter } => match cx.eval_p(env, *who) {
            Some(p) => {
                let gy = cx.s.players[p.idx()].graveyard.clone();
                for &r in gy.iter().rev() {
                    let c = cx.chars(r);
                    if cx.filter_match(filter, r, &c, false, env.controller) {
                        return vec![r];
                    }
                }
                Vec::new()
            }
            None => Vec::new(),
        },
    }
}

/// Destroys every permanent matching `filter` whose mana value is at most `paid`.
fn energy_wrath_destroy(cx: &mut Cx, env: &Env, filter: &ObjFilter, paid: u16) {
    let mut f = filter.clone();
    f.cmc = Some((Cmp::Le, paid as i32));
    for r in cx.select(env, &f) {
        if !cx.s.is_live(r) {
            continue;
        }
        cx.refresh();
        if !cx.chars(r).keywords.contains(Keywords::INDESTRUCTIBLE) {
            cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
        }
    }
}

fn leaf(cx: &mut Cx, env: &Env, aux: &mut u16, sc: &mut [u16; 3], e: &Effect) -> Flow {
    let src = env.source;
    match e {
        Effect::Damage { amount, to } => {
            let n = cx.eval_expr(env, amount).max(0) as u32;
            let target = match to {
                Rcpt::Target(i) => env.target(*i),
                Rcpt::Player(p) => cx.eval_p(env, *p).map(Target::Player),
                Rcpt::Obj(o) => cx.eval_o(env, *o).map(Target::Obj),
            };
            if let Some(t) = target {
                if let Target::Obj(r) = t {
                    if !cx.s.is_live(r) {
                        return Flow::Next;
                    }
                }
                cx.deal_damage(src, t, n, false);
            }
            Flow::Next
        }
        Effect::Draw { who, n } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let n = cx.eval_expr(env, n);
                cx.draw_n(p, n);
            }
            Flow::Next
        }
        Effect::GainLife { who, n } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let n = cx.eval_expr(env, n).max(0) as u32;
                cx.gain_life(p, n);
            }
            Flow::Next
        }
        Effect::LoseLife { who, n } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let n = cx.eval_expr(env, n).max(0) as u32;
                cx.lose_life(p, n);
            }
            Flow::Next
        }
        Effect::Destroy(o) => {
            let objs = objs_of(cx, env, o);
            cx.begin_batch(&objs);
            for r in objs {
                if !cx.s.is_live(r) {
                    continue;
                }
                cx.refresh();
                if !cx.chars(r).keywords.contains(Keywords::INDESTRUCTIBLE) {
                    cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
                }
            }
            cx.end_batch();
            Flow::Next
        }
        Effect::Exile(o) => {
            cx.s.moved.clear();
            let objs = objs_of(cx, env, o);
            cx.begin_batch(&objs);
            for r in objs {
                if cx.s.is_live(r) {
                    if let Some(nr) = cx.move_zone(r, ZoneKind::Exile, MoveOpts::default()) {
                        cx.s.moved.push(nr);
                    }
                }
            }
            cx.end_batch();
            Flow::Next
        }
        Effect::ExileUntil { objs: o, when, ability } => {
            cx.s.moved.clear();
            let mut cap = Captured::default();
            let targets = objs_of(cx, env, o);
            let any = !targets.is_empty();
            cx.begin_batch(&targets);
            for r in targets {
                if cx.s.is_live(r) {
                    if let Some(nr) = cx.move_zone(r, ZoneKind::Exile, MoveOpts::default()) {
                        cx.s.moved.push(nr);
                        cap.objs.push(nr);
                    }
                }
            }
            cx.end_batch();
            // The delayed trigger exists even if the exiled object (a token) cannot come back.
            if any {
                cx.add_delayed(env, *when, *ability, cap);
            }
            Flow::Next
        }
        Effect::DelayEventObj { when, ability } => {
            let mut cap = Captured::default();
            cap.objs.extend(env.cap.obj);
            if !cap.objs.is_empty() {
                cx.add_delayed(env, *when, *ability, cap);
            }
            Flow::Next
        }
        Effect::RingTempts => {
            let me = env.controller;
            if *aux != 0 {
                return Flow::Next;
            }
            if cx.s.players[me.idx()].ring == 0 {
                let def = cx.db.id("The Ring").expect("the Ring emblem is in the pool");
                let r = cx.s.alloc_slot(ObjKind::Token, def, me);
                cx.s.obj_mut_raw(r).zone = ZoneKind::Command;
                let mut vid = [ViewId::NONE; 2];
                for v in 0..2usize {
                    vid[v] = ViewId(cx.s.next_vid[v]);
                    cx.s.next_vid[v] += 1;
                }
                cx.s.vids[r.slot as usize] = vid;
                cx.s.players[me.idx()].emblems.push(r);
            }
            let lvl = &mut cx.s.players[me.idx()].ring;
            *lvl = (*lvl + 1).min(4);
            cx.refresh();
            let cands: Vec<ObjRef> = cx.s.battlefield.iter().copied().filter(|&r| cx.s.obj(r).controller == me && cx.s.derived[r.slot as usize].types.contains(Types::CREATURE)).collect();
            let opts = card_options(cx, me, &cands);
            if opts.is_empty() {
                return Flow::Next;
            }
            ask(cx, me, DecisionKind::ChooseCards { purpose: CardsPurpose::RingBearer, remaining: 1 }, opts);
            Flow::Await
        }
        Effect::WatchAttacks { ability, until } => {
            if let Some(source) = env.source {
                let def = cx.s.stack.last().map(|e| e.def).unwrap_or(cx.s.objs[source.slot as usize].def);
                cx.s.attack_watch.push(AttackWatch { source, def, ability: *ability, controller: env.controller, until: *until });
            }
            Flow::Next
        }
        Effect::Delay { when, ability } => {
            cx.add_delayed(env, *when, *ability, Captured::default());
            Flow::Next
        }
        Effect::Attach { what, to } => {
            if let (Some(a), Some(t)) = (cx.eval_o(env, *what), cx.eval_o(env, *to)) {
                cx.refresh();
                if cx.s.obj(a).zone == ZoneKind::Battlefield && cx.s.obj(t).zone == ZoneKind::Battlefield && cx.chars(t).types.contains(Types::CREATURE) {
                    cx.s.obj_mut(a).attached_to = Some(t);
                    cx.s.derived_dirty = true;
                }
            }
            Flow::Next
        }
        Effect::LookTop { who } => {
            if let Some(p) = cx.eval_p(env, *who) {
                cx.look_top(env.controller, p);
            }
            Flow::Next
        }
        Effect::Bounce(o) => {
            cx.s.moved.clear();
            for r in objs_of(cx, env, o) {
                if cx.s.is_live(r) {
                    if let Some(nr) = cx.move_zone(r, ZoneKind::Hand, MoveOpts::default()) {
                        cx.s.moved.push(nr);
                    }
                }
            }
            Flow::Next
        }
        Effect::CounterSpell(i) => {
            if let Some(Target::Obj(r)) = env.target(*i) {
                if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Stack && cx.can_be_countered(r) {
                    cx.emit(Event::SpellCountered { obj: r });
                    counter_dest(cx, r, ZoneKind::Graveyard);
                }
            }
            Flow::Next
        }
        Effect::CounterSpellOf(o) => {
            if let Some(r) = cx.eval_o(env, *o) {
                if cx.s.obj(r).zone == ZoneKind::Stack && cx.can_be_countered(r) {
                    cx.emit(Event::SpellCountered { obj: r });
                    counter_dest(cx, r, ZoneKind::Graveyard);
                }
            }
            Flow::Next
        }
        Effect::CounterUnless { target, pays } => {
            let victim = match env.target(*target) {
                Some(Target::Obj(r)) if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Stack => r,
                _ => return Flow::Next,
            };
            counter_unless(cx, victim, pays, *aux)
        }
        Effect::CounterEventUnless { pays } => {
            let victim = match env.cap.obj {
                Some(r) if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Stack => r,
                _ => return Flow::Next,
            };
            counter_unless(cx, victim, pays, *aux)
        }
        Effect::Mill { who, n } => {
            cx.s.moved.clear();
            if let Some(p) = cx.eval_p(env, *who) {
                for _ in 0..cx.eval_expr(env, n).max(0) {
                    if let Some(r) = cx.mill_one_ref(p) {
                        cx.s.moved.push(r);
                    }
                }
            }
            Flow::Next
        }
        Effect::CreateToken { token, n, who } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let def = cx.db.id(token).unwrap_or_else(|| panic!("unknown token {token:?}"));
                cx.s.moved.clear();
                for _ in 0..cx.eval_expr(env, n).max(0) {
                    if let Some(t) = cx.create_token(def, p) {
                        cx.s.moved.push(t);
                    }
                }
            }
            Flow::Next
        }
        Effect::AddCounters { objs, kind, n } => {
            let n = cx.eval_expr(env, n);
            for r in objs_of(cx, env, objs) {
                cx.add_counters(r, *kind, n);
            }
            Flow::Next
        }
        Effect::Tap(o) => {
            for r in objs_of(cx, env, o) {
                if cx.s.is_live(r) {
                    cx.tap(r);
                }
            }
            Flow::Next
        }
        Effect::Untap(o) => {
            for r in objs_of(cx, env, o) {
                if cx.s.is_live(r) {
                    cx.untap(r);
                }
            }
            Flow::Next
        }
        Effect::Discard { who, n } => {
            let p = match cx.eval_p(env, *who) {
                Some(p) => p,
                None => return Flow::Next,
            };
            let total = cx.eval_expr(env, n).max(0) as usize;
            let left = total.saturating_sub(*aux as usize);
            let hand_len = cx.s.players[p.idx()].hand.len();
            if left == 0 || hand_len == 0 {
                return Flow::Next;
            }
            if hand_len < left {
                // Fewer cards than asked for: discard everything, no choice to make.
                let hand: Vec<ObjRef> = cx.s.players[p.idx()].hand.clone();
                for r in hand {
                    cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
                }
                return Flow::Next;
            }
            let opts = card_options(cx, p, &cx.s.players[p.idx()].hand.clone());
            ask(cx, p, DecisionKind::ChooseCards { purpose: CardsPurpose::DiscardEffect, remaining: left as u8 }, opts);
            Flow::Await
        }
        Effect::CounterAny(i) => {
            if let Some(Target::Obj(r)) = env.target(*i) {
                if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Stack {
                    if cx.s.obj(r).kind == ObjKind::Ability {
                        cx.s.stack.retain(|x| x.obj != r);
                        cx.commit_move(r, ZoneKind::Gone, MoveOpts { quiet: true, ..Default::default() }, None);
                    } else if cx.can_be_countered(r) {
                        cx.emit(Event::SpellCountered { obj: r });
                        counter_dest(cx, r, ZoneKind::Graveyard);
                    }
                }
            }
            Flow::Next
        }
        Effect::CopySpell { what, n } => {
            // aux: 0 = make the next copy, 2 = answered "change targets?" is pending, 3 = choosing
            // new targets slot by slot for the newest copy, 4 = this copy is done.
            if *aux == 0 {
                if sc[2] == 0 {
                    // First visit: how many copies.
                    sc[0] = cx.eval_expr(env, n).clamp(0, 200) as u16;
                    sc[2] = 1;
                }
                if sc[0] == 0 {
                    sc[2] = 0;
                    return Flow::Next;
                }
                let orig = match cx.eval_o(env, *what) {
                    Some(r) => r,
                    None => {
                        sc[2] = 0;
                        return Flow::Next;
                    }
                };
                let e = match cx.s.stack.iter().find(|e| e.obj == orig) {
                    Some(e) => e.clone(),
                    None => {
                        sc[2] = 0;
                        return Flow::Next;
                    }
                };
                cx.make_spell_copy(&e, env.controller);
                sc[0] -= 1;
                sc[1] = 0;
                let copy = cx.s.stack.last().cloned().expect("a copy was just pushed");
                if entry_specs(cx.db, &copy).is_empty() {
                    *aux = 4;
                } else {
                    ask(cx, copy.controller, DecisionKind::ChangeTargets, vec![Opt::Yes, Opt::No]);
                    *aux = 2;
                    return Flow::Await;
                }
            }
            if *aux == 3 {
                // Choose new targets for the newest copy, slot by slot.
                let copy = cx.s.stack.last().cloned().expect("a copy is on the stack");
                let specs = entry_specs(cx.db, &copy);
                loop {
                    let slot = sc[1] as usize;
                    if slot >= specs.len() {
                        *aux = 4;
                        break;
                    }
                    let spec = &specs[slot];
                    let cands: Vec<Target> = cx.legal_targets(spec, copy.controller, Some(copy.obj)).into_iter().filter(|t| *t != Target::Obj(copy.obj)).collect();
                    if cands.is_empty() {
                        sc[1] += 1;
                        continue;
                    }
                    let mut opts: Vec<Opt> = cands.into_iter().map(Opt::Target).collect();
                    if spec.optional {
                        opts.push(Opt::Done);
                    }
                    ask(cx, copy.controller, DecisionKind::ChooseTarget { slot: slot as u8 }, opts);
                    return Flow::Await;
                }
            }
            // This copy is done: the next one, or finished.
            *aux = 0;
            if sc[0] == 0 {
                sc[2] = 0;
                return Flow::Next;
            }
            leaf(cx, env, aux, sc, &Effect::CopySpell { what: *what, n: n.clone() })
        }
        Effect::RevealPickTypes { look } => wait(cx.fx_reveal_pick_types(env, look, aux, sc)),
        Effect::AddManaRestricted => Flow::Next,
        Effect::ExileUntilLeaves { creature } => {
            let me = env.controller;
            if *aux != 0 {
                return Flow::Next;
            }
            let victim = match env.target(0) {
                Some(Target::Player(p)) => p,
                _ => return Flow::Next,
            };
            let mut cands: Vec<ObjRef> = Vec::new();
            cx.reveal_hand_to(me, victim);
            for h in cx.s.players[victim.idx()].hand.clone() {
                if !cx.db.def(cx.s.obj(h).def).types.contains(Types::LAND) {
                    cands.push(h);
                }
            }
            if let Some(Target::Obj(c)) = env.target(*creature) {
                if cx.s.is_live(c) && cx.s.obj(c).zone == ZoneKind::Battlefield {
                    cands.push(c);
                }
            }
            if cands.is_empty() {
                return Flow::Next;
            }
            let mut opts = card_options(cx, me, &cands);
            opts.push(Opt::Done);
            ask(cx, me, DecisionKind::ChooseCards { purpose: CardsPurpose::ExileUntilLeaves, remaining: 1 }, opts);
            Flow::Await
        }
        Effect::CastFromGraveyard { filter } => {
            let p = env.controller;
            if *aux != 0 {
                return Flow::Next;
            }
            let gy = cx.s.players[p.idx()].graveyard.clone();
            let mut ok = Vec::new();
            for r in gy {
                let c = cx.chars(r);
                if cx.filter_match(filter, r, &c, false, p) && !cx.cast_restricted(p, r) && cx.way_feasible_x(p, r, crate::restrict::GY_WAY, false) {
                    ok.push(r);
                }
            }
            if ok.is_empty() {
                return Flow::Next;
            }
            let mut opts = card_options(cx, p, &ok);
            opts.push(Opt::Done);
            ask(cx, p, DecisionKind::ChooseCards { purpose: CardsPurpose::CastFromGraveyard, remaining: 1 }, opts);
            Flow::Await
        }
        Effect::EnergyWrath { filter } => {
            let p = env.controller;
            match *aux {
                0 => {
                    cx.s.players[p.idx()].energy += env.x.max(0) as u32;
                    let have = cx.s.players[p.idx()].energy;
                    if have == 0 {
                        sc[0] = 0;
                        *aux = 1;
                    } else {
                        let opts: Vec<Opt> = (0..=have).map(Opt::Number).collect();
                        ask(cx, p, DecisionKind::PayEnergyAmount, opts);
                        return Flow::Await;
                    }
                    energy_wrath_destroy(cx, env, filter, sc[0]);
                    Flow::Next
                }
                _ => {
                    energy_wrath_destroy(cx, env, filter, sc[0]);
                    Flow::Next
                }
            }
        }
        Effect::ExileCastEnergy => {
            let p = env.controller;
            if *aux != 0 {
                return Flow::Next;
            }
            let mut found: Option<ObjRef> = None;
            while let Some(top) = cx.s.players[p.idx()].library.last().copied() {
                let is_land = cx.db.def(cx.s.obj(top).def).types.contains(Types::LAND);
                match cx.move_zone(top, ZoneKind::Exile, MoveOpts::default()) {
                    Some(nr) => {
                        if !is_land {
                            found = Some(nr);
                            break;
                        }
                    }
                    None => break,
                }
            }
            if let Some(r) = found {
                if cx.s.is_live(r) && !cx.cast_restricted(p, r) && cx.way_feasible_x(p, r, crate::restrict::ENERGY_WAY, false) {
                    let mut opts = card_options(cx, p, &[r]);
                    opts.push(Opt::Done);
                    ask(cx, p, DecisionKind::ChooseCards { purpose: CardsPurpose::CastFromExile, remaining: 1 }, opts);
                    return Flow::Await;
                }
            }
            Flow::Next
        }
        Effect::CopyAs { objs, pt, color, subtype } => {
            let list = objs_of(cx, env, objs);
            cx.s.moved.clear();
            for r in list {
                if cx.s.is_live(r) {
                    let def = cx.s.obj(r).def;
                    if let Some(t) = cx.create_token(def, env.controller) {
                        cx.s.moved.push(t);
                        let ts = cx.s.next_ts();
                        for (layer, effect) in [(Layer::L4, ContEffect::SetColors(*color)), (Layer::L4, ContEffect::SetCreatureType(*subtype)), (Layer::L7b, ContEffect::SetPT(pt.0, pt.1))] {
                            cx.s.effects.push(ContInst { ts, layer, objs: [t].into_iter().collect(), effect, until: Until::Forever, controller: env.controller });
                        }
                        cx.s.derived_dirty = true;
                    }
                }
            }
            Flow::Next
        }
        Effect::ExileReturnTransformed => {
            let src = match env.source {
                Some(s) if cx.s.is_live(s) && cx.s.obj(s).zone == ZoneKind::Battlefield => s,
                _ => return Flow::Next,
            };
            let owner = cx.s.obj(src).owner;
            if let Some(nr) = cx.move_zone(src, ZoneKind::Exile, MoveOpts::default()) {
                if cx.s.is_live(nr) && cx.s.obj(nr).zone == ZoneKind::Exile {
                    cx.move_zone(nr, ZoneKind::Battlefield, MoveOpts { controller: Some(owner), face: true, ..Default::default() });
                }
            }
            Flow::Next
        }
        Effect::KeepOneOfEach => {
            let opp = env.controller.other();
            const KINDS: [Types; 4] = [Types::ARTIFACT, Types::CREATURE, Types::ENCHANTMENT, Types::PLANESWALKER];
            if *aux == 0 && sc[0] == 0 {
                cx.s.moved.clear();
                sc[0] = 1;
            }
            cx.refresh();
            loop {
                let stage = *aux as usize;
                if stage >= 4 {
                    break;
                }
                let cands: Vec<ObjRef> = cx.s.battlefield.iter().copied().filter(|&r| {
                    let c = cx.s.derived[r.slot as usize];
                    cx.s.obj(r).controller == opp && !c.types.contains(Types::LAND) && c.types.contains(KINDS[stage])
                }).collect();
                match cands.len() {
                    0 => {
                        *aux += 1;
                    }
                    1 => {
                        cx.s.moved.push(cands[0]);
                        *aux += 1;
                    }
                    _ => {
                        let opts = card_options(cx, opp, &cands);
                        if opts.len() == 1 {
                            if let Opt::Card(r) = opts[0] {
                                cx.s.moved.push(r);
                            }
                            *aux += 1;
                            continue;
                        }
                        ask(cx, opp, DecisionKind::ChooseCards { purpose: CardsPurpose::KeepOne, remaining: 1 }, opts);
                        return Flow::Await;
                    }
                }
            }
            let kept = cx.s.moved.clone();
            let doomed: Vec<ObjRef> = cx.s.battlefield.iter().copied().filter(|&r| cx.s.obj(r).controller == opp && !cx.s.derived[r.slot as usize].types.contains(Types::LAND) && !kept.iter().any(|&k| k == r || (k.slot == r.slot))).collect();
            for r in doomed {
                if cx.s.is_live(r) {
                    cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
                }
            }
            sc[0] = 0;
            Flow::Next
        }
        Effect::DigCreatureAttacking => {
            let me = env.controller;
            let opp = me.other();
            if *aux == 1 {
                // The attack target has been chosen; everything else was done before asking.
                return Flow::Next;
            }
            let mut rest: Vec<ObjRef> = Vec::new();
            let mut found: Option<ObjRef> = None;
            let mut entered: Option<ObjRef> = None;
            while let Some(top) = cx.s.players[me.idx()].library.last().copied() {
                cx.reveal_to(opp, top);
                cx.reveal_to(me, top);
                let is_creature = cx.db.def(cx.s.obj(top).def).types.contains(Types::CREATURE);
                // Take it out of the library (it is revealed, not yet anywhere else).
                cx.s.players[me.idx()].library.pop();
                if is_creature {
                    found = Some(top);
                    break;
                }
                rest.push(top);
            }
            // Put everything back so the zone changes are ordinary moves: the creature on top, the rest below it.
            cx.random_bottom_order(me, &mut rest);
            for &r in rest.iter() {
                cx.s.players[me.idx()].library.push(r);
            }
            if let Some(c) = found {
                cx.s.players[me.idx()].library.push(c);
            }
            if let Some(c) = found {
                if let Some(nr) = cx.move_zone(c, ZoneKind::Battlefield, MoveOpts { controller: Some(me), tapped: true, ..Default::default() }) {
                    if cx.s.is_live(nr) && cx.s.obj(nr).zone == ZoneKind::Battlefield {
                        entered = Some(nr);
                    }
                }
            }
            for r in rest {
                if cx.s.is_live(r) {
                    cx.move_zone(r, ZoneKind::Library, MoveOpts { to_bottom: true, ..Default::default() });
                }
            }
            // The new attacker's controller chooses what it attacks (508.4).
            cx.s.moved.clear();
            if let Some(nr) = entered {
                cx.s.moved.push(nr);
                let opts = crate::combat::attack_target_opts(cx, opp);
                if opts.len() > 1 {
                    ask(cx, me, DecisionKind::DeclareAttacker { creature: nr }, opts);
                    return Flow::Await;
                }
                put_attacking(cx, nr, AttackTarget::Player(opp));
            }
            Flow::Next
        }
        Effect::GambitReveal => {
            let active = cx.s.turn.active;
            let order = [active, active.other()];
            loop {
                let stage = *aux as usize;
                if stage >= 2 {
                    break;
                }
                let seat = order[stage];
                let hand = cx.s.players[seat.idx()].hand.clone();
                if hand.is_empty() {
                    sc[stage] = 0;
                    *aux += 1;
                    continue;
                }
                let opts = card_options(cx, seat, &hand);
                ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::GambitChoose, remaining: 1 }, opts);
                return Flow::Await;
            }
            let chosen: Vec<ObjRef> = (0..2).filter(|&i| sc[i] != 0).map(|i| cx.s.cur_ref(sc[i] - 1)).filter(|&r| cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Hand).collect();
            for &r in &chosen {
                let owner = cx.s.obj(r).owner;
                cx.reveal_to(owner.other(), r);
            }
            let creatures: Vec<ObjRef> = chosen.iter().copied().filter(|&r| cx.db.def(cx.s.obj(r).def).types.contains(Types::CREATURE)).collect();
            if let Some(low) = creatures.iter().map(|&r| cx.db.def(cx.s.obj(r).def).mana_value()).min() {
                // Both owners put theirs onto the battlefield as one event (614.12): neither sees the
                // other's creature when it enters (the text does not say "simultaneously"; we read it so).
                cx.begin_batch(&[]);
                for r in creatures {
                    if cx.db.def(cx.s.obj(r).def).mana_value() == low && cx.s.is_live(r) {
                        cx.move_zone(r, ZoneKind::Battlefield, MoveOpts::default());
                    }
                }
                cx.end_batch();
            }
            Flow::Next
        }
        Effect::MiracleCast(w) => {
            let p = env.controller;
            if *aux != 0 {
                return Flow::Next;
            }
            match env.source {
                // The miracle is only offered when its cost can be paid (a player who cannot pay
                // would not reveal); revealing is otherwise forced here (known deviation, 702.94a).
                Some(s) if cx.s.is_live(s) && cx.s.obj(s).zone == ZoneKind::Hand && !cx.cast_restricted(p, s) && cx.way_feasible_inner(p, s, *w, false, false) => {
                    // Revealing the card is part of the choice: the opponent learns it either way.
                    cx.reveal_to(p.other(), s);
                    ask(cx, p, DecisionKind::ChooseCards { purpose: CardsPurpose::CastMiracle, remaining: 1 }, vec![Opt::Card(s), Opt::Done]);
                    Flow::Await
                }
                _ => Flow::Next,
            }
        }
        Effect::PileBack => {
            let card = match env.cap.obj {
                Some(c) if cx.s.is_live(c) && cx.s.obj(c).zone == ZoneKind::Graveyard => c,
                _ => return Flow::Next,
            };
            let owner = cx.s.obj(card).owner;
            // Exiled face down and put back on top, the card and the six cards below it end up in a
            // random order at the top: the six never become visible, so only the shuffle is done.
            if cx.move_zone(card, ZoneKind::Library, MoveOpts::default()).is_some() {
                let lib = &mut cx.s.players[owner.idx()].library;
                let n = lib.len().min(7);
                let at = lib.len() - n;
                let mut top: Vec<ObjRef> = lib[at..].to_vec();
                cx.s.rng.shuffle(&mut top);
                let lib = &mut cx.s.players[owner.idx()].library;
                lib.truncate(at);
                lib.extend(top.iter().copied());
                for r in top {
                    cx.s.knowledge[r.slot as usize].pos_known_to = 0;
                }
            }
            Flow::Next
        }
        Effect::NinjutsuEnter => {
            let src = match env.source {
                Some(s) if cx.s.is_live(s) && cx.s.obj(s).zone == ZoneKind::Hand => s,
                _ => return Flow::Next,
            };
            let target = match (env.cap.obj, env.cap.player) {
                (Some(w), _) => AttackTarget::Walker(w),
                (None, Some(p)) => AttackTarget::Player(p),
                _ => return Flow::Next,
            };
            if let Some(nr) = cx.move_zone(src, ZoneKind::Battlefield, MoveOpts { controller: Some(env.controller), tapped: true, ..Default::default() }) {
                if cx.s.is_live(nr) && cx.s.obj(nr).zone == ZoneKind::Battlefield {
                    if let Some(cb) = cx.s.combat.as_mut() {
                        cb.attackers.push(AttackerInfo { obj: nr, target, blockers: smallvec::SmallVec::new(), blocked: false });
                    }
                }
            }
            Flow::Next
        }
        Effect::CreateEmblem(name) => {
            let def = cx.db.id(name).unwrap_or_else(|| panic!("unknown emblem {name:?}"));
            let p = env.controller;
            let r = cx.s.alloc_slot(ObjKind::Token, def, p);
            cx.s.obj_mut_raw(r).zone = ZoneKind::Command;
            let mut vid = [ViewId::NONE; 2];
            for v in 0..2usize {
                vid[v] = ViewId(cx.s.next_vid[v]);
                cx.s.next_vid[v] += 1;
            }
            cx.s.vids[r.slot as usize] = vid;
            cx.s.players[p.idx()].emblems.push(r);
            cx.s.derived_dirty = true;
            Flow::Next
        }
        Effect::ReanimateAttach => {
            if let Some(src) = env.source {
                if cx.s.is_live(src) && cx.s.obj(src).zone == ZoneKind::Battlefield {
                    if let Some(t) = cx.s.obj(src).attached_to {
                        if cx.s.is_live(t) && cx.s.obj(t).zone == ZoneKind::Graveyard {
                            if let Some(nr) = cx.move_zone(t, ZoneKind::Battlefield, MoveOpts { controller: Some(env.controller), ..Default::default() }) {
                                if cx.s.is_live(src) && cx.s.obj(src).zone == ZoneKind::Battlefield {
                                    cx.s.obj_mut(src).attached_to = Some(nr);
                                    cx.s.derived_dirty = true;
                                }
                            }
                        }
                    }
                    // Whatever it enchanted before, the Aura now needs a creature put onto the
                    // battlefield with it; if the return failed (Grafdigger's Cage) nothing fits and
                    // state-based actions put it into the graveyard (704.5m).
                    if cx.s.is_live(src) && cx.s.obj(src).zone == ZoneKind::Battlefield {
                        let ok = cx.s.obj(src).attached_to.map(|t| cx.s.is_live(t) && cx.s.obj(t).zone == ZoneKind::Battlefield).unwrap_or(false);
                        if !ok {
                            cx.s.obj_mut(src).attached_to = None;
                            cx.s.derived_dirty = true;
                        }
                    }
                }
            }
            Flow::Next
        }
        Effect::ExtractSame { target } => {
            let me = env.controller;
            if *aux == 99 {
                return Flow::Next;
            }
            if *aux == 0 {
                let t = match cx.eval_o(env, ORef::Target(*target)) {
                    Some(t) => t,
                    None => return Flow::Next,
                };
                sc[0] = cx.s.obj(t).def.0;
                sc[1] = cx.s.obj(t).owner.idx() as u16;
                *aux = 1;
            }
            let (def, owner) = (CardDefId(sc[0]), Seat(sc[1] as u8));
            // Searching a hidden zone means looking at all of it (701.23a).
            cx.reveal_hand_to(me, owner);
            let p = &cx.s.players[owner.idx()];
            let cands: Vec<ObjRef> = p.graveyard.iter().chain(p.hand.iter()).chain(p.library.iter()).copied().filter(|&r| cx.s.obj(r).def == def).collect();
            if cands.is_empty() {
                cx.shuffle_library(owner);
                return Flow::Next;
            }
            for &r in &cands {
                cx.reveal_to(me, r);
            }
            let mut opts = card_options(cx, me, &cands);
            opts.push(Opt::Done);
            ask(cx, me, DecisionKind::ChooseCards { purpose: CardsPurpose::ExtractExile, remaining: cands.len() as u8 }, opts);
            Flow::Await
        }
        Effect::Pile { n } => wait(cx.fx_pile(env, *n, aux, sc)),
        Effect::LookPutTop { look } => wait(cx.fx_look_put_top(env, look, aux, sc)),
        Effect::Win(who) => {
            if let Some(p) = cx.eval_p(env, *who) {
                cx.s.result = Some(GameResult::Win(p));
            }
            Flow::Next
        }
        Effect::Scry(n) => wait(cx.fx_scry(env, n, false, aux, sc)),
        Effect::Surveil(n) => wait(cx.fx_scry(env, n, true, aux, sc)),
        Effect::PutBack(n) => wait(cx.fx_put_back(env, n, aux, sc)),
        Effect::Reorder(n) => wait(cx.fx_reorder(env, n, aux, sc)),
        Effect::LookTake { look, take } => wait(cx.fx_look_take(env, look, *take, aux, sc)),
        Effect::Search { who, filter, .. } => wait(cx.fx_search(env, *who, filter, aux)),
        Effect::Shuffle(who) => {
            if let Some(p) = cx.eval_p(env, *who) {
                cx.shuffle_library(p);
            }
            Flow::Next
        }
        Effect::CounterSpellExile(i) => {
            if let Some(Target::Obj(r)) = env.target(*i) {
                if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Stack && cx.can_be_countered(r) {
                    cx.emit(Event::SpellCountered { obj: r });
                    cx.move_zone(r, ZoneKind::Exile, MoveOpts::default());
                }
            }
            Flow::Next
        }
        Effect::MoveTo { objs: o, to, ctrl, tapped } => {
            let c = ctrl.and_then(|p| cx.eval_p(env, p));
            cx.s.moved.clear();
            let objs = objs_of(cx, env, o);
            cx.begin_batch(&objs);
            for r in objs {
                if cx.s.is_live(r) {
                    if let Some(nr) = cx.move_zone(r, *to, MoveOpts { controller: c, tapped: *tapped, ..Default::default() }) {
                        cx.s.moved.push(nr);
                    }
                }
            }
            cx.end_batch();
            Flow::Next
        }
        Effect::Sacrifice(o) => {
            let objs = objs_of(cx, env, o);
            cx.begin_batch(&objs);
            for r in objs {
                if cx.s.is_live(r) {
                    cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
                }
            }
            cx.end_batch();
            Flow::Next
        }
        Effect::CounterAbility(i) => {
            if let Some(Target::Obj(r)) = env.target(*i) {
                if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Stack && cx.s.obj(r).kind == ObjKind::Ability {
                    cx.s.stack.retain(|x| x.obj != r);
                    cx.commit_move(r, ZoneKind::Gone, MoveOpts { quiet: true, ..Default::default() }, None);
                }
            }
            Flow::Next
        }
        Effect::DiscardChoose { who, filter } => {
            let victim = match cx.eval_p(env, *who) {
                Some(p) => p,
                None => return Flow::Next,
            };
            if *aux != 0 {
                return Flow::Next;
            }
            let chooser = env.controller;
            let hand: Vec<ObjRef> = cx.s.players[victim.idx()].hand.clone();
            cx.reveal_hand_to(chooser, victim);
            let cands: Vec<ObjRef> = hand.into_iter().filter(|&h| {
                let c = cx.chars(h);
                cx.filter_match(filter, h, &c, false, chooser)
            }).collect();
            if cands.is_empty() {
                return Flow::Next;
            }
            let opts = card_options(cx, chooser, &cands);
            ask(cx, chooser, DecisionKind::ChooseCards { purpose: CardsPurpose::DiscardChosen, remaining: 1 }, opts);
            Flow::Await
        }
        Effect::ChooseName { lands } => match *aux {
            0 => {
                ask(cx, env.controller, DecisionKind::ChooseName { lands: *lands }, crate::stabilize::name_options(cx, *lands));
                Flow::Await
            }
            _ => Flow::Next,
        },
        Effect::DiscardNamed { who } => {
            let victim = match cx.eval_p(env, *who) {
                Some(p) => p,
                None => return Flow::Next,
            };
            let chooser = env.controller;
            let name = cx.s.name_choice;
            let hand: Vec<ObjRef> = cx.s.players[victim.idx()].hand.clone();
            for &h in &hand {
                cx.s.knowledge[h.slot as usize].known_to |= 1 << chooser.idx();
            }
            for h in hand {
                if name != 0 && cx.s.obj(h).def.0 + 1 == name {
                    cx.move_zone(h, ZoneKind::Graveyard, MoveOpts::default());
                }
            }
            cx.s.name_choice = 0;
            Flow::Next
        }
        Effect::Continuous { objs, layer, effect, until } => {
            let list = objs_of(cx, env, objs);
            let ts = cx.s.next_ts();
            cx.s.effects.push(ContInst { ts, layer: *layer, objs: list.into_iter().collect(), effect: effect.clone(), until: *until, controller: env.controller });
            cx.s.derived_dirty = true;
            Flow::Next
        }
        Effect::PlayerFx { who, fx, until } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let ts = cx.s.next_ts();
                cx.s.player_fx.push(PlayerFxInst { ts, player: p, fx: *fx, until: *until, controller: env.controller });
            }
            Flow::Next
        }
        Effect::AddMana { color, n } => {
            cx.add_mana(env.controller, *color, *n);
            Flow::Next
        }
        Effect::AddManaAny { n } => match *aux {
            0 => {
                ask(cx, env.controller, DecisionKind::ChooseColor, (0..5).map(Opt::Choice).collect());
                Flow::Await
            }
            _ => {
                let n = cx.eval_expr(env, n).max(0) as u8;
                if n > 0 {
                    cx.add_mana(env.controller, ManaColor::ALL[(sc[0] as usize).min(4)], n);
                }
                Flow::Next
            }
        },
        Effect::SacrificeChosen { who, filter } => {
            let victim = match cx.eval_p(env, *who) {
                Some(p) => p,
                None => return Flow::Next,
            };
            if *aux != 0 {
                return Flow::Next;
            }
            cx.refresh();
            let cands: Vec<ObjRef> = cx.s.battlefield.clone().into_iter().filter(|&r| {
                if cx.s.obj(r).controller != victim {
                    return false;
                }
                let c = cx.chars(r);
                cx.filter_match(filter, r, &c, false, victim)
            }).collect();
            let opts = card_options(cx, victim, &cands);
            match opts.len() {
                0 => Flow::Next,
                _ => {
                    ask(cx, victim, DecisionKind::ChooseCards { purpose: CardsPurpose::SacrificeEffect, remaining: 1 }, opts);
                    Flow::Await
                }
            }
        }
        Effect::PutChosen { who, from, filter, cmc, optional, .. } => {
            let p = match cx.eval_p(env, *who) {
                Some(p) => p,
                None => return Flow::Next,
            };
            if *aux != 0 {
                return Flow::Next;
            }
            let want = cmc.as_ref().map(|e| cx.eval_expr(env, e));
            let pool: Vec<ObjRef> = match from {
                ZoneKind::Graveyard => cx.s.players[p.idx()].graveyard.clone(),
                _ => cx.s.players[p.idx()].hand.clone(),
            };
            let cands: Vec<ObjRef> = pool.into_iter().filter(|&r| {
                let c = cx.chars(r);
                cx.filter_match(filter, r, &c, false, p) && want.map(|w| cx.db.def(cx.s.obj(r).def).mana_value() as i32 == w).unwrap_or(true)
            }).collect();
            if cands.is_empty() {
                return Flow::Next;
            }
            let mut opts = card_options(cx, p, &cands);
            if *optional {
                opts.push(Opt::Done);
            }
            ask(cx, p, DecisionKind::ChooseCards { purpose: CardsPurpose::PutOnto, remaining: 1 }, opts);
            Flow::Await
        }
        Effect::ExileGraveyard { who } => {
            if let Some(p) = cx.eval_p(env, *who) {
                for r in cx.s.players[p.idx()].graveyard.clone() {
                    cx.move_zone(r, ZoneKind::Exile, MoveOpts::default());
                }
            }
            Flow::Next
        }
        Effect::Amass { token, n } => {
            let me = env.controller;
            let n = cx.eval_expr(env, n).max(0) as u16;
            match *aux {
                0 => {
                    cx.refresh();
                    let armies = |cx: &Cx| -> Vec<ObjRef> {
                        cx.s.battlefield.iter().copied().filter(|&r| cx.s.obj(r).controller == me && cx.s.derived[r.slot as usize].subtypes.has(SUB_ARMY)).collect()
                    };
                    let mut list = armies(cx);
                    if list.is_empty() {
                        let def = cx.db.id(token).unwrap_or_else(|| panic!("unknown token {token:?}"));
                        cx.s.moved.clear();
                        if let Some(t) = cx.create_token(def, me) {
                            cx.s.moved.push(t);
                        }
                        cx.refresh();
                        list = armies(cx);
                    }
                    let opts = card_options(cx, me, &list);
                    match opts.len() {
                        0 => Flow::Next,
                        1 => {
                            if let Opt::Card(r) = opts[0] {
                                cx.add_counters(r, CounterKind::PlusOne, n as i32);
                            }
                            Flow::Next
                        }
                        _ => {
                            sc[0] = n;
                            ask(cx, me, DecisionKind::ChooseCards { purpose: CardsPurpose::AmassOnto, remaining: 1 }, opts);
                            Flow::Await
                        }
                    }
                }
                _ => Flow::Next,
            }
        }
        Effect::TakeMoved { filter, optional, .. } => {
            if *aux != 0 {
                return Flow::Next;
            }
            let me = env.controller;
            let cands: Vec<ObjRef> = cx.s.moved.clone().into_iter().filter(|&r| {
                if !cx.s.is_live(r) {
                    return false;
                }
                let c = cx.chars(r);
                cx.filter_match(filter, r, &c, false, me)
            }).collect();
            if cands.is_empty() {
                return Flow::Next;
            }
            let mut opts = card_options(cx, me, &cands);
            if *optional {
                opts.push(Opt::Done);
            }
            ask(cx, me, DecisionKind::ChooseCards { purpose: CardsPurpose::PutOnto, remaining: 1 }, opts);
            Flow::Await
        }
        Effect::EachPutFromHand { filter } => {
            let active = cx.s.turn.active;
            let order = [active, active.other()];
            loop {
                let stage = *aux as usize;
                if stage >= 2 {
                    break;
                }
                let seat = order[stage];
                let cands: Vec<ObjRef> = cx.s.players[seat.idx()].hand.clone().into_iter().filter(|&r| {
                    let c = cx.chars(r);
                    cx.filter_match(filter, r, &c, false, seat)
                }).collect();
                if cands.is_empty() {
                    sc[stage] = 0;
                    *aux += 1;
                    continue;
                }
                let mut opts = card_options(cx, seat, &cands);
                opts.push(Opt::Done);
                ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::PutEach, remaining: 1 }, opts);
                return Flow::Await;
            }
            // Both seats have chosen: everything enters together.
            cx.s.moved.clear();
            cx.begin_batch(&[]);
            for stage in 0..2 {
                if sc[stage] != 0 {
                    let r = cx.s.cur_ref(sc[stage] - 1);
                    if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Hand {
                        if let Some(nr) = cx.move_zone(r, ZoneKind::Battlefield, MoveOpts::default()) {
                            cx.s.moved.push(nr);
                        }
                    }
                }
            }
            cx.end_batch();
            Flow::Next
        }
        Effect::CopyToken(o) => {
            cx.s.moved.clear();
            for r in objs_of(cx, env, o) {
                if cx.s.is_live(r) && cx.s.obj(r).kind == ObjKind::Token {
                    let def = cx.s.obj(r).def;
                    if let Some(t) = cx.create_token(def, env.controller) {
                        cx.s.moved.push(t);
                    }
                }
            }
            Flow::Next
        }
        Effect::GainEnergy { who, n } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let n = cx.eval_expr(env, n).max(0) as u32;
                cx.s.players[p.idx()].energy += n;
            }
            Flow::Next
        }
        Effect::LoseEnergy { who, n } => {
            if let Some(p) = cx.eval_p(env, *who) {
                let n = cx.eval_expr(env, n).max(0) as u32;
                let e = &mut cx.s.players[p.idx()].energy;
                *e = e.saturating_sub(n);
            }
            Flow::Next
        }
        Effect::Reflexive { ability } => {
            if let Some(source) = env.source {
                let def = cx.s.stack.last().map(|e| e.def).unwrap_or(cx.s.objs[source.slot as usize].def);
                cx.s.pending_triggers.push(PendingTrigger { source, def, ability: *ability, controller: env.controller, cap: Captured::default() });
            }
            Flow::Next
        }
        Effect::DelayMoved { when, ability } => {
            let mut cap = Captured::default();
            cap.objs.extend(cx.s.moved.iter().copied());
            if !cap.objs.is_empty() {
                cx.add_delayed(env, *when, *ability, cap);
            }
            Flow::Next
        }
        Effect::ExileLinked(o) => {
            cx.s.moved.clear();
            if let Some(src) = env.source {
                for r in objs_of(cx, env, o) {
                    if cx.s.is_live(r) {
                        let (owner, cmc) = (cx.s.obj(r).owner, cx.db.def(cx.s.obj(r).def).mana_value() as u8);
                        if let Some(nr) = cx.move_zone(r, ZoneKind::Exile, MoveOpts::default()) {
                            cx.s.moved.push(nr);
                            cx.s.links.push(Link { src, owner, cmc });
                        }
                    }
                }
            }
            Flow::Next
        }
        Effect::LinkedToken { token } => {
            if let Some(src) = env.source {
                // The leaves trigger's source is the object in its new zone: one generation on.
                let is_mine = |l: &Link| l.src.slot == src.slot && (l.src.gen == src.gen || l.src.gen.wrapping_add(1) == src.gen);
                let mine: Vec<Link> = cx.s.links.iter().filter(|l| is_mine(l)).cloned().collect();
                cx.s.links.retain(|l| !is_mine(l));
                let def = cx.db.id(token).unwrap_or_else(|| panic!("unknown token {token:?}"));
                cx.s.moved.clear();
                for l in mine {
                    if let Some(t) = cx.create_token(def, l.owner) {
                        cx.s.moved.push(t);
                        let ts = cx.s.next_ts();
                        cx.s.effects.push(ContInst { ts, layer: Layer::L7b, objs: [t].into_iter().collect(), effect: ContEffect::SetPT(l.cmc as i16, l.cmc as i16), until: Until::Forever, controller: l.owner });
                        cx.s.derived_dirty = true;
                    }
                }
            }
            Flow::Next
        }
        Effect::SetPTX { objs, x, until } => {
            let n = cx.eval_expr(env, x).max(0) as i16;
            let list = objs_of(cx, env, objs);
            let ts = cx.s.next_ts();
            cx.s.effects.push(ContInst { ts, layer: Layer::L7b, objs: list.into_iter().collect(), effect: ContEffect::SetPT(n, n), until: *until, controller: env.controller });
            cx.s.derived_dirty = true;
            Flow::Next
        }
        Effect::MarkUsed(i) => {
            if let Some(src) = env.source {
                cx.s.used.push((src, *i));
            }
            Flow::Next
        }
        Effect::TapAttackMoved => {
            // Each creature put onto the battlefield attacking: its controller chooses which player
            // or planeswalker it attacks (508.4); the choice is skipped when there is only one.
            let opp = env.controller.other();
            let moved: Vec<ObjRef> = cx.s.moved.iter().copied().collect();
            while (*aux as usize) < moved.len() {
                let r = moved[*aux as usize];
                if !(cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Battlefield) {
                    *aux += 1;
                    continue;
                }
                let opts = crate::combat::attack_target_opts(cx, opp);
                if opts.len() > 1 {
                    ask(cx, env.controller, DecisionKind::DeclareAttacker { creature: r }, opts);
                    return Flow::Await;
                }
                put_attacking(cx, r, AttackTarget::Player(opp));
                *aux += 1;
            }
            Flow::Next
        }
        Effect::AllowCastMoved => {
            let moved: Vec<ObjRef> = cx.s.moved.iter().copied().collect();
            for r in moved {
                if cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Exile && cx.s.obj(r).kind == ObjKind::Card {
                    let owner = cx.s.obj(r).owner;
                    cx.s.exile_plays.push((r, owner));
                }
            }
            Flow::Next
        }
        Effect::ExilePlay { n } => {
            let p = env.controller;
            let n = cx.eval_expr(env, n).max(0);
            let live: Vec<(ObjRef, Seat)> = cx.s.exile_plays.iter().copied().filter(|&(r, _)| cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Exile).collect();
            cx.s.exile_plays = live;
            for _ in 0..n {
                let Some(&top) = cx.s.players[p.idx()].library.last() else { break };
                if let Some(nr) = cx.move_zone(top, ZoneKind::Exile, MoveOpts::default()) {
                    cx.s.exile_plays.push((nr, p));
                }
            }
            Flow::Next
        }
        Effect::DrawRevealCast { n } => {
            let p = env.controller;
            if *aux == 0 {
                let n = cx.eval_expr(env, n).max(0);
                cx.s.moved.clear();
                for _ in 0..n {
                    let top = cx.s.players[p.idx()].library.last().copied();
                    if cx.draw(p) {
                        if let Some(r) = top {
                            // The card is a new object in hand now; find it by its slot.
                            let nr = cx.s.cur_ref(r.slot);
                            cx.s.knowledge[r.slot as usize].known_to = 3;
                            cx.s.moved.push(nr);
                        }
                    }
                }
                let cands: Vec<ObjRef> = cx.s.moved.clone().into_iter().filter(|&r| cx.s.is_live(r) && cx.s.obj(r).zone == ZoneKind::Hand).collect();
                let mut ok = Vec::new();
                for r in cands {
                    let def = cx.db.def(cx.s.obj(r).def);
                    if !def.types.contains(Types::LAND) && !cx.cast_restricted(p, r) && cx.way_feasible_x(p, r, crate::restrict::FREE_WAY, false) {
                        ok.push(r);
                    }
                }
                if ok.is_empty() {
                    return Flow::Next;
                }
                let mut opts = card_options(cx, p, &ok);
                opts.push(Opt::Done);
                ask(cx, p, DecisionKind::ChooseCards { purpose: CardsPurpose::CastFree, remaining: 1 }, opts);
                return Flow::Await;
            }
            Flow::Next
        }
        Effect::Venture => {
            if *aux != 0 {
                return Flow::Next;
            }
            let p = env.controller;
            if let Some(d) = cx.s.players[p.idx()].dungeon {
                let def = cx.s.obj(d).def;
                let exits = cx.db.def(def).dungeon.as_ref().map(|d| d.rooms[cx.s.players[p.idx()].room as usize].exits.clone()).unwrap_or_default();
                match exits.len() {
                    0 => {
                        // Venturing from the bottommost room (CR 701.49c): that dungeon is completed first.
                        cx.complete_dungeon(p);
                    }
                    1 => {
                        cx.enter_room(p, exits[0]);
                        return Flow::Next;
                    }
                    n => {
                        ask(cx, p, DecisionKind::ChooseRoom, (0..n as u8).map(Opt::Choice).collect());
                        return Flow::Await;
                    }
                }
            }
            let n = cx.db.dungeons.len() as u8;
            ask(cx, p, DecisionKind::ChooseDungeon, (0..n).map(Opt::Choice).collect());
            Flow::Await
        }
        Effect::SacrificeOneEach { who, filters } => {
            let victim = match cx.eval_p(env, *who) {
                Some(p) => p,
                None => return Flow::Next,
            };
            loop {
                if *aux == 1 {
                    sc[0] += 1;
                    *aux = 0;
                }
                let k = sc[0] as usize;
                if k >= filters.len() {
                    sc[0] = 0;
                    return Flow::Next;
                }
                cx.refresh();
                let pool: Vec<ObjRef> = cx.s.battlefield.clone().into_iter().filter(|&r| cx.s.obj(r).controller == victim).collect();
                let fits: Vec<Vec<bool>> = filters
                    .iter()
                    .map(|flt| {
                        pool.iter()
                            .map(|&r| {
                                let c = cx.chars(r);
                                cx.filter_match(flt, r, &c, false, victim)
                            })
                            .collect()
                    })
                    .collect();
                let used = vec![false; pool.len()];
                let best = max_match(&fits, k, &used);
                let mut cands = Vec::new();
                for (i, &r) in pool.iter().enumerate() {
                    if fits[k][i] {
                        let mut u = used.clone();
                        u[i] = true;
                        if 1 + max_match(&fits, k + 1, &u) == best {
                            cands.push(r);
                        }
                    }
                }
                let opts = card_options(cx, victim, &cands);
                match opts.len() {
                    0 => sc[0] += 1,
                    _ => {
                        ask(cx, victim, DecisionKind::ChooseCards { purpose: CardsPurpose::SacrificeEffect, remaining: 1 }, opts);
                        return Flow::Await;
                    }
                }
            }
        }
        Effect::MayPay { .. } | Effect::MayElse { .. } | Effect::ForEachPlayer { .. } => unreachable!("control flow is compiled away"),
        Effect::Seq(_) | Effect::If { .. } | Effect::May { .. } | Effect::ForEach { .. } | Effect::Repeat { .. } => unreachable!("control flow is compiled away"),
    }
}

/// The most filters (from index `k` on) that can be given distinct permanents of the pool.
fn max_match(fits: &[Vec<bool>], k: usize, used: &[bool]) -> usize {
    if k >= fits.len() {
        return 0;
    }
    let mut best = max_match(fits, k + 1, used);
    for i in 0..used.len() {
        if fits[k][i] && !used[i] {
            let mut u = used.to_vec();
            u[i] = true;
            best = best.max(1 + max_match(fits, k + 1, &u));
        }
    }
    best
}

/// Card choices for `seat` from `cards`, collapsed by definition and canonically ordered.
pub fn card_options(cx: &Cx, seat: Seat, cards: &[ObjRef]) -> Vec<Opt> {
    let mut v: Vec<ObjRef> = cards.to_vec();
    v.sort_by_key(|&r| cx.obj_key(seat, r));
    let mut out: Vec<ObjRef> = Vec::new();
    // Interchangeable cards (same definition and same observable state) are offered once.
    let same = |a: ObjRef, b: ObjRef| {
        let (x, y) = (cx.s.obj(a), cx.s.obj(b));
        // Only permanents differ by observable state; cards in hidden zones differ by nothing.
        if x.zone != ZoneKind::Battlefield || y.zone != ZoneKind::Battlefield {
            return x.def == y.def && x.zone == y.zone;
        }
        // Same rules-relevant state, including derived characteristics and anything that refers to
        // the object by identity (see `obj_sig`).
        cx.obj_sig(a) == cx.obj_sig(b) && (x.entered_turn == cx.s.turn.number) == (y.entered_turn == cx.s.turn.number)
    };
    for r in v {
        if !out.iter().any(|&o| same(o, r)) {
            out.push(r);
        }
    }
    out.into_iter().map(Opt::Card).collect()
}

/// Taps `r` and adds it to the attackers, attacking `target` (it was never declared as an attacker).
fn put_attacking(cx: &mut Cx, r: ObjRef, target: AttackTarget) {
    cx.tap(r);
    if let Some(cb) = cx.s.combat.as_mut() {
        cb.attackers.push(AttackerInfo { obj: r, target, blockers: smallvec::SmallVec::new(), blocked: false });
    }
}

pub fn feed(cx: &mut Cx, f: &mut ResolveFrame, p: &Pending, opt: Opt) -> Next {
    // Library effects: `Search` needs its destination from the instruction being run.
    let (dest, put_to) = {
        let code = entry_code(cx.db, &f.entry);
        match code.get(f.pc as usize) {
            Some(Instr::Leaf(Effect::Search { dest, tapped, .. })) => (Some((*dest, *tapped)), None),
            Some(Instr::Leaf(Effect::PutChosen { to, tapped, .. })) => (None, Some((*to, *tapped))),
            Some(Instr::Leaf(Effect::TakeMoved { to, .. })) => (None, Some((*to, false))),
            _ => (None, None),
        }
    };
    if let Some(n) = crate::libfx::feed(cx, f, p, opt, dest) {
        return n;
    }
    match (p.kind, opt) {
        (DecisionKind::DeclareAttacker { creature }, Opt::Attack(t)) => {
            put_attacking(cx, creature, t);
            f.aux += 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::ExtractExile, .. }, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Exile, MoveOpts::default());
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::ExtractExile, .. }, _) => {
            cx.shuffle_library(Seat(f.sc[1] as u8));
            f.aux = 99;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::DiscardEffect, .. }, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
            f.aux += 1; // one more discarded; the op re-evaluates how many are left
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::SacrificeEffect, .. }, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::PutEach, .. }, o) => {
            let stage = f.aux as usize;
            f.sc[stage] = if let Opt::Card(r) = o { r.slot + 1 } else { 0 };
            f.aux += 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::KeepOne, .. }, Opt::Card(r)) => {
            cx.s.moved.push(r);
            f.aux += 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::GambitChoose, .. }, Opt::Card(r)) => {
            let stage = f.aux as usize;
            f.sc[stage] = r.slot + 1;
            f.aux += 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::RingBearer, .. }, Opt::Card(r)) => {
            cx.s.players[p.seat.idx()].ring_bearer = Some(r);
            cx.s.derived_dirty = true;
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::AmassOnto, .. }, Opt::Card(r)) => {
            let n = f.sc[0] as i32;
            cx.add_counters(r, CounterKind::PlusOne, n);
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::PutOnto, .. }, o) => {
            if let (Opt::Card(r), Some((to, tapped))) = (o, put_to) {
                cx.move_zone(r, to, MoveOpts { tapped, controller: if to == ZoneKind::Battlefield { Some(p.seat) } else { None }, ..Default::default() });
            }
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::ExileUntilLeaves, .. }, o) => {
            f.aux = 1;
            if let (Opt::Card(r), StackKind::Ability { source, .. }) = (o, f.entry.kind) {
                // The source must still be on the battlefield (610.3b).
                if cx.s.is_live(source) && cx.s.obj(source).zone == ZoneKind::Battlefield && cx.s.is_live(r) {
                    let from = cx.s.obj(r).zone;
                    if let Some(nr) = cx.move_zone(r, ZoneKind::Exile, MoveOpts::default()) {
                        cx.s.until_links.push(UntilLink { src: source, card: nr, from });
                    }
                }
            }
            Next::Stay
        }
        (DecisionKind::PayEnergyAmount, Opt::Number(n)) => {
            cx.s.players[p.seat.idx()].energy -= n;
            f.sc[0] = n as u16;
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::CastFromGraveyard, .. }, o) => {
            f.aux = 1;
            match o {
                Opt::Card(r) => Next::Push(Frame::Cast(CastFrame::spell(r, p.seat, crate::restrict::GY_WAY, ZoneKind::Graveyard))),
                _ => Next::Stay,
            }
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::CastMiracle, .. }, o) => {
            f.aux = 1;
            match o {
                Opt::Card(r) => {
                    let way = match cx.db.def(f.entry.def).abilities.get(match f.entry.kind { StackKind::Ability { ability, .. } => ability as usize, _ => 0 }) {
                        Some(AbilityDef::Triggered(TriggeredDef { effect: Some(Effect::MiracleCast(w)), .. })) => *w,
                        _ => 1,
                    };
                    Next::Push(Frame::Cast(CastFrame::spell(r, p.seat, way, ZoneKind::Hand)))
                }
                _ => Next::Stay,
            }
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::CastFromExile, .. }, o) => {
            f.aux = 1;
            match o {
                Opt::Card(r) => Next::Push(Frame::Cast(CastFrame::spell(r, p.seat, crate::restrict::ENERGY_WAY, ZoneKind::Exile))),
                _ => Next::Stay,
            }
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::CastFree, .. }, o) => {
            f.aux = 1;
            match o {
                Opt::Card(r) => Next::Push(Frame::Cast(CastFrame::spell(r, p.seat, crate::restrict::FREE_WAY, ZoneKind::Hand))),
                _ => Next::Stay,
            }
        }
        (DecisionKind::ChooseDungeon, Opt::Choice(i)) => {
            let p = p.seat;
            let def = cx.db.dungeons[i as usize];
            cx.start_dungeon(p, def);
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseRoom, Opt::Choice(i)) => {
            let p = p.seat;
            let d = cx.s.players[p.idx()].dungeon.expect("a dungeon is in play");
            let to = cx.db.def(cx.s.obj(d).def).dungeon.as_ref().expect("dungeon def").rooms[cx.s.players[p.idx()].room as usize].exits[i as usize];
            cx.enter_room(p, to);
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChangeTargets, o @ (Opt::Yes | Opt::No)) => {
            f.aux = if o == Opt::Yes { 3 } else { 4 };
            Next::Stay
        }
        (DecisionKind::ChooseTarget { slot }, o @ (Opt::Target(_) | Opt::Done)) => {
            // New targets for a copy of a spell (the newest stack entry).
            let t = if let Opt::Target(t) = o { t } else { Target::None };
            if let Some(e) = cx.s.stack.last_mut() {
                if (slot as usize) < e.targets.len() {
                    e.targets[slot as usize] = t;
                }
            }
            f.sc[1] += 1;
            Next::Stay
        }
        (DecisionKind::ChooseName { .. }, Opt::Name(d)) => {
            cx.s.name_choice = d + 1;
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseColor, Opt::Choice(i)) => {
            f.sc[0] = i as u16;
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::DiscardChosen, .. }, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::PayUnless, o @ (Opt::Yes | Opt::No)) => {
            f.aux = if o == Opt::Yes { 1 } else { 2 };
            Next::Stay
        }
        (DecisionKind::May, Opt::Yes) => {
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::May, Opt::No) => {
            f.aux = 2;
            Next::Stay
        }
        _ => unreachable!("unexpected decision in resolve frame: {:?}", p.kind),
    }
}

/// Where a countered spell goes: an alternative way that exiles (flashback) overrides `dest`.
/// Does a spell cast this way go to exile instead of the graveyard (flashback, Bilbo)?
fn exile_after_cast(def: &CardDef, way: u8) -> bool {
    if way == crate::restrict::GY_WAY {
        return def.types.intersects(Types::SPELL_KIND);
    }
    crate::cost::alt_of(def, way).map(|a| a.exile_after).unwrap_or(false)
}

/// "Counter it unless its controller pays": ask the payer (when they can), pay, or counter.
fn counter_unless(cx: &mut Cx, victim: ObjRef, pays: &crate::mana::ManaCost, aux: u16) -> Flow {
    let payer = cx.s.stack.iter().find(|e| e.obj == victim).map(|e| e.controller).unwrap_or(cx.s.obj(victim).controller);
    match aux {
        0 => {
            if cx.plan_cost(payer, pays, 0, None, None).is_some() {
                ask(cx, payer, DecisionKind::PayUnless, vec![Opt::Yes, Opt::No]);
                Flow::Await
            } else {
                counter_stack_obj(cx, victim);
                Flow::Next
            }
        }
        1 => {
            let (pay, srcs) = cx.plan_cost(payer, pays, 0, None, None).expect("payment was offered, so it is possible");
            cx.apply_payment(payer, &pay, &srcs, &[]);
            Flow::Next
        }
        _ => {
            counter_stack_obj(cx, victim);
            Flow::Next
        }
    }
}

/// Counters a spell (it goes to its usual place) or an ability (it ceases to exist).
fn counter_stack_obj(cx: &mut Cx, victim: ObjRef) {
    if !cx.s.is_live(victim) || cx.s.obj(victim).zone != ZoneKind::Stack {
        return;
    }
    if matches!(cx.s.stack.iter().find(|e| e.obj == victim).map(|e| e.kind), Some(StackKind::Ability { .. })) {
        cx.s.stack.retain(|x| x.obj != victim);
        cx.commit_move(victim, ZoneKind::Gone, MoveOpts { quiet: true, ..Default::default() }, None);
    } else {
        cx.emit(Event::SpellCountered { obj: victim });
        counter_dest(cx, victim, ZoneKind::Graveyard);
    }
}

fn counter_dest(cx: &mut Cx, r: ObjRef, dest: ZoneKind) {
    let exile = cx.s.stack.iter().find(|e| e.obj == r).map(|e| exile_after_cast(cx.db.def(e.def), e.cast.way)).unwrap_or(false);
    cx.move_zone(r, if exile { ZoneKind::Exile } else { dest }, MoveOpts::default());
}

fn finish(cx: &mut Cx, f: &mut ResolveFrame) {
    let e = &f.entry;
    if !cx.s.is_live(e.obj) {
        return;
    }
    match e.kind {
        StackKind::Ability { .. } => {
            // Abilities cease to exist when they leave the stack.
            cx.s.stack.retain(|x| x.obj != e.obj);
            cx.commit_move(e.obj, ZoneKind::Gone, MoveOpts { quiet: true, ..Default::default() }, None);
        }
        StackKind::Spell => {
            let permanent = cx.db.def(e.def).types.intersects(Types::PERMANENT);
            let ctrl = e.controller;
            if permanent && !f.fizzled && !f.countered {
                let nr = cx.move_zone(e.obj, ZoneKind::Battlefield, MoveOpts { controller: Some(ctrl), cast: true, delved: e.cast.delve_is, cast_from: Some((e.cast.from, e.cast.way)), ..Default::default() });
                // An Aura spell enters attached to what it targeted (303.4f).
                if let (Some(nr), true) = (nr, cx.db.def(e.def).subtypes.has(SUB_AURA)) {
                    if let Some(Target::Obj(t)) = e.targets.first().copied() {
                        if cx.s.is_live(t) && cx.s.is_live(nr) {
                            cx.s.obj_mut(nr).attached_to = Some(t);
                            cx.s.derived_dirty = true;
                        }
                    }
                }
            } else if e.cast.way == crate::restrict::ADVENTURE_WAY && !f.fizzled && !f.countered {
                // An Adventure that resolved: exiled "on an adventure"; its owner may cast the creature from exile.
                if let Some(nr) = cx.move_zone(e.obj, ZoneKind::Exile, MoveOpts::default()) {
                    if cx.s.is_live(nr) {
                        let owner = cx.s.obj(nr).owner;
                        cx.s.exile_plays.push((nr, owner));
                    }
                }
            } else {
                let exile = exile_after_cast(cx.db.def(e.def), e.cast.way);
                cx.move_zone(e.obj, if exile { ZoneKind::Exile } else { ZoneKind::Graveyard }, MoveOpts::default());
            }
        }
    }
}

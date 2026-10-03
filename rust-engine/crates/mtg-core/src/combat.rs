//! Combat (doc 01 section 11): per-creature attack and block decisions with canonicalization,
//! first/double strike steps, trample, deathtouch, lifelink, and damage assignment per CR 510.1c
//! (no blocker ordering). Damage is collected into a batch and dealt together; SBA runs after.

use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::event::Event;
use crate::frame::*;
use crate::ids::*;
use crate::ir::{ContEffect, Layer, PlayerFx, Until};
use crate::state::*;
use crate::types::*;
use smallvec::SmallVec;

impl<'a> Cx<'a> {
    pub fn attack_candidates(&mut self, seat: Seat) -> SmallVec<[ObjRef; 8]> {
        self.refresh();
        let mut v: SmallVec<[ObjRef; 8]> = SmallVec::new();
        for &r in self.s.battlefield.iter() {
            let o = &self.s.objs[r.slot as usize];
            let c = self.s.derived[r.slot as usize];
            if o.controller == seat
                && !o.tapped
                && !o.phased
                && c.types.contains(Types::CREATURE)
                && !c.keywords.contains(Keywords::DEFENDER)
                && !c.cant_attack
                && (c.keywords.contains(Keywords::HASTE) || o.entered_turn < self.s.turn.own_turn[seat.idx()])
            {
                v.push(r);
            }
        }
        let keyed: Vec<(u32, u64, u32, ObjRef)> = v.iter().map(|&r| (self.s.objs[r.slot as usize].def.0 as u32, self.obj_sig(r), self.s.vids[r.slot as usize][seat.idx()].0, r)).collect();
        let mut keyed = keyed;
        keyed.sort_by_key(|k| (k.0, k.1, k.2));
        keyed.into_iter().map(|k| k.3).collect()
    }

    fn can_block_pair(&self, attacker: ObjRef, blocker: ObjRef) -> bool {
        let a = self.chars(attacker);
        let b = self.chars(blocker);
        if a.keywords.contains(Keywords::FLYING) && !b.keywords.intersects(Keywords::FLYING | Keywords::REACH) {
            return false;
        }
        if b.keywords.contains(Keywords::BLOCKS_ONLY_FLYERS) && !a.keywords.contains(Keywords::FLYING) {
            return false;
        }
        // The Ring: the Ring-bearer can't be blocked by creatures with greater power.
        if self.s.players[self.s.obj(attacker).controller.idx()].ring_bearer == Some(attacker) && self.s.players[self.s.obj(attacker).controller.idx()].ring > 0 && b.power > a.power {
            return false;
        }
        // Swampwalk: can't be blocked while the defending player controls a Swamp (702.14c).
        if a.keywords.contains(Keywords::SWAMPWALK) {
            let def = self.s.obj(blocker).controller;
            let swamp = self.s.battlefield.iter().any(|&r| {
                let c = self.s.derived[r.slot as usize];
                self.s.objs[r.slot as usize].controller == def && c.types.contains(Types::LAND) && c.subtypes.has(SUB_SWAMP)
            });
            if swamp {
                return false;
            }
        }
        true
    }

    pub fn block_candidates(&mut self, seat: Seat) -> SmallVec<[ObjRef; 8]> {
        self.refresh();
        let attackers: SmallVec<[ObjRef; 8]> = match &self.s.combat {
            Some(c) => c.attackers.iter().map(|a| a.obj).filter(|&a| self.s.is_live(a)).collect(),
            None => SmallVec::new(),
        };
        let mut v: SmallVec<[ObjRef; 8]> = SmallVec::new();
        for &r in self.s.battlefield.iter() {
            let o = &self.s.objs[r.slot as usize];
            let c = self.s.derived[r.slot as usize];
            if o.controller == seat && !o.tapped && !o.phased && c.types.contains(Types::CREATURE) && attackers.iter().any(|&a| self.can_block_pair(a, r)) {
                v.push(r);
            }
        }
        v.sort_by_key(|&r| self.obj_key(seat, r));
        v
    }

    /// Menace feasibility: can every attacker that currently has exactly one blocker still get a
    /// second one from `remaining`, with each remaining creature used at most once?
    fn menace_feasible(&self, remaining: &[ObjRef], extra: Option<(usize, ObjRef)>) -> bool {
        let combat = match &self.s.combat {
            Some(c) => c,
            None => return true,
        };
        let mut deficient: SmallVec<[usize; 4]> = SmallVec::new();
        for (i, a) in combat.attackers.iter().enumerate() {
            let mut n = a.blockers.len();
            if let Some((ei, _)) = extra {
                if ei == i {
                    n += 1;
                }
            }
            if n == 1 && self.chars(a.obj).keywords.contains(Keywords::MENACE) {
                deficient.push(i);
            }
        }
        if deficient.is_empty() {
            return true;
        }
        let mut owner: SmallVec<[i16; 8]> = SmallVec::from_elem(-1, remaining.len());
        fn assign(d: usize, deficient: &[usize], remaining: &[ObjRef], owner: &mut [i16], seen: &mut [bool], ok: &dyn Fn(usize, ObjRef) -> bool) -> bool {
            for (ri, &r) in remaining.iter().enumerate() {
                if seen[ri] || !ok(deficient[d], r) {
                    continue;
                }
                seen[ri] = true;
                if owner[ri] < 0 || assign(owner[ri] as usize, deficient, remaining, owner, seen, ok) {
                    owner[ri] = d as i16;
                    return true;
                }
            }
            false
        }
        let ok = |ai: usize, r: ObjRef| self.can_block_pair(combat.attackers[ai].obj, r);
        for d in 0..deficient.len() {
            let mut seen: SmallVec<[bool; 8]> = SmallVec::from_elem(false, remaining.len());
            if !assign(d, &deficient, remaining, &mut owner, &mut seen, &ok) {
                return false;
            }
        }
        true
    }

    fn deals_damage_in(&self, r: ObjRef, first_step: bool) -> bool {
        let k = self.chars(r).keywords;
        if first_step {
            k.intersects(Keywords::FIRST_STRIKE | Keywords::DOUBLE_STRIKE)
        } else {
            let first_struck = self.s.combat.as_ref().map(|c| c.first_struck.contains(&r)).unwrap_or(false);
            !first_struck || k.contains(Keywords::DOUBLE_STRIKE)
        }
    }

    /// Does any creature in combat have first or double strike? (Decides whether the first-strike
    /// damage step happens at all.)
    pub fn combat_has_first_strike(&mut self) -> bool {
        self.refresh();
        let combat = match &self.s.combat {
            Some(c) => c,
            None => return false,
        };
        for a in &combat.attackers {
            if self.s.is_live(a.obj) && self.chars(a.obj).keywords.intersects(Keywords::FIRST_STRIKE | Keywords::DOUBLE_STRIKE) {
                return true;
            }
            for &b in &a.blockers {
                if self.s.is_live(b) && self.chars(b).keywords.intersects(Keywords::FIRST_STRIKE | Keywords::DOUBLE_STRIKE) {
                    return true;
                }
            }
        }
        false
    }
}

/// What a creature attacking on behalf of the other seat can attack: the player, then each of
/// their planeswalkers.
pub(crate) fn attack_target_opts(cx: &mut Cx, defender: Seat) -> Vec<Opt> {
    let mut opts = vec![Opt::Attack(AttackTarget::Player(defender))];
    let walkers: Vec<ObjRef> = cx.s.battlefield.iter().copied().filter(|&w| cx.s.obj(w).controller == defender && cx.chars(w).types.contains(Types::PLANESWALKER)).collect();
    for w in walkers {
        opts.push(Opt::Attack(AttackTarget::Walker(w)));
    }
    opts
}

pub fn attackers_frame(cx: &mut Cx) -> Frame {
    let seat = cx.s.turn.active;
    let cands = cx.attack_candidates(seat);
    Frame::Combat(CombatFrame { stage: CombatStage::Attackers { cands, i: 0, declined_sig: None }, batch: SmallVec::new() })
}

pub fn blockers_frame(cx: &mut Cx) -> Frame {
    let def = cx.s.turn.active.other();
    let cands = cx.block_candidates(def);
    Frame::Combat(CombatFrame { stage: CombatStage::Blockers { cands, i: 0 }, batch: SmallVec::new() })
}

pub fn damage_frame(first_strike: bool) -> Frame {
    Frame::Combat(CombatFrame {
        stage: CombatStage::Damage { first_strike, attacker_idx: 0, started: false, blocker_idx: 0, remaining: 0, plan: SmallVec::new() },
        batch: SmallVec::new(),
    })
}

pub fn run(cx: &mut Cx, f: &mut CombatFrame) -> Next {
    cx.refresh();
    match &mut f.stage {
        CombatStage::Attackers { cands, i, declined_sig } => {
            let seat = cx.s.turn.active;
            while (*i as usize) < cands.len() {
                let c = cands[*i as usize];
                if !cx.s.is_live(c) || cx.s.obj(c).tapped {
                    *i += 1;
                    continue;
                }
                cx.refresh();
                let sig = cx.obj_sig(c);
                let mut opts = Vec::with_capacity(2);
                if *declined_sig != Some(sig) {
                    opts = attack_target_opts(cx, seat.other());
                }
                // "Attacks each combat if able" (508.1d): no way to decline.
                if !cx.chars(c).keywords.contains(Keywords::MUST_ATTACK) {
                    opts.push(Opt::NoAttack);
                }
                ask(cx, seat, DecisionKind::DeclareAttacker { creature: c }, opts);
                return Next::Await;
            }
            // Declaration complete: tap attackers without vigilance.
            let atk: SmallVec<[ObjRef; 8]> = cx.s.combat.as_ref().map(|c| c.attackers.iter().map(|a| a.obj).collect()).unwrap_or_default();
            for &a in atk.iter() {
                if !cx.chars(a).keywords.contains(Keywords::VIGILANCE) {
                    cx.tap(a);
                }
            }
            for &a in atk.iter() {
                cx.emit(Event::Attacks { obj: a });
            }
            cx.emit(Event::AttackersDeclared { count: atk.len().min(255) as u8 });
            Next::Done
        }
        CombatStage::Blockers { cands, i } => {
            let seat = cx.s.turn.active.other();
            while (*i as usize) < cands.len() {
                let c = cands[*i as usize];
                if !cx.s.is_live(c) || cx.s.obj(c).tapped {
                    *i += 1;
                    continue;
                }
                let rem: SmallVec<[ObjRef; 8]> = cands[(*i as usize + 1)..].iter().copied().filter(|&r| cx.s.is_live(r)).collect();
                let attackers: SmallVec<[(usize, ObjRef); 8]> = cx.s.combat.as_ref().unwrap().attackers.iter().enumerate().map(|(k, a)| (k, a.obj)).collect();
                let mut keyed: Vec<((u32, u32), Opt)> = Vec::new();
                for (k, a) in attackers {
                    if cx.s.is_live(a) && cx.can_block_pair(a, c) && cx.menace_feasible(&rem, Some((k, c))) {
                        keyed.push((cx.obj_key(seat, a), Opt::Block(a)));
                    }
                }
                keyed.sort_by_key(|k| k.0);
                let mut opts: Vec<Opt> = keyed.into_iter().map(|k| k.1).collect();
                if cx.menace_feasible(&rem, None) {
                    opts.push(Opt::NoBlock);
                }
                ask(cx, seat, DecisionKind::DeclareBlocker { creature: c }, opts);
                return Next::Await;
            }
            let n: usize = cx.s.combat.as_ref().unwrap().attackers.iter().map(|a| a.blockers.len()).sum();
            for a in cx.s.combat.as_mut().unwrap().attackers.iter_mut() {
                a.blocked = !a.blockers.is_empty();
            }
            let pairs: Vec<(ObjRef, ObjRef)> = cx.s.combat.as_ref().unwrap().attackers.iter().flat_map(|a| a.blockers.iter().map(move |&b| (a.obj, b))).collect();
            cx.emit(Event::BlockersDeclared { count: n.min(255) as u8 });
            for (attacker, blocker) in pairs {
                cx.emit(Event::Blocked { attacker, blocker });
            }
            Next::Done
        }
        CombatStage::Damage { .. } => run_damage(cx, f),
    }
}

pub fn feed(cx: &mut Cx, f: &mut CombatFrame, p: &Pending, opt: Opt) -> Next {
    match (&mut f.stage, p.kind, opt) {
        (CombatStage::Attackers { cands, i, declined_sig }, DecisionKind::DeclareAttacker { creature }, Opt::Attack(t)) => {
            let _ = (cands, declined_sig);
            cx.s.combat.as_mut().unwrap().attackers.push(AttackerInfo { obj: creature, target: t, blockers: SmallVec::new(), blocked: false });
            *i += 1;
            Next::Stay
        }
        (CombatStage::Attackers { i, declined_sig, .. }, DecisionKind::DeclareAttacker { creature }, Opt::NoAttack) => {
            cx.refresh();
            *declined_sig = Some(cx.obj_sig(creature));
            *i += 1;
            Next::Stay
        }
        (CombatStage::Blockers { i, .. }, DecisionKind::DeclareBlocker { creature }, Opt::Block(a)) => {
            let combat = cx.s.combat.as_mut().unwrap();
            let ai = combat.attackers.iter_mut().find(|x| x.obj == a).expect("blocked attacker");
            ai.blockers.push(creature);
            *i += 1;
            Next::Stay
        }
        (CombatStage::Blockers { i, .. }, DecisionKind::DeclareBlocker { .. }, Opt::NoBlock) => {
            *i += 1;
            Next::Stay
        }
        (CombatStage::Damage { plan, blocker_idx, remaining, .. }, DecisionKind::AssignDamage { blocker, .. }, Opt::Number(k)) => {
            plan.push((Target::Obj(blocker), k as u16));
            *remaining -= k as u16;
            *blocker_idx += 1;
            Next::Stay
        }
        _ => unreachable!("unexpected combat feed"),
    }
}

fn lethal_for(cx: &Cx, attacker: ObjRef, blocker: ObjRef) -> u16 {
    if cx.chars(attacker).keywords.contains(Keywords::DEATHTOUCH) {
        return 1;
    }
    let t = cx.chars(blocker).toughness;
    let d = cx.s.obj(blocker).damage as i32;
    (t - d).max(0) as u16
}

fn live_creature_blockers(cx: &Cx, blockers: &[ObjRef]) -> SmallVec<[ObjRef; 3]> {
    blockers
        .iter()
        .copied()
        .filter(|&b| cx.s.is_live(b) && cx.s.obj(b).zone == ZoneKind::Battlefield && cx.chars(b).types.contains(Types::CREATURE))
        .collect()
}

fn run_damage(cx: &mut Cx, f: &mut CombatFrame) -> Next {
    let (first_strike, mut attacker_idx, mut started, mut blocker_idx, mut remaining, mut plan) = match &f.stage {
        CombatStage::Damage { first_strike, attacker_idx, started, blocker_idx, remaining, plan } => (*first_strike, *attacker_idx, *started, *blocker_idx, *remaining, plan.clone()),
        _ => unreachable!(),
    };
    let atk_seat = cx.s.turn.active;
    let def_seat = atk_seat.other();
    let attackers: Vec<AttackerInfo> = cx.s.combat.as_ref().map(|c| c.attackers.clone()).unwrap_or_default();

    // CR 510.4: the regular damage step is for creatures that had neither first strike nor double
    // strike when the first-strike step began, plus double strikers. Record who had one of them
    // as the step begins (not who happened to deal damage: a 0-power first striker still counts).
    if first_strike && attacker_idx == 0 && !started {
        let mut all: SmallVec<[ObjRef; 12]> = SmallVec::new();
        for a in attackers.iter() {
            all.push(a.obj);
            all.extend(a.blockers.iter().copied());
        }
        for r in all {
            if cx.s.is_live(r) && cx.chars(r).keywords.intersects(Keywords::FIRST_STRIKE | Keywords::DOUBLE_STRIKE) {
                let combat = cx.s.combat.as_mut().unwrap();
                if !combat.first_struck.contains(&r) {
                    combat.first_struck.push(r);
                }
            }
        }
    }

    macro_rules! save {
        () => {
            f.stage = CombatStage::Damage { first_strike, attacker_idx, started, blocker_idx, remaining, plan: plan.clone() };
        };
    }

    while (attacker_idx as usize) < attackers.len() {
        let a = &attackers[attacker_idx as usize];
        let ar = a.obj;
        let live = cx.s.is_live(ar) && cx.s.obj(ar).zone == ZoneKind::Battlefield && cx.chars(ar).types.contains(Types::CREATURE);
        if !live || !cx.deals_damage_in(ar, first_strike) {
            attacker_idx += 1;
            continue;
        }
        let power = cx.chars(ar).power.max(0) as u16;
        let trample = cx.chars(ar).keywords.contains(Keywords::TRAMPLE);
        let blockers = live_creature_blockers(cx, &a.blockers);
        // Damage meant for a planeswalker that is gone is simply lost (Prevented in `deal`).
        let defending = match a.target {
            AttackTarget::Player(p) => Target::Player(p),
            AttackTarget::Walker(w) => Target::Obj(w),
        };
        if !started {
            if power == 0 {
                attacker_idx += 1;
                continue;
            }
            if !a.blocked {
                f.batch.push((Some(ar), defending, power as u32));
                attacker_idx += 1;
                continue;
            }
            if blockers.is_empty() {
                if trample {
                    f.batch.push((Some(ar), defending, power as u32));
                }
                attacker_idx += 1;
                continue;
            }
            if blockers.len() == 1 && !trample {
                let b = blockers[0];
                if trample {
                    let l = lethal_for(cx, ar, b).min(power);
                    if l > 0 {
                        f.batch.push((Some(ar), Target::Obj(b), l as u32));
                    }
                    if power > l {
                        f.batch.push((Some(ar), defending, (power - l) as u32));
                    }
                } else {
                    f.batch.push((Some(ar), Target::Obj(b), power as u32));
                }
                attacker_idx += 1;
                continue;
            }
            // Several blockers: the attacking player divides damage (CR 510.1c).
            started = true;
            remaining = power;
            blocker_idx = 0;
            plan.clear();
        }
        // Sequential assignment among several blockers.
        let nb = blockers.len() as u16;
        if blocker_idx >= nb || (!trample && blocker_idx == nb - 1) {
            // Finalize this attacker.
            if !trample && blocker_idx == nb - 1 {
                plan.push((Target::Obj(blockers[blocker_idx as usize]), remaining));
                remaining = 0;
            }
            for (t, n) in plan.iter() {
                if *n > 0 {
                    f.batch.push((Some(ar), *t, *n as u32));
                }
            }
            if trample && remaining > 0 {
                f.batch.push((Some(ar), defending, remaining as u32));
            }
            started = false;
            attacker_idx += 1;
            plan.clear();
            remaining = 0;
            blocker_idx = 0;
            continue;
        }
        let j = blocker_idx as usize;
        let b = blockers[j];
        let mut opts: Vec<Opt> = Vec::new();
        if trample && blocker_idx == nb - 1 {
            // Last blocker with trample: the rest may go to the player only if every blocker has
            // received lethal damage; otherwise everything must stay on the blockers.
            let all_prev_lethal = plan.iter().enumerate().all(|(k, (_, n))| *n >= lethal_for(cx, ar, blockers[k]));
            let lethal_j = lethal_for(cx, ar, b);
            for k in 0..=remaining {
                if k == remaining || (all_prev_lethal && k >= lethal_j) {
                    opts.push(Opt::Number(k as u32));
                }
            }
        } else {
            for k in 0..=remaining {
                opts.push(Opt::Number(k as u32));
            }
        }
        save!();
        ask(cx, atk_seat, DecisionKind::AssignDamage { attacker: ar, blocker: b, remaining }, opts);
        return Next::Await;
    }

    // Blockers deal damage to the creatures they block.
    for a in attackers.iter() {
        if !cx.s.is_live(a.obj) || cx.s.obj(a.obj).zone != ZoneKind::Battlefield {
            continue;
        }
        for &b in live_creature_blockers(cx, &a.blockers).iter() {
            if cx.deals_damage_in(b, first_strike) {
                let p = cx.chars(b).power.max(0) as u32;
                if p > 0 {
                    f.batch.push((Some(b), Target::Obj(a.obj), p));
                }
            }
        }
    }
    let _ = def_seat;
    let batch = std::mem::take(&mut f.batch);
    for (src, to, n) in batch {
        cx.deal_damage(src, to, n, true);
    }
    Next::Done
}

//! Replacement effects (CR 614/616). Only "enters the battlefield" replacements exist so far:
//! enters tapped, enters with counters, "can't enter", and "exile it instead". They are applied
//! in a fixed order (the entering card's own first, then permanents in battlefield order); every
//! combination in the pool gives the same result in any order (doc 01 section 9, scope note) except
//! "can't enter" (Grafdigger's Cage), which takes precedence over every other replacement (614.17c).

use crate::cx::Cx;
use crate::eval::Env;
use crate::ids::*;
use crate::ir::*;
use crate::ops::MoveOpts;
use crate::state::*;
use crate::types::*;

/// What the entering-replacements decided.
pub struct EnterMods {
    pub to: ZoneKind,
    pub opts: MoveOpts,
    pub prevented: bool,
}

impl<'a> Cx<'a> {
    /// Applies the enters-the-battlefield replacements to a proposed move of `obj` to the
    /// battlefield (the object may be a card in some zone, a spell, or a not-yet-placed token).
    pub fn enters_replacements(&mut self, obj: ObjRef, mut opts: MoveOpts) -> EnterMods {
        self.refresh();
        let mut to = ZoneKind::Battlefield;
        let (def_id, owner, from) = {
            let o = &self.s.objs[obj.slot as usize];
            (o.def, o.owner, o.zone)
        };
        let def_id = if opts.face { self.db.def(def_id).back_id.unwrap_or(def_id) } else { def_id };
        let would_ctrl = opts.controller.unwrap_or(owner);
        let db = self.db;
        // Collect (def, source controller, is_self, source object) in application order.
        let mut list: Vec<(EntersDef, Seat, Option<ObjRef>)> = Vec::new();
        let own_lost = self.would_lose_abilities(obj, would_ctrl);
        for a in &db.def(def_id).abilities {
            if own_lost {
                break;
            }
            if let AbilityDef::Enters(d) = a {
                if d.filter.self_only {
                    list.push((d.clone(), would_ctrl, None));
                }
            }
        }
        // Permanents entering together do not replace each other's entry (614.12).
        let bf: Vec<ObjRef> = if self.s.batch.depth > 0 {
            self.s.battlefield.iter().copied().filter(|r| self.s.batch.pre_bf.contains(r)).collect()
        } else {
            self.s.battlefield.clone()
        };
        for src in bf {
            let so = self.s.obj(src);
            if so.phased || self.s.derived[src.slot as usize].lost {
                continue;
            }
            let sctrl = so.controller;
            for a in &db.def(so.def).abilities {
                if let AbilityDef::Enters(d) = a {
                    if !d.filter.self_only {
                        list.push((d.clone(), sctrl, Some(src)));
                    }
                }
            }
        }
        // CR 614.17c: an event that "can't" happen can only be replaced by a self-replacement, so
        // "can't enter" effects (Grafdigger's Cage) apply first and nothing else gets to replace
        // the entry (Containment Priest's "exile it instead" never sees it).
        for pass in 0..2 {
        for (d, sctrl, src) in list.iter().cloned() {
            if (pass == 0) != matches!(d.effect, EntersEffect::Prevent) {
                continue;
            }
            if let Some(w) = d.cast {
                if w != opts.cast {
                    continue;
                }
            }
            if let Some(f) = d.from {
                if f != from {
                    continue;
                }
            }
            let mut f = d.filter.clone();
            let rel = f.controller;
            f.controller = Rel::Any;
            f.self_only = false;
            f.zone = from;
            let c = crate::derive::base_chars(db.def(def_id));
            if !self.filter_match(&f, obj, &c, false, sctrl) {
                continue;
            }
            match rel {
                Rel::Any => {}
                Rel::You => {
                    if would_ctrl != sctrl {
                        continue;
                    }
                }
                Rel::Opp => {
                    if would_ctrl == sctrl {
                        continue;
                    }
                }
            }
            if let Some(cond) = &d.cond {
                let cap = Captured::default();
                let env = Env { controller: sctrl, source: src.or(Some(obj)), source_lki: None, targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
                if !self.eval_cond(&env, cond) {
                    continue;
                }
            }
            match d.effect {
                EntersEffect::Tapped => opts.tapped = true,
                EntersEffect::Counters(k, n) => opts.counters = Some((k, n as u16)),
                EntersEffect::CountersDelved(k) => {
                    if opts.delved > 0 {
                        opts.counters = Some((k, opts.delved as u16));
                    }
                }
                EntersEffect::CountersIfWay(k, n, way) => {
                    if opts.cast_from.map(|c| c.1) == Some(way) {
                        opts.counters = Some((k, n as u16));
                    }
                }
                EntersEffect::EndStepAbility(a, way) => {
                    if opts.cast_from.map(|c| c.1) == Some(way) {
                        opts.end_step_ability = Some(a);
                    }
                }
                EntersEffect::Exile => {
                    to = ZoneKind::Exile;
                    opts.tapped = false;
                    opts.counters = None;
                    break;
                }
                EntersEffect::Prevent => return EnterMods { to, opts, prevented: true },
            }
        }
        }
        EnterMods { to, opts, prevented: false }
    }
}

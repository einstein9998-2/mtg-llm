//! Static restrictions on casting and activating ("can't"), cast permissions (Aluren) and the
//! checks the enumerator and mana-source scan make against them (CR 101.2: "can't" beats "can").

use crate::cx::Cx;
use crate::eval::Env;
use crate::ids::*;
use crate::ir::*;
use crate::state::Captured;
use crate::types::*;

/// First virtual cast way (see `PermitDef`).
pub const VIRTUAL_WAY_BASE: u8 = 128;
/// The way of a spell cast without paying its mana cost by an effect (during its resolution).
pub const FREE_WAY: u8 = 255;
/// A spell cast by an effect, paying energy equal to its mana value instead of its mana cost.
pub const ENERGY_WAY: u8 = 254;
/// A spell cast from the graveyard by an effect (Bilbo): normal cost; an instant or sorcery is
/// exiled instead of going to the graveyard.
pub const GY_WAY: u8 = 253;
/// Casting the Adventure half of a card from hand.
pub const ADVENTURE_WAY: u8 = 252;

impl<'a> Cx<'a> {
    fn rel_applies(who: Rel, seat: Seat, ctrl: Seat) -> bool {
        match who {
            Rel::Any => true,
            Rel::You => seat == ctrl,
            Rel::Opp => seat != ctrl,
        }
    }

    fn static_cond_holds(&mut self, cond: &Option<Cond>, ctrl: Seat, src: ObjRef) -> bool {
        match cond {
            None => true,
            Some(c) => {
                let cap = Captured::default();
                let env = Env { controller: ctrl, source: Some(src), source_lki: None, targets: &[], legal: &[], base: 0, x: 0, cap: &cap, each: [None, None], tlki: &[] };
                self.eval_cond(&env, c)
            }
        }
    }

    /// Does the spell (a card in some zone) match a spell filter, given the permanent `src` the
    /// filter comes from (for the chosen-name flag)?
    pub(crate) fn spell_matches(&mut self, f: &ObjFilter, spell: ObjRef, src: ObjRef, ctrl: Seat) -> bool {
        let mut f2 = f.clone();
        f2.controller = Rel::Any;
        f2.owner = Rel::Any;
        let c = self.chars(spell);
        if !self.filter_match(&f2, spell, &c, false, ctrl) {
            return false;
        }
        if f.chosen_name {
            let chosen = self.s.obj(src).chosen;
            if chosen == 0 || self.s.obj(spell).def.0 + 1 != chosen {
                return false;
            }
        }
        true
    }

    /// Is `seat` forbidden to cast `spell` (a card in hand, graveyard, ...) right now?
    pub(crate) fn cast_restricted(&mut self, seat: Seat, spell: ObjRef) -> bool {
        self.cast_restricted_x(seat, spell, None)
    }

    /// With `x = Some(n)` the spell is already on the stack with X announced: only the rules that
    /// look at its mana value (which now includes X) can still forbid it.
    pub(crate) fn cast_restricted_x(&mut self, seat: Seat, spell: ObjRef, x: Option<u8>) -> bool {
        // "Target player can't cast spells this turn" (Orim's Chant). It applies at announcement
        // only: a spell already on the stack (`x` is `Some`) was legally cast.
        if x.is_none() && self.s.player_fx.iter().any(|f| f.player == seat && f.fx == PlayerFx::CantCast) {
            return true;
        }
        if !self.db.has_restrict {
            return false;
        }
        self.refresh();
        let zone = self.s.obj(spell).zone;
        let bf: Vec<ObjRef> = self.s.battlefield.clone();
        for src in bf {
            let (ctrl, def_id, phased) = {
                let o = self.s.obj(src);
                (o.controller, o.def, o.phased)
            };
            if phased {
                continue;
            }
            let db = self.db;
            for a in &db.def(def_id).abilities {
                let r = match a {
                    AbilityDef::Restrict(r) => r,
                    _ => continue,
                };
                let (applies, cmc_over_lands, second_nc, from) = match &r.what {
                    Restriction::CantCast { applies, cmc_over_lands, second_noncreature, from } => (applies, *cmc_over_lands, *second_noncreature, from),
                    _ => continue,
                };
                if !Self::rel_applies(r.who, seat, ctrl) {
                    continue;
                }
                if x.is_some() && !cmc_over_lands {
                    continue;
                }
                if x.is_none() && !from.is_empty() && !from.contains(&zone) {
                    continue;
                }
                if !self.static_cond_holds(&r.cond, ctrl, src) {
                    continue;
                }
                if !self.spell_matches(applies, spell, src, ctrl) {
                    continue;
                }
                if cmc_over_lands {
                    let lands = self.s.battlefield.iter().filter(|&&b| self.s.objs[b.slot as usize].controller == seat && self.s.derived[b.slot as usize].types.contains(Types::LAND)).count() as u32;
                    let d = self.db.def(self.s.obj(spell).def);
                    let mv = d.mana_value() + x.unwrap_or(0) as u32 * d.cost.map(|c| c.x as u32).unwrap_or(0);
                    if mv <= lands {
                        continue;
                    }
                }
                if second_nc {
                    let noncreature = !self.chars(spell).types.contains(Types::CREATURE);
                    if !noncreature || self.s.players[seat.idx()].turn.noncreature_cast < 1 {
                        continue;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Is an activated ability of `src` (a permanent) forbidden right now?
    pub(crate) fn activation_blocked(&mut self, src: ObjRef, is_mana: bool) -> bool {
        if !self.db.has_restrict {
            return false;
        }
        self.refresh();
        let bf: Vec<ObjRef> = self.s.battlefield.clone();
        for rs in bf {
            let (ctrl, def_id, phased) = {
                let o = self.s.obj(rs);
                (o.controller, o.def, o.phased)
            };
            if phased {
                continue;
            }
            let db = self.db;
            for a in &db.def(def_id).abilities {
                let r = match a {
                    AbilityDef::Restrict(r) => r,
                    _ => continue,
                };
                let (sources, except_mana) = match &r.what {
                    Restriction::CantActivate { sources, except_mana } => (sources, *except_mana),
                    _ => continue,
                };
                if is_mana && except_mana {
                    continue;
                }
                if !self.static_cond_holds(&r.cond, ctrl, rs) {
                    continue;
                }
                let c = self.chars(src);
                let mut f = sources.clone();
                f.controller = Rel::Any;
                f.owner = Rel::Any;
                if !self.filter_match(&f, src, &c, false, ctrl) {
                    continue;
                }
                if sources.chosen_name {
                    let chosen = self.s.obj(rs).chosen;
                    if chosen == 0 || self.s.obj(src).def.0 + 1 != chosen {
                        continue;
                    }
                }
                return true;
            }
        }
        false
    }

    /// The virtual cast ways `seat` has for `card` from permissions on the battlefield.
    pub(crate) fn permit_ways(&mut self, seat: Seat, card: ObjRef) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        if self.db.permit_keys.is_empty() || self.s.obj(card).zone != ZoneKind::Hand {
            return out;
        }
        self.refresh();
        let bf: Vec<ObjRef> = self.s.battlefield.clone();
        for src in bf {
            let (ctrl, def_id, phased) = {
                let o = self.s.obj(src);
                (o.controller, o.def, o.phased)
            };
            if phased {
                continue;
            }
            let db = self.db;
            for a in &db.def(def_id).abilities {
                if let AbilityDef::Permit(p) = a {
                    let way = db.permit_way(&p.key).expect("permit key registered");
                    if out.contains(&way) || !Self::rel_applies(p.who, seat, ctrl) {
                        continue;
                    }
                    if self.spell_matches(&p.applies, card, src, ctrl) {
                        out.push(way);
                    }
                }
            }
        }
        out.sort();
        out
    }

    /// Does the permission behind virtual `way` grant flash?
    pub(crate) fn permit_flash(&self, way: u8) -> bool {
        let key = match self.db.permit_keys.get((way - VIRTUAL_WAY_BASE) as usize) {
            Some(k) => k,
            None => return false,
        };
        self.db.defs.iter().any(|d| d.abilities.iter().any(|a| matches!(a, AbilityDef::Permit(p) if &p.key == key && p.flash)))
    }
}

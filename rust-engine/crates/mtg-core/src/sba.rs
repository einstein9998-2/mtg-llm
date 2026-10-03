//! State-based actions (doc 01 section 8; CR 704). All applicable SBA in a pass are performed
//! simultaneously (CR 704.3).

use crate::cx::Cx;
use crate::decision::GameResult;
use crate::event::Lki;
use crate::ids::*;
use crate::ops::MoveOpts;
use crate::types::*;
use smallvec::SmallVec;

impl<'a> Cx<'a> {
    /// One SBA pass. Returns true if anything was performed.
    pub fn sba_pass(&mut self) -> bool {
        self.refresh();
        let mut changed = false;

        // Player losses (704.5a, 704.5b), evaluated together so simultaneous losses draw.
        let mut loses = [false; 2];
        for p in 0..2 {
            let pl = &self.s.players[p];
            if !pl.lost && (pl.life <= 0 || pl.drew_from_empty) {
                loses[p] = true;
            }
        }
        if loses[0] || loses[1] {
            for p in 0..2 {
                if loses[p] {
                    self.s.players[p].lost = true;
                }
            }
            self.s.result = Some(match (loses[0], loses[1]) {
                (true, true) => GameResult::Draw,
                (true, false) => GameResult::Win(Seat::P1),
                (false, true) => GameResult::Win(Seat::P0),
                _ => unreachable!(),
            });
            self.emit(crate::event::Event::GameEnded);
            return true;
        }

        // Ascend (CR 702.131b): the blessing is permanent once gained.
        if self.db.has_ascend {
            let mut count = [0u32; 2];
            let mut has = [false; 2];
            for &r in self.s.battlefield.iter() {
                let o = &self.s.objs[r.slot as usize];
                count[o.controller.idx()] += 1;
                if !o.phased && self.db.def(o.def).abilities.iter().any(|a| matches!(a, crate::ir::AbilityDef::Ascend)) {
                    has[o.controller.idx()] = true;
                }
            }
            for p in 0..2 {
                if has[p] && count[p] >= 10 {
                    self.s.players[p].blessing = true;
                }
            }
        }

        if self.dungeon_sba() {
            changed = true;
        }

        // Creature SBA, computed against one consistent snapshot, then performed together.
        let mut to_gy: SmallVec<[(ObjRef, Lki); 8]> = SmallVec::new();
        let n = self.s.battlefield.len();
        for i in 0..n {
            let r = self.s.battlefield[i];
            let o = &self.s.objs[r.slot as usize];
            let c = self.s.derived[r.slot as usize];
            if o.phased || !c.types.contains(Types::CREATURE) {
                continue;
            }
            let dies = if c.toughness <= 0 {
                true // 704.5f: not destruction; indestructible does not help
            } else {
                let lethal = o.damage as i32 >= c.toughness || o.dt_damage;
                lethal && !c.keywords.contains(Keywords::INDESTRUCTIBLE)
            };
            if dies {
                to_gy.push((r, Lki::placeholder()));
            }
        }
        for e in to_gy.iter_mut() {
            e.1 = self.lki(e.0);
        }
        // An Aura attached to something that no longer exists goes to the graveyard (704.5m).
        let mut aura_gy: SmallVec<[ObjRef; 2]> = SmallVec::new();
        for i in 0..n {
            let r = self.s.battlefield[i];
            let o = &self.s.objs[r.slot as usize];
            if self.db.def(o.def).subtypes.has(SUB_AURA) && !o.phased && !o.attached_to.map(|t| self.s.is_live(t)).unwrap_or(false) {
                aura_gy.push(r);
            }
        }
        self.begin_batch(&[]);
        for r in aura_gy {
            self.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
            changed = true;
        }
        self.end_batch();
        // A planeswalker with no loyalty counters goes to the graveyard (704.5i).
        let mut walker_gy: SmallVec<[ObjRef; 2]> = SmallVec::new();
        let n = self.s.battlefield.len();
        for i in 0..n {
            let r = self.s.battlefield[i];
            let o = &self.s.objs[r.slot as usize];
            if !o.phased && self.s.derived[r.slot as usize].types.contains(Types::PLANESWALKER) && o.counter(CounterKind::Loyalty) == 0 {
                walker_gy.push(r);
            }
        }
        self.begin_batch(&[]);
        for r in walker_gy {
            self.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
            changed = true;
        }
        self.end_batch();
        // Equipment whose creature is gone or no longer a creature becomes unattached (704.5n).
        let n = self.s.battlefield.len();
        for i in 0..n {
            let r = self.s.battlefield[i];
            if self.db.def(self.s.objs[r.slot as usize].def).subtypes.has(SUB_AURA) {
                continue;
            }
            if let Some(t) = self.s.objs[r.slot as usize].attached_to {
                let ok = self.s.is_live(t) && self.s.objs[t.slot as usize].zone == ZoneKind::Battlefield && self.s.derived[t.slot as usize].types.contains(Types::CREATURE);
                if !ok {
                    self.s.objs[r.slot as usize].attached_to = None;
                    self.s.derived_dirty = true;
                    changed = true;
                }
            }
        }
        // Counter annihilation (704.5q) — recorded before moving anything.
        let mut annihilate: SmallVec<[(ObjRef, u16); 4]> = SmallVec::new();
        for i in 0..n {
            let r = self.s.battlefield[i];
            let o = &self.s.objs[r.slot as usize];
            let p = o.counter(CounterKind::PlusOne);
            let m = o.counter(CounterKind::MinusOne);
            if p > 0 && m > 0 {
                annihilate.push((r, p.min(m)));
            }
        }
        for (r, k) in annihilate {
            self.add_counters(r, CounterKind::PlusOne, -(k as i32));
            self.add_counters(r, CounterKind::MinusOne, -(k as i32));
            changed = true;
        }
        self.begin_batch(&[]);
        for (r, lki) in to_gy {
            if self.s.is_live(r) {
                self.commit_move(r, ZoneKind::Graveyard, MoveOpts::default(), Some(lki));
                changed = true;
            }
        }
        self.end_batch();
        changed
    }
}

impl<'a> Cx<'a> {
    /// The first set of two or more legendary permanents with the same name under one controller
    /// (704.5j), in battlefield order.
    pub(crate) fn legend_violation(&mut self) -> Option<(Seat, Vec<ObjRef>)> {
        self.refresh();
        let legends: Vec<ObjRef> = self
            .s
            .battlefield
            .iter()
            .copied()
            .filter(|r| !self.s.objs[r.slot as usize].phased && self.s.derived[r.slot as usize].supertypes.contains(Supertypes::LEGENDARY))
            .collect();
        for (i, &a) in legends.iter().enumerate() {
            let (na, ca) = (&self.db.def(self.s.obj(a).def).name, self.s.obj(a).controller);
            let group: Vec<ObjRef> = legends[i..].iter().copied().filter(|&b| self.s.obj(b).controller == ca && &self.db.def(self.s.obj(b).def).name == na).collect();
            if group.len() > 1 && group[0] == a {
                return Some((ca, group));
            }
        }
        None
    }
}

impl Lki {
    pub(crate) fn placeholder() -> Lki {
        Lki {
            def: CardDefId(0),
            owner: Seat::P0,
            controller: Seat::P0,
            types: Types::empty(),
            supertypes: Supertypes::empty(),
            subtypes: SubtypeSet::default(),
            colors: Colors::empty(),
            power: 0,
            toughness: 0,
            counters: Vec::new(),
            keywords: Keywords::empty(),
            was_token: false,
            attached: None,
        }
    }
}

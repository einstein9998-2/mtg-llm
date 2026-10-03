//! Dungeons and the venture marker (CR 309, 701.49).

use crate::cx::Cx;
use crate::ids::*;
use crate::state::*;
use crate::types::*;

impl<'a> Cx<'a> {
    /// Puts a fresh dungeon card in the command zone with the marker on its topmost room.
    pub(crate) fn start_dungeon(&mut self, p: Seat, def: CardDefId) {
        let r = self.s.alloc_slot(ObjKind::Token, def, p);
        {
            let o = self.s.obj_mut_raw(r);
            o.zone = ZoneKind::Command;
        }
        let mut vid = [ViewId::NONE; 2];
        for v in 0..2usize {
            vid[v] = ViewId(self.s.next_vid[v]);
            self.s.next_vid[v] += 1;
        }
        self.s.vids[r.slot as usize] = vid;
        self.s.players[p.idx()].dungeon = Some(r);
        self.enter_room(p, 0);
    }

    /// Moves the venture marker into room `to`; its ability triggers.
    pub(crate) fn enter_room(&mut self, p: Seat, to: u8) {
        let d = self.s.players[p.idx()].dungeon.expect("a dungeon is in play");
        self.s.players[p.idx()].room = to;
        let def = self.s.obj(d).def;
        self.s.pending_triggers.push(PendingTrigger { source: d, def, ability: to, controller: p, cap: Default::default() });
    }

    /// The dungeon card leaves the game; its owner has completed it (CR 309.7).
    pub(crate) fn complete_dungeon(&mut self, p: Seat) {
        if let Some(d) = self.s.players[p.idx()].dungeon.take() {
            let def = self.s.obj(d).def;
            self.s.players[p.idx()].completed.push(def);
            self.s.players[p.idx()].room = 0;
            self.s.free_slot(d);
        }
    }

    /// CR 704.5t: a marker on the bottommost room whose dungeon is not the source of a room
    /// ability still waiting to resolve completes the dungeon.
    pub(crate) fn dungeon_sba(&mut self) -> bool {
        let mut changed = false;
        for p in 0..2usize {
            let Some(d) = self.s.players[p].dungeon else { continue };
            let def = self.s.obj(d).def;
            let bottom = self.db.def(def).dungeon.as_ref().map(|dd| dd.rooms[self.s.players[p].room as usize].exits.is_empty()).unwrap_or(false);
            if !bottom {
                continue;
            }
            let busy = self.s.pending_triggers.iter().any(|t| t.source == d) || self.s.stack.iter().any(|e| matches!(e.kind, StackKind::Ability { source, .. } if source == d));
            if !busy {
                self.complete_dungeon(Seat(p as u8));
                changed = true;
            }
        }
        changed
    }
}

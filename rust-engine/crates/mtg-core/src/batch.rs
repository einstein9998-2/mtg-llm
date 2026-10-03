//! Simultaneous zone changes (CR 603.10a, 614.12): a batch collects the zone-change events of one
//! multi-object action (state-based deaths, "destroy all", a delayed "return them") and matches
//! triggers once at its end, so that leaves-the-battlefield abilities see every object that left
//! together, enters-the-battlefield abilities see every object that entered together, and enters
//! replacements apply only from permanents that were there before the batch.

use crate::cx::Cx;
use crate::event::{Event, Lki};
use crate::ids::*;

#[derive(Clone, Default)]
pub struct Batch {
    pub(crate) depth: u8,
    /// The battlefield when the batch began.
    pub(crate) pre_bf: Vec<ObjRef>,
    /// Zone-change events held back, in order.
    pub(crate) events: Vec<Event>,
    /// Objects that left the battlefield during the batch: (ref after the move, definition, look-back).
    pub(crate) departed: Vec<(ObjRef, CardDefId, Lki)>,
    /// Look-back snapshots taken before the first departure, keyed by the pre-move ref.
    pub(crate) lki: Vec<(ObjRef, Lki)>,
}

impl<'a> Cx<'a> {
    /// Starts (or nests in) a simultaneous batch. `objs` are the permanents about to leave: when
    /// there are several, their look-back information is fixed now, before any of them moves.
    pub(crate) fn begin_batch(&mut self, objs: &[ObjRef]) {
        if self.s.batch.depth == 0 {
            self.s.batch.pre_bf = self.s.battlefield.clone();
            self.s.batch.events.clear();
            self.s.batch.departed.clear();
            self.s.batch.lki.clear();
        }
        self.s.batch.depth += 1;
        let on_bf: Vec<ObjRef> = objs.iter().copied().filter(|&r| self.s.is_live(r) && self.s.obj(r).zone == crate::types::ZoneKind::Battlefield).collect();
        if on_bf.len() > 1 {
            for r in on_bf {
                if !self.s.batch.lki.iter().any(|(x, _)| *x == r) {
                    let l = self.lki(r);
                    self.s.batch.lki.push((r, l));
                }
            }
        }
    }

    /// Ends a batch; the outermost one matches the held-back events.
    pub(crate) fn end_batch(&mut self) {
        debug_assert!(self.s.batch.depth > 0);
        self.s.batch.depth -= 1;
        if self.s.batch.depth > 0 {
            return;
        }
        let events = std::mem::take(&mut self.s.batch.events);
        for e in &events {
            self.match_triggers(e);
        }
        self.s.batch.departed.clear();
        self.s.batch.lki.clear();
        self.s.batch.pre_bf.clear();
    }
}

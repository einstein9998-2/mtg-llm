//! `Cx`: the working context (`&mut State` + `&CardDb`) every rules procedure runs in.

use crate::card::CardDb;
use crate::event::Event;
use crate::state::State;

pub struct Cx<'a> {
    pub s: &'a mut State,
    pub db: &'a CardDb,
}

impl<'a> Cx<'a> {
    pub fn new(s: &'a mut State, db: &'a CardDb) -> Cx<'a> {
        Cx { s, db }
    }

    /// The single emission point for events: logs them for the owning `Game`, and (from M2)
    /// feeds the trigger matcher.
    #[inline]
    pub fn emit(&mut self, e: Event) {
        if self.s.batch.depth > 0 {
            if let Event::ZoneChange { obj, def, from, lki, .. } = &e {
                if *from == crate::types::ZoneKind::Battlefield {
                    if let Some(l) = lki {
                        self.s.batch.departed.push((*obj, *def, l.clone()));
                    }
                }
                self.s.batch.events.push(e.clone());
                self.s.events.push(e);
                return;
            }
        }
        self.match_triggers(&e);
        self.s.events.push(e);
    }
}

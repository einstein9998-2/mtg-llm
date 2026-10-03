//! Identifier newtypes. Internal ids never reach agents (doc 02 section 4.1).

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Seat(pub u8);

impl Seat {
    pub const P0: Seat = Seat(0);
    pub const P1: Seat = Seat(1);
    #[inline]
    pub fn other(self) -> Seat {
        Seat(self.0 ^ 1)
    }
    #[inline]
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

/// A physical card. Assigned in **decklist order** (never library order), so it carries no
/// information about shuffles. Equal to the arena slot of the card object.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CardId(pub u16);

/// Reference to an object. `gen` increments on every zone change, so a stale reference means
/// "that object no longer exists as such" (CR 400.7, 608.2b).
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct ObjRef {
    pub slot: u16,
    pub gen: u16,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct CardDefId(pub u16);

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct DecisionId(pub u32);

/// Per-observer opaque object id (doc 02 section 4.2). 0 means "not visible to that observer".
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct ViewId(pub u32);

impl ViewId {
    pub const NONE: ViewId = ViewId(0);
}

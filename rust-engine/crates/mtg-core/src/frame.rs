//! The continuation stack (doc 01 section 6.2): every in-progress multi-step procedure is plain
//! data here, so the engine can pause at any decision and be cloned mid-resolution.

use crate::ids::*;
use crate::decision::Targets;
use crate::state::{PendingTrigger, StackEntry};
use crate::types::ZoneKind;
use smallvec::SmallVec;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Frame {
    /// Bottom frame: drives steps, priority and turn structure via `TurnState.mode`.
    Turn,
    Pregame(PregameFrame),
    Stabilize(StabFrame),
    Cast(CastFrame),
    Resolve(ResolveFrame),
    Combat(CombatFrame),
    Cleanup(CleanupFrame),
}

// ---- pregame ----------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PregameStage {
    Setup,
    /// Ask the player at `who` (position in play order) keep or mulligan.
    AskKeep,
    /// Player at `who` bottoms `left` more cards.
    Bottom { left: u8 },
    Done,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PregameFrame {
    pub stage: PregameStage,
    pub who: u8,
}

impl PregameFrame {
    pub fn new() -> PregameFrame {
        PregameFrame { stage: PregameStage::Setup, who: 0 }
    }
}

// ---- stabilize --------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StabStage {
    Sba,
    Triggers,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StabFrame {
    pub stage: StabStage,
    pub changed: bool,
    /// Some trigger was put on the stack in this pass (SBA are checked again).
    pub placed: bool,
    /// The trigger whose targets are being chosen, and how far along it is.
    pub cur: Option<PendingTrigger>,
    pub slot: u8,
    pub targets: Targets,
}

impl StabFrame {
    pub fn new() -> StabFrame {
        StabFrame { stage: StabStage::Sba, changed: false, placed: false, cur: None, slot: 0, targets: Targets::new() }
    }
}

// ---- casting ----------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CastStage {
    Start,
    /// Choose modes (`count` chosen so far, last chosen mode index + 1 in `next_min`).
    Modes,
    Replicate,
    X,
    Targets { slot: u8 },
    /// Choose graveyard cards to exile with delve.
    Delve,
    AddCost,
    /// Decide Phyrexian symbols: next color to look at, symbols of it still undecided.
    Phyrexian { color: u8, left: u8 },
    Picks { i: u8 },
    Pay,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CastFrame {
    /// The spell object on the stack, or the ability object.
    pub obj: ObjRef,
    pub controller: Seat,
    /// For an activated ability: (source permanent, ability index).
    pub ability: Option<(ObjRef, u8)>,
    pub stage: CastStage,
    pub from: ZoneKind,
    pub modes: u16,
    pub mode_count: u8,
    pub mode_next: u8,
    pub ctx: crate::cost::CastCtx,
}

impl CastFrame {
    pub fn spell(card: ObjRef, controller: Seat, way: u8, from: ZoneKind) -> CastFrame {
        CastFrame { obj: card, controller, ability: None, stage: CastStage::Start, from, modes: 0, mode_count: 0, mode_next: 0, ctx: crate::cost::CastCtx { way, not_hand: from != ZoneKind::Hand, ..Default::default() } }
    }
    pub fn activation(src: ObjRef, controller: Seat, idx: u8) -> CastFrame {
        CastFrame { obj: src, controller, ability: Some((src, idx)), stage: CastStage::Start, from: ZoneKind::Battlefield, modes: 1, mode_count: 1, mode_next: 1, ctx: Default::default() }
    }
}

// ---- resolution -------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResolveStage {
    Start,
    Ops,
    Finish,
}

/// State of an enclosing loop in the effect VM.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct LoopSt {
    pub list: SmallVec<[ObjRef; 6]>,
    /// Position in `list`. Wider than a byte: a list can exceed 255 objects (a token-copy loop),
    /// and a wrapped index made the loop run forever (RFC 0003).
    pub idx: u32,
    /// Remaining iterations of a `Repeat`.
    pub count: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResolveFrame {
    pub entry: StackEntry,
    pub stage: ResolveStage,
    pub pc: u16,
    /// Per target slot: still legal at resolution (CR 608.2b).
    pub legal: SmallVec<[bool; 2]>,
    pub countered: bool,
    pub fizzled: bool,
    /// Progress inside a multi-decision op (for example cards left to discard).
    pub aux: u16,
    /// Scratch registers of the running op (library effects).
    pub sc: [u16; 3],
    pub loops: [LoopSt; 2],
    /// Last-known information of object targets, captured when resolution starts (CR 608.2b).
    pub tlki: SmallVec<[Option<crate::event::Lki>; 2]>,
    /// Offset of the current mode's target slots within `entry.targets`.
    pub base: u8,
}

impl ResolveFrame {
    pub fn new(entry: StackEntry) -> ResolveFrame {
        ResolveFrame { entry, stage: ResolveStage::Start, pc: 0, legal: Default::default(), countered: false, fizzled: false, aux: 0, sc: [0; 3], loops: Default::default(), tlki: Default::default(), base: 0 }
    }
}

// ---- combat -----------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CombatStage {
    Attackers { cands: SmallVec<[ObjRef; 8]>, i: u16, declined_sig: Option<u64> },
    Blockers { cands: SmallVec<[ObjRef; 8]>, i: u16 },
    /// Combat damage assignment and dealing. `first_strike` selects which step.
    Damage { first_strike: bool, attacker_idx: u16, started: bool, blocker_idx: u16, remaining: u16, plan: SmallVec<[(crate::decision::Target, u16); 6]> },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CombatFrame {
    pub stage: CombatStage,
    /// Damage collected in this step, dealt simultaneously when assignment is complete.
    pub batch: SmallVec<[(Option<ObjRef>, crate::decision::Target, u32); 12]>,
}

// ---- cleanup ----------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CleanupStage {
    Discard,
    Wear,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CleanupFrame {
    pub stage: CleanupStage,
}

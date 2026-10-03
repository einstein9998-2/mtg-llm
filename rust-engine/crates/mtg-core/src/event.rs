//! Events: what actually happened, after replacement (doc 01 section 7.1). Closed enum so adding
//! a variant forces every consumer (redaction, hashing, trigger matching) to handle it.

use crate::decision::Target;
use crate::ids::*;
use crate::types::*;

/// Last-known information of an object that left a zone (doc 01 section 4.1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lki {
    pub def: CardDefId,
    pub owner: Seat,
    pub controller: Seat,
    pub types: Types,
    pub supertypes: Supertypes,
    pub subtypes: SubtypeSet,
    pub colors: Colors,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Keywords,
    pub was_token: bool,
    pub counters: Vec<(CounterKind, u16)>,
    /// The object it was attached to.
    pub attached: Option<ObjRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    TurnBegan { turn: u16, active: Seat },
    StepBegan { step: Step },
    /// Zone change. `obj` is the NEW reference. View ids are per seat at the time of the event.
    ZoneChange {
        obj: ObjRef,
        def: CardDefId,
        from: ZoneKind,
        to: ZoneKind,
        owner: Seat,
        controller: Seat,
        vid_old: [ViewId; 2],
        vid_new: [ViewId; 2],
        lki: Option<Lki>,
    },
    Drew { player: Seat, obj: ObjRef, def: CardDefId, vid: [ViewId; 2] },
    /// Drawing from an empty library (SBA 704.5b will make the player lose).
    DrewFromEmpty { player: Seat },
    Damage { source: Option<ObjRef>, to: Target, amount: u32, combat: bool },
    LifeChange { player: Seat, delta: i32 },
    SpellCast { obj: ObjRef, def: CardDefId, controller: Seat, vid: [ViewId; 2] },
    AbilityActivated { source: ObjRef, controller: Seat },
    /// A permanent became the target of the spell or ability `by` (controlled by `by_ctrl`).
    BecameTarget { obj: ObjRef, by: ObjRef, by_ctrl: Seat },
    LandPlayed { obj: ObjRef, controller: Seat },
    Tapped { obj: ObjRef },
    Untapped { obj: ObjRef },
    CountersChanged { obj: ObjRef, kind: CounterKind, delta: i32 },
    Shuffled { player: Seat },
    /// One creature was declared as an attacker (emitted before `AttackersDeclared`).
    Attacks { obj: ObjRef },
    AttackersDeclared { count: u8 },
    BlockersDeclared { count: u8 },
    /// `blocker` was declared as a blocker of `attacker` (emitted for every blocking pair once blockers are final).
    Blocked { attacker: ObjRef, blocker: ObjRef },
    SpellCountered { obj: ObjRef },
    SpellFizzled { obj: ObjRef },
    ManaAdded { player: Seat, color: ManaColor, n: u8 },
    MulliganTaken { player: Seat },
    HandKept { player: Seat, bottomed: u8 },
    TokenCreated { obj: ObjRef, def: CardDefId, controller: Seat },
    GameEnded,
}

//! Decisions and options (doc 02 section 2.2). Core-internal form; `mtg-view` translates these to
//! agent-facing `ActionDesc`s in the observer's `ViewId` space.

use crate::ids::*;
use smallvec::SmallVec;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Target {
    Player(Seat),
    Obj(ObjRef),
    /// An optional ("up to") slot left empty; keeps later slots aligned.
    None,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum AttackTarget {
    Player(Seat),
    /// A planeswalker an opponent controls.
    Walker(ObjRef),
}

/// One legal answer to a pending decision. Every offered `Opt` is pre-validated as completable.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Opt {
    Pass,
    PlayLand(ObjRef),
    /// Play the back face of a modal double-faced card as a land; `true` = pay 3 life so it enters untapped.
    PlayLandBack(ObjRef, bool),
    /// Cast `card` using the card's way `way` (0 = its mana cost, i = i-th alternative cost).
    Cast(ObjRef, u8),
    Activate { src: ObjRef, ability: u8 },
    /// Activate a mana ability of `src` producing one color (index into W U B R G C). `ability`
    /// is the ability's index, or 255 for the intrinsic ability of a basic land type.
    Mana { src: ObjRef, ability: u8, color: u8, pick: Option<ObjRef> },
    Target(Target),
    Card(ObjRef),
    Yes,
    No,
    Keep,
    Mulligan,
    Attack(AttackTarget),
    NoAttack,
    /// Block this attacker.
    Block(ObjRef),
    NoBlock,
    Number(u32),
    Done,
    /// A mode of a modal spell (index into its modes).
    Mode(u8),
    /// One of several alternatives (index).
    Choice(u8),
    /// A card name (the card definition's id).
    Name(u16),
    /// A creature type (index into `SUBTYPE_NAMES`).
    Type(u8),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CardsPurpose {
    DiscardToHandSize,
    /// Choosing an object to pay a cost (sacrifice, exile, return, discard).
    PayCost,
    DiscardEffect,
    /// The caster picks a card from the revealed hand of an opponent (Duress, Thoughtseize).
    DiscardChosen,
    BottomAfterMulligan,
    /// Brainstorm-style: cards from hand going back on top of the library.
    PutBack,
    /// Scry: cards going to the bottom (one at a time; `Done` ends the selection).
    ScryBottom,
    /// Surveil: cards going to the graveyard.
    SurveilGraveyard,
    /// Look at the top cards and take some into hand.
    LookTake,
    /// Library search; `Done` fails to find.
    Search,
    /// Ordering of cards going back on top of the library, chosen top first.
    OrderTop,
    /// Ordering of cards put on the bottom, chosen top-most first.
    OrderBottom,
    /// Thassa's Oracle: the card that stays on top; `Done` puts every looked-at card on the bottom.
    OracleTop,
    /// Doomsday: the cards (from library and graveyard) that make up the pile.
    DoomsdayPile,
    /// Surgical Extraction: cards found (and exiled) one at a time; `Done` ends the search.
    ExtractExile,
    /// An effect sacrifices a permanent the chooser controls.
    SacrificeEffect,
    /// An effect puts a card chosen from a zone onto the battlefield (or elsewhere); `Done` declines.
    PutOnto,
    /// Amass: the Army that gets the counters.
    AmassOnto,
    /// Show and Tell: each player in turn secretly picks a card (or none) to put onto the battlefield.
    PutEach,
    /// An effect lets the player cast one of the cards for free during its resolution; `Done` declines.
    CastFree,
    /// Raptor-style: cast the exiled card paying energy; `Done` declines.
    CastFromExile,
    /// Bilbo: cast a card from the graveyard; `Done` declines.
    CastFromGraveyard,
    /// Cloak and Dagger: the card (from a revealed hand, or a creature) to exile until the source leaves.
    ExileUntilLeaves,
    /// The Ring tempts you: the creature that becomes the Ring-bearer.
    RingBearer,
    /// Miracle: cast the card for its miracle cost; `Done` declines.
    CastMiracle,
    /// Ajani's ultimate: the permanent kept for one card type.
    KeepOne,
    /// Stronghold Gambit: the card chosen from hand.
    GambitChoose,
    /// Atraxa: cards put into hand from the revealed ones, at most one per card type; `Done` ends.
    RevealPick,
    /// Delve: graveyard cards exiled to pay generic mana; `Done` ends the selection.
    Delve,
}

impl CardsPurpose {
    pub fn is_order(self) -> bool {
        matches!(self, CardsPurpose::OrderTop | CardsPurpose::OrderBottom)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DecisionKind {
    Priority,
    Mulligan,
    ChooseTarget { slot: u8 },
    DeclareAttacker { creature: ObjRef },
    DeclareBlocker { creature: ObjRef },
    ChooseCards { purpose: CardsPurpose, remaining: u8 },
    /// "You may ...": yes or no.
    May,
    /// Choose which of your simultaneously triggered abilities goes on the stack next (the last
    /// one put on the stack resolves first). Options are `Opt::Choice(i)` = i-th pending trigger.
    OrderTriggers,
    /// Choose a color (options are `Opt::Choice(i)`, i = index into W U B R G).
    ChooseColor,
    /// Legend rule (704.5j): choose the one permanent to keep; the rest go to the graveyard.
    LegendRule,
    /// Choose a card name (`Opt::Name`); lands are offered only if `lands`.
    ChooseName { lands: bool },
    /// Choose a creature type (`Opt::Type`).
    ChooseType,
    /// Pay for a "counter unless its controller pays" effect? (yes = pay)
    PayUnless,
    ChooseMode { spell: ObjRef, chosen: u8 },
    ChooseX,
    /// Choose the dungeon of a fresh venture (`Opt::Choice(i)` = i-th dungeon of the database).
    ChooseDungeon,
    /// Choose the next room (`Opt::Choice(i)` = i-th exit of the current room).
    ChooseRoom,
    /// "You may choose new targets for the copy": yes or no.
    ChangeTargets,
    /// How many times to pay the replicate cost (`Opt::Number`).
    ChooseReplicate,
    /// Pay any amount of energy (`Opt::Number(n)`, 0 to the energy held).
    PayEnergyAmount,
    ChooseAddCost { spell: ObjRef },
    /// Pay this Phyrexian symbol with life instead of mana?
    PayPhyrexian { color: u8 },
    /// Combat damage division: assign this many points to `blocker` (value chosen from the mask).
    AssignDamage { attacker: ObjRef, blocker: ObjRef, remaining: u16 },
}

impl DecisionKind {
    /// Mulligan and bottoming are low-frequency decisions (doc 01 section 17).
    pub fn low_frequency(&self) -> bool {
        matches!(
            self,
            DecisionKind::Mulligan
                | DecisionKind::ChooseCards { purpose: CardsPurpose::BottomAfterMulligan, .. }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pending {
    pub id: DecisionId,
    pub seat: Seat,
    pub kind: DecisionKind,
    pub options: Vec<Opt>,
}

pub type Targets = SmallVec<[Target; 2]>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ApplyError {
    StaleDecision,
    BadIndex,
    GameOver,
    NoDecision,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum GameResult {
    Win(Seat),
    Draw,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    NeedDecision(Seat),
    GameOver(GameResult),
}

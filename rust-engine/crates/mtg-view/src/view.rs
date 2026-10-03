//! Agent-facing types: `Observation`, `Decision`, `ActionDesc`, `ViewEvent` (doc 02 section 2).
//!
//! Everything here is built by *projection*: new structs are constructed from allowed fields
//! only. There is no "serialize state then blank out secrets" path (doc 02 section 3 item 2).
//! Internal ids (`ObjRef`, `CardId`, slots, generations, event counters) never appear.

use mtg_core::ids::*;
use mtg_core::types::*;

/// A card as an observer sees it. Identity (`name`, `def`) is present only where the observer is
/// entitled to it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewCard {
    pub vid: ViewId,
    pub def: CardDefId,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ViewTarget {
    Player { me: bool },
    Object { vid: ViewId, name: String },
    /// The target left its zone (stale reference).
    Gone,
    /// An optional target left empty.
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewPermanent {
    pub vid: ViewId,
    pub def: CardDefId,
    pub name: String,
    pub controlled_by_me: bool,
    pub owned_by_me: bool,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage: u16,
    pub counters: Vec<(CounterKind, u16)>,
    pub types: Types,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Keywords,
    pub attacking: bool,
    /// What an attacking creature attacks (a creature that entered attacking has a target too).
    pub attack_target: Option<ViewAttackTarget>,
    pub blocking: Option<ViewId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewAttackTarget {
    Player { me: bool },
    Planeswalker(ViewId),
}

/// Things a player has outside the zones: dungeon, emblems, the Ring, the city's blessing.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ViewDesignations {
    /// Dungeon in the command zone and the room the venture marker is in (index into the dungeon).
    pub dungeon: Option<(String, u8)>,
    pub completed_dungeons: Vec<String>,
    pub emblems: Vec<String>,
    /// Times the Ring has tempted this player (0 to 4).
    pub ring_level: u8,
    pub ring_bearer: Option<ViewId>,
    pub blessing: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewStackItem {
    pub vid: ViewId,
    pub def: CardDefId,
    pub name: String,
    pub controlled_by_me: bool,
    pub is_ability: bool,
    pub targets: Vec<ViewTarget>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SelfView {
    pub life: i32,
    pub energy: u32,
    pub pool: [u8; 6],
    pub hand: Vec<ViewCard>,
    pub graveyard: Vec<ViewCard>,
    pub library_count: usize,
    pub lands_played: u8,
    /// The library cards whose identity and position this seat knows: from the top (top first)
    /// and from the bottom (bottom-most last).
    pub library_known_top: Vec<ViewCard>,
    pub library_known_bottom: Vec<ViewCard>,
    /// This player's cards in exile that this seat can see (exile is public).
    pub exile: Vec<ViewCard>,
    pub designations: ViewDesignations,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OppView {
    pub life: i32,
    pub energy: u32,
    pub hand_count: usize,
    pub graveyard: Vec<ViewCard>,
    pub library_count: usize,
    pub lands_played: u8,
    /// Cards of the opponent's hand this seat has seen (revealed or previously public).
    pub revealed_hand: Vec<ViewCard>,
    /// What this seat knows about the opponent's library order.
    pub library_known_top: Vec<ViewCard>,
    pub library_known_bottom: Vec<ViewCard>,
    pub exile: Vec<ViewCard>,
    pub designations: ViewDesignations,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionKind {
    Pass,
    PlayLand,
    CastSpell,
    ActivateAbility,
    /// Activate a mana ability, producing one color.
    ActivateMana,
    ChooseTarget,
    ChooseCard,
    Keep,
    Mulligan,
    Attack,
    NoAttack,
    Block,
    NoBlock,
    Number,
    Done,
    Yes,
    No,
    Mode,
    Choice,
    /// A card name or creature type (the label is the name).
    Name,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActionDesc {
    pub idx: u16,
    pub kind: ActionKind,
    /// The card/permanent/player acted on, in the observer's id space.
    pub subject: Option<ViewId>,
    /// Definition of the card acted on, when the chooser is entitled to know it (a card it is
    /// offered to pick is, even in a library; a hidden card is not).
    pub subject_def: Option<CardDefId>,
    /// Number payloads (damage assignment).
    pub value: Option<u32>,
    /// Short canonical text for prompts: card names only.
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ViewDecisionKind {
    Priority,
    Mulligan,
    ChooseTarget { slot: u8 },
    DeclareAttacker { creature: ViewId },
    DeclareBlocker { creature: ViewId },
    ChooseCards { purpose: mtg_core::decision::CardsPurpose, remaining: u8 },
    AssignDamage { attacker: ViewId, blocker: ViewId, remaining: u16 },
    May,
    OrderTriggers,
    ChooseMode { chosen: u8 },
    ChooseX,
    ChooseAddCost,
    PayPhyrexian,
    ChooseColor,
    ChooseDungeon,
    ChooseRoom,
    ChooseName,
    LegendRule,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Decision {
    pub id: DecisionId,
    pub seat: Seat,
    pub kind: ViewDecisionKind,
    pub options: Vec<ActionDesc>,
    /// Human/LLM-readable framing of what is being decided.
    pub context: String,
    /// `options.len() == 1` (doc 02 section 5.2).
    pub trivial: bool,
    pub low_frequency: bool,
}

/// Redacted event as seen by one seat (doc 02 section 7).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ViewEvent {
    TurnBegan { turn: u16, active_is_me: bool },
    StepBegan { step: Step },
    /// A visible card changed zone. `vid_new` is NONE if it went to a hidden zone.
    Moved { name: String, vid_old: ViewId, vid_new: ViewId, from: ZoneKind, to: ZoneKind, owner_is_me: bool },
    /// Cards moved between zones the observer cannot see: counts only.
    HiddenMove { owner_is_me: bool, from: ZoneKind, to: ZoneKind },
    DrewCard { vid: ViewId, name: String },
    OppDrewCard,
    DrewFromEmptyLibrary { me: bool },
    Damage { source: Option<ViewId>, to: ViewTarget, amount: u32, combat: bool },
    LifeChange { me: bool, delta: i32 },
    SpellCast { vid: ViewId, name: String, by_me: bool },
    AbilityActivated { source: Option<ViewId>, by_me: bool },
    LandPlayed { vid: ViewId, name: String, by_me: bool },
    Tapped { vid: ViewId },
    Untapped { vid: ViewId },
    CountersChanged { vid: ViewId, kind: CounterKind, delta: i32 },
    Shuffled { me: bool },
    AttackersDeclared { count: u8 },
    BlockersDeclared { count: u8 },
    SpellCountered { vid: ViewId },
    SpellFizzled { vid: ViewId },
    MulliganTaken { me: bool },
    HandKept { me: bool, bottomed: u8 },
    TokenCreated { vid: ViewId, name: String, by_me: bool },
    GameEnded,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Observation {
    pub seat: Seat,
    pub turn: u16,
    pub step: Step,
    pub active_is_me: bool,
    pub priority_is_me: bool,
    pub me: SelfView,
    pub opp: OppView,
    pub battlefield: Vec<ViewPermanent>,
    pub stack: Vec<ViewStackItem>,
    /// Events since this seat last acted, already redacted.
    pub events: Vec<ViewEvent>,
    /// Present when `seat` is the decider.
    pub decision: Option<Decision>,
    /// Hash over THIS struct only, never over `State`.
    pub view_hash: u64,
}

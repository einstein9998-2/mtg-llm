//! Card IR v2: the closed vocabulary of rules primitives (doc 03 section 3). Everything here is
//! plain data, derives `Serialize`/`Deserialize` (the RON surface syntax maps onto it one to
//! one), and derives `Hash` so `CardDb::content_hash` pins exactly what a record was made with.
//! No floats, no hash maps (doc 01 R4).
//!
//! Adding a variant is a core-adjacent change: it needs a handler in `exec`/`eval`, a lint, and
//! a test (doc 03 section 3 preamble, doc 01 section 14.1).

use crate::ids::*;
use crate::mana::ManaCost;
use crate::types::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// Small vocabulary
// ---------------------------------------------------------------------------------------------

/// Relative to the controller of the spell/ability being evaluated.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub enum Rel {
    #[default]
    Any,
    You,
    Opp,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Cmp {
    pub fn test(self, a: i32, b: i32) -> bool {
        match self {
            Cmp::Eq => a == b,
            Cmp::Ne => a != b,
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b,
        }
    }
}

fn cond_true() -> Cond {
    Cond::True
}

fn zone_bf() -> ZoneKind {
    ZoneKind::Battlefield
}

/// Object predicate used for targets, statics, costs and "each"/"all" effects (doc 03 section
/// 3.2). Empty/`None` fields mean "don't care".
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ObjFilter {
    #[serde(default = "zone_bf")]
    pub zone: ZoneKind,
    pub types_any: Types,
    pub types_not: Types,
    pub supertypes_any: Supertypes,
    pub supertypes_not: Supertypes,
    pub subtypes_any: SubtypeSet,
    pub subtypes_not: SubtypeSet,
    pub colors_any: Colors,
    pub colors_not: Colors,
    /// `Some(true)` = colorless, `Some(false)` = has a color.
    pub colorless: Option<bool>,
    pub keywords_any: Keywords,
    pub controller: Rel,
    pub owner: Rel,
    pub cmc: Option<(Cmp, i32)>,
    /// The mana value equals X of the ability or spell being put on the stack.
    pub cmc_x: bool,
    pub power: Option<(Cmp, i32)>,
    pub toughness: Option<(Cmp, i32)>,
    pub names: Vec<String>,
    pub not_self: bool,
    /// Only the source itself matches ("when this enters").
    pub self_only: bool,
    pub token: Option<bool>,
    pub tapped: Option<bool>,
    pub attacking: Option<bool>,
    /// `Some(true)`: an attacking creature that has not been blocked (ninjutsu).
    pub unblocked: Option<bool>,
    /// The object is its controller's Ring-bearer.
    pub ring_bearer: bool,
    /// `Some(true)`: it was put into its current zone from the battlefield this turn.
    pub from_bf_this_turn: Option<bool>,
    pub has_counter: Option<CounterKind>,
    /// For objects on the stack: spells vs abilities.
    pub stack: Option<StackSel>,
    /// Only the object the source permanent is attached to (equipped or enchanted creature).
    pub equipped_by_source: bool,
    /// `Some(true)`: the mana cost contains {X}.
    pub has_x: Option<bool>,
    /// The name is the one chosen for the source permanent (Disruptor Flute); the consumers that
    /// know the source check this (it is ignored by plain filter matching).
    pub chosen_name: bool,
    /// `Some(true)`: came onto the battlefield (or under its controller's control) this turn.
    pub entered_this_turn: Option<bool>,
    /// The object also matches if it matches any of these (an "or" of whole filters).
    pub alt: Vec<ObjFilter>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum StackSel {
    Spell,
    /// Any activated or triggered ability.
    Ability,
    Triggered,
}

impl Default for ObjFilter {
    fn default() -> Self {
        ObjFilter {
            zone: ZoneKind::Battlefield,
            types_any: Types::empty(),
            types_not: Types::empty(),
            supertypes_any: Supertypes::empty(),
            supertypes_not: Supertypes::empty(),
            subtypes_any: SubtypeSet::EMPTY,
            subtypes_not: SubtypeSet::EMPTY,
            colors_any: Colors::empty(),
            colors_not: Colors::empty(),
            colorless: None,
            keywords_any: Keywords::empty(),
            controller: Rel::Any,
            owner: Rel::Any,
            cmc: None,
            cmc_x: false,
            power: None,
            toughness: None,
            names: Vec::new(),
            not_self: false,
            self_only: false,
            token: None,
            tapped: None,
            attacking: None,
            unblocked: None,
            ring_bearer: false,
            from_bf_this_turn: None,
            has_counter: None,
            stack: None,
            equipped_by_source: false,
            has_x: None,
            chosen_name: false,
            entered_this_turn: None,
            alt: Vec::new(),
        }
    }
}

impl ObjFilter {
    pub fn new(zone: ZoneKind) -> ObjFilter {
        ObjFilter { zone, ..ObjFilter::default() }
    }
    pub fn on_battlefield(types: Types) -> ObjFilter {
        ObjFilter { types_any: types, ..ObjFilter::default() }
    }
    pub fn with_controller(mut self, r: Rel) -> ObjFilter {
        self.controller = r;
        self
    }
}

/// What a single target slot may be: an object matching `objects`, and/or a player. `optional`
/// slots ("up to one") may be left empty.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TargetSpec {
    pub objects: Option<ObjFilter>,
    /// `Some(rel)` if players matching `rel` (relative to the controller) are legal.
    pub players: Option<Rel>,
    pub optional: bool,
    /// The target must differ from the targets chosen in earlier slots of the same spell
    /// ("one or two targets": the same word may not name one object twice).
    pub distinct: bool,
    /// "X target ...": the slot repeats X times (X as chosen when the spell is cast).
    #[serde(default)]
    pub x_count: bool,
}

impl TargetSpec {
    pub fn creature() -> TargetSpec {
        TargetSpec { objects: Some(ObjFilter::on_battlefield(Types::CREATURE)), ..Default::default() }
    }
    pub fn any_target() -> TargetSpec {
        TargetSpec { objects: Some(ObjFilter::on_battlefield(Types::CREATURE | Types::PLANESWALKER)), players: Some(Rel::Any), optional: false, distinct: false, x_count: false }
    }
    pub fn player() -> TargetSpec {
        TargetSpec { players: Some(Rel::Any), ..Default::default() }
    }
    pub fn spell() -> TargetSpec {
        TargetSpec { objects: Some(ObjFilter::new(ZoneKind::Stack)), ..Default::default() }
    }
}

// ---------------------------------------------------------------------------------------------
// References, values, conditions
// ---------------------------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PRef {
    You,
    Opp,
    Active,
    /// A target slot holding a player.
    Target(u8),
    /// The controller / owner of an object (live object, or last-known for a departed one).
    ControllerOf(ORef),
    OwnerOf(ORef),
    /// The player named by the triggering event (who drew, who cast, who was damaged, ...).
    EventPlayer,
    /// The player to the left in the iteration of `ForEachPlayer`.
    Each,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ORef {
    /// The source of the ability (the spell itself for a spell).
    This,
    /// A target slot holding an object.
    Target(u8),
    /// The object bound by the enclosing `ForEach` at this nesting depth.
    Each(u8),
    /// The object named by the triggering event.
    EventObj,
    /// The i-th object (new object) moved by the most recent `Exile`/`MoveTo`/`Bounce` effect.
    Moved(u8),
    /// The object the source is attached to (last known if the source has left).
    AttachedTo,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Expr {
    Const(i32),
    X,
    /// "That much" (damage dealt, life gained, ...) from the triggering event.
    EventAmount,
    Count(ObjFilter),
    Life(PRef),
    CardsInHand(PRef),
    CardsInLibrary(PRef),
    CardsInGraveyard(PRef),
    SpellsCastThisTurn(PRef),
    LifeLostThisTurn(PRef),
    LifeGainedThisTurn(PRef),
    /// The Ring's level for the player (0 = the Ring has not tempted them).
    RingLevel(PRef),
    /// Cards the player drew this turn so far.
    CardsDrawnThisTurn(PRef),
    PowerOf(ORef),
    ToughnessOf(ORef),
    CmcOf(ORef),
    /// Times the replicate cost was paid for the spell on the stack.
    Replicated(ORef),
    /// Mana symbols of these colors in the mana costs of permanents `PRef` controls (hybrid counts for each color).
    Devotion(Colors, PRef),
    Counters(ORef, CounterKind),
    /// Number of different colors of mana spent to cast the spell (converge).
    ColorsSpent(ORef),
    Energy(PRef),
    /// Mana spent to cast the spell (-1 if it is no longer on the stack).
    ManaSpent(ORef),
    /// Distinct card types among cards in the graveyard(s) of the given player (`None` = all).
    GraveyardTypes(Option<PRef>),
    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Times(Box<Expr>, Box<Expr>),
    Max(Box<Expr>, Box<Expr>),
    Min(Box<Expr>, Box<Expr>),
    HalfDown(Box<Expr>),
    HalfUp(Box<Expr>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Cond {
    True,
    Not(Box<Cond>),
    And(Vec<Cond>),
    Or(Vec<Cond>),
    Cmp(Expr, Cmp, Expr),
    /// It is the given player's turn.
    IsActive(PRef),
    /// `who` controls at least `at_least` objects matching the filter (filter controller is
    /// ignored; `who` decides).
    Controls { who: PRef, filter: ObjFilter, at_least: u8 },
    /// The object currently matches the filter (LKI for departed objects).
    Matches(ORef, ObjFilter),
    /// Delirium: four or more card types among cards in `who`'s graveyard.
    Delirium(PRef),
    /// The source permanent was cast, from `from` (any zone if `None`) by the alternative cost
    /// with key `key` (any way if `None`; `Some("")` = by its mana cost).
    WasCast { from: Option<ZoneKind>, key: Option<String> },
    /// The object is (still) controlled by the player.
    ControlledBy(ORef, PRef),
    /// `who` cast a spell this turn that has at least one of these colors.
    CastColor { who: PRef, colors: Colors },
    /// Revolt: a permanent the player controlled left the battlefield this turn.
    Revolt(PRef),
    /// The player has the city's blessing.
    Blessing(PRef),
    /// This ability of the source already did its once-each-turn thing this turn.
    UsedThisTurn(u8),
    /// The object has a counter of this kind on it.
    HasCounter(ORef, CounterKind),
    /// The player has completed the named dungeon.
    Completed { who: PRef, dungeon: String },
}

// ---------------------------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Rcpt {
    /// A target slot holding a creature, planeswalker or player.
    Target(u8),
    Player(PRef),
    Obj(ORef),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Objs {
    One(ORef),
    All(ObjFilter),
    /// The objects a delayed trigger captured when it was created.
    Captured,
    /// Every still-legal target object from this slot on ("X target creatures").
    TargetsFrom(u8),
    /// The topmost card of `who`'s graveyard that matches the filter (none if no card does).
    TopOfGraveyard { who: PRef, filter: ObjFilter },
}

/// When a delayed triggered ability fires.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum DelayWhen {
    /// At the beginning of the next end step (this turn's if it has not begun yet).
    NextEndStep,
    /// At the beginning of the next upkeep, whoever's turn it is.
    NextUpkeep,
    /// At the beginning of the next end of combat step.
    EndCombat,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Effect {
    // ---- control flow ----
    Seq(Vec<Effect>),
    If { cond: Cond, then: Box<Effect>, els: Option<Box<Effect>> },
    /// `who` may choose to do `then` (yes/no decision).
    May { who: PRef, then: Box<Effect> },
    /// Runs `body` once per object matching the filter, snapshot taken at loop start; the current
    /// object is `ORef::Each(depth)` where depth is the nesting level of `ForEach` (outermost 0).
    ForEach { filter: ObjFilter, body: Box<Effect> },
    Repeat { n: Expr, body: Box<Effect> },
    // ---- leaves ----
    Damage { amount: Expr, to: Rcpt },
    Draw { who: PRef, n: Expr },
    GainLife { who: PRef, n: Expr },
    LoseLife { who: PRef, n: Expr },
    Destroy(Objs),
    Exile(Objs),
    Bounce(Objs),
    /// Counter the spell in the given target slot.
    CounterSpell(u8),
    /// Counter the spell the reference names (it must still be on the stack).
    CounterSpellOf(ORef),
    /// Counter the spell in the target slot unless its controller pays `pays` (offered only if
    /// the payment is possible).
    CounterUnless { target: u8, pays: ManaCost },
    /// Counter the spell or ability that caused this trigger (`EventObj`) unless its controller pays (ward).
    CounterEventUnless { pays: ManaCost },
    /// The player chooses `n` cards from hand to discard (all if fewer).
    Discard { who: PRef, n: Expr },
    Mill { who: PRef, n: Expr },
    CreateToken { token: String, n: Expr, who: PRef },
    AddCounters { objs: Objs, kind: CounterKind, n: Expr },
    Tap(Objs),
    Untap(Objs),
    /// Mana abilities and mana-producing spells.
    AddMana { color: ManaColor, n: u8 },
    /// `n` mana of one color of the controller's choice (never colorless).
    AddManaAny { n: Expr },
    /// Creates a continuous effect on the objects as they are now (CR 611.2c: the set is locked in).
    Continuous { objs: Objs, layer: Layer, effect: ContEffect, until: Until },
    /// Creates a continuous effect on a player.
    PlayerFx { who: PRef, fx: PlayerFx, until: Until },
    /// Look at the top `n` cards; put any of them on the bottom (you choose which and the order),
    /// the rest back on top in any order.
    Scry(Expr),
    /// Look at the top `n` cards; put any of them into your graveyard, the rest back on top in
    /// any order.
    Surveil(Expr),
    /// Put `n` cards from your hand on top of your library in any order (all of them if fewer).
    PutBack(Expr),
    /// Look at the top `n` cards and put them back in any order.
    Reorder(Expr),
    /// Look at the top `look` cards; put `take` of them into your hand and the rest on the bottom
    /// of your library in any order.
    LookTake { look: Expr, take: u8 },
    /// Search your library for a card matching the filter (you may fail to find), put it into
    /// `dest`, then shuffle.
    Search {
        #[serde(default = "pref_you")]
        who: PRef,
        filter: ObjFilter,
        dest: ZoneKind,
        #[serde(default)]
        tapped: bool,
    },
    Shuffle(PRef),
    /// Counter the spell in the target slot and exile it instead of putting it into the graveyard.
    CounterSpellExile(u8),
    /// Move the objects to `to` (return from graveyard, reanimate, ...); `ctrl` is the new
    /// controller on the battlefield (default: the owner).
    MoveTo {
        objs: Objs,
        to: ZoneKind,
        #[serde(default)]
        ctrl: Option<PRef>,
        #[serde(default)]
        tapped: bool,
    },
    /// The controller sacrifices the objects (no regeneration or indestructible concerns).
    Sacrifice(Objs),
    /// Counter the activated or triggered ability in the target slot.
    CounterAbility(u8),
    /// `chooser` (the controller) looks at `who`'s hand and chooses a card matching the filter;
    /// `who` discards it. The hand is revealed to the chooser.
    DiscardChoose { who: PRef, filter: ObjFilter },
    /// Attach `what` (an Equipment or Aura) to `to`. Does nothing unless `to` is a creature on the battlefield.
    Attach { what: ORef, to: ORef },
    /// Look at the top card of `who`'s library (the controller learns it and its position).
    LookTop { who: PRef },
    /// Creates a delayed trigger: at `when`, ability `ability` of this source (an `AbilityDef::Triggered`
    /// whose `on` is `Delayed`) triggers for the controller.
    Delay { when: DelayWhen, ability: u8 },
    /// Until `until`, whenever a creature an opponent controls attacks you or a planeswalker you
    /// control, ability `ability` (a delayed ability of this card; `EventObj` = the attacker) triggers.
    WatchAttacks { ability: u8, until: Until },
    /// Look at the top `look` cards; put up to one of them back on top and the rest on the bottom in
    /// a random order (Thassa's Oracle).
    LookPutTop { look: Expr },
    /// The player wins the game.
    Win(PRef),
    /// Counter the spell or the ability in the target slot.
    CounterAny(u8),
    /// Put `n` copies of the spell the reference names onto the stack, controlled by the effect's
    /// controller, who may choose new targets for each copy (storm, replicate).
    CopySpell { what: ORef, n: Expr },
    /// The controller chooses a card name (nonland unless `lands`); it is remembered until the
    /// resolution ends (`DiscardNamed`).
    ChooseName { #[serde(default)] lands: bool },
    /// `who` reveals their hand and discards every card with the chosen name.
    DiscardNamed { who: PRef },
    /// Exiles the objects, then creates a delayed trigger that remembers them (they are the
    /// `Captured` objects of the delayed ability).
    ExileUntil { objs: Objs, when: DelayWhen, ability: u8 },
    /// `who` may pay `pays`; if they do, `then` happens (yes/no, then the payment is made).
    MayPay { who: PRef, pays: ManaCost, then: Box<Effect> },
    /// `who` chooses a permanent they control that matches the filter and sacrifices it (no
    /// decision if only one card qualifies, nothing if none does).
    SacrificeChosen { who: PRef, filter: ObjFilter },
    /// `who` chooses a card in `from` that matches the filter (and has mana value `cmc`, if given)
    /// and puts it into `to`; with `optional` they may choose none.
    PutChosen { who: PRef, from: ZoneKind, filter: ObjFilter, cmc: Option<Expr>, to: ZoneKind, #[serde(default)] tapped: bool, #[serde(default)] optional: bool },
    /// Exiles every card in `who`'s graveyard.
    ExileGraveyard { who: PRef },
    GainEnergy { who: PRef, n: Expr },
    LoseEnergy { who: PRef, n: Expr },
    /// Puts ability `ability` of this source (a triggered ability with `on: Delayed`) on the stack
    /// as a reflexive trigger (CR 603.12), with its own targets.
    Reflexive { ability: u8 },
    /// Like `Delay`, capturing the objects the previous `MoveTo`/`CreateToken` produced.
    DelayMoved { when: DelayWhen, ability: u8 },
    /// Reveal the top `look` cards of the controller's library; for each card type they may put a
    /// card of that type among them into their hand (one card cannot serve two types); the rest go
    /// to the bottom in a random order (Atraxa).
    RevealPickTypes { look: Expr },
    /// Doomsday: the controller picks `n` cards from library and graveyard, exiles the rest, and
    /// puts the pile on top of the library in an order they choose.
    Pile { n: u8 },
    /// Surgical Extraction: the controller searches the owner of the target card's graveyard, hand
    /// and library for any number of cards with the target's name and exiles them; that player shuffles.
    ExtractSame { target: u8 },
    /// Animate Dead: if the source Aura is on the battlefield, return the card it is attached to
    /// from the graveyard under the controller's control and attach the Aura to it.
    ReanimateAttach,
    /// Cavern of Souls: one mana of any color (chosen on activation), restricted to creature
    /// spells of the type chosen for the source as it entered.
    AddManaRestricted,
    /// Amped Raptor: exile cards from the top of the library until a nonland card is exiled; the
    /// controller may cast it by paying energy equal to its mana value.
    ExileCastEnergy,
    /// The Ring tempts you (CR 701.54): the ring level goes up (to 4 at most), and you choose a
    /// creature you control as your Ring-bearer.
    RingTempts,
    /// Like `Delay`, remembering the object that caused the trigger (`EventObj`).
    DelayEventObj { when: DelayWhen, ability: u8 },
    /// Exile the source permanent, then return it to the battlefield transformed under its owner's control.
    ExileReturnTransformed,
    /// Ajani, Nacatl Avenger's ultimate: each opponent chooses an artifact, a creature, an
    /// enchantment and a planeswalker among their nonland permanents, then sacrifices the rest.
    KeepOneOfEach,
    /// Raph & Mikey: reveal cards from the top of the library until a creature card is revealed; it
    /// enters tapped and attacking what the source attacks, the rest go to the bottom in a random order.
    DigCreatureAttacking,
    /// Stronghold Gambit: each player chooses a card in hand, all are revealed, and the owners of the
    /// creature cards with the lowest mana value among them put them onto the battlefield.
    GambitReveal,
    /// Miracle: the controller may cast the source card (still in their hand) using the alternative
    /// cost with this index (1 = the first `Alt`), during the resolution of the trigger.
    MiracleCast(u8),
    /// Praesidium Protectiva: exile the card that caused this trigger (in its owner's graveyard)
    /// and the top six cards of its owner's library, shuffle that pile and put it back on top.
    PileBack,
    /// Ninjutsu: put the source card (still in hand) onto the battlefield tapped and attacking the
    /// player or planeswalker the returned attacker was attacking (kept in the ability's captured data).
    NinjutsuEnter,
    /// The controller gets an emblem (a command-zone object of the named definition).
    CreateEmblem(String),
    /// Wrath of the Skies: the controller gets X energy, may pay any amount of it, then every
    /// permanent matching the filter with mana value at most the energy paid is destroyed.
    EnergyWrath { filter: ObjFilter },
    /// The controller may cast a card matching the filter from their graveyard (during resolution).
    CastFromGraveyard { filter: ObjFilter },
    /// Cloak and Dagger: the target player reveals their hand; the controller may exile a nonland
    /// card from it or the chosen creature (target slot `creature`) until the source leaves.
    ExileUntilLeaves { creature: u8 },
    /// A token that is a copy of the exiled card(s), except it is a P/T creature of a color and
    /// creature type (Lazotep Quarry).
    CopyAs { objs: Objs, pt: (i16, i16), color: Colors, subtype: u8 },
    /// Each player, starting with the active player, may choose a card in their hand matching the
    /// filter; then all chosen cards are put onto the battlefield at the same time (Show and Tell).
    EachPutFromHand { filter: ObjFilter },
    /// Exile the top `n` cards of your library; you may play them for as long as they stay exiled.
    ExilePlay { n: Expr },
    /// The tokens or permanents just put onto the battlefield (the previous instruction) become
    /// tapped and attacking; their controller chooses the player or planeswalker (508.4), skipped
    /// when only the player is possible (Mobilize).
    TapAttackMoved,
    /// The cards just moved (by the previous instruction) may be cast by their owner for as long
    /// as they stay exiled (Quantum Riddler's warp: "you may cast it from exile on a later turn").
    AllowCastMoved,
    /// Draw `n` cards and reveal them; you may cast one of them without paying its mana cost.
    DrawRevealCast { n: Expr },
    /// Records that ability `i` of the source has been used this turn (see `Cond::UsedThisTurn`).
    MarkUsed(u8),
    /// Venture into the dungeon (CR 701.49) for the controller.
    Venture,
    /// Like `May` with an alternative: `then` if `who` says yes, else `els`.
    MayElse { who: PRef, #[serde(default = "cond_true")] can: Cond, then: Box<Effect>, els: Box<Effect> },
    /// Runs `body` once for each player, the active player first; `PRef::Each` names the current one.
    ForEachPlayer { body: Box<Effect> },
    /// `who` sacrifices one permanent for each filter (a different permanent per filter), choosing
    /// them one by one among the permanents that still allow a complete assignment.
    SacrificeOneEach { who: PRef, filters: Vec<ObjFilter> },
    /// Creates a token that is a copy of each token the objects name (for the effect's controller).
    CopyToken(Objs),
    /// Exiles the objects and remembers them as linked to this source (Skyclave Apparition).
    ExileLinked(Objs),
    /// For every object linked to this source: its owner creates a token whose power and
    /// toughness equal the exiled card's mana value, then the links are forgotten.
    LinkedToken { token: String },
    /// Amass: if the controller has no Army, they create the named Army token; then they put `n`
    /// +1/+1 counters on an Army they control (their choice if several).
    Amass { token: String, n: Expr },
    /// The controller chooses a card matching the filter among the cards the previous milling or
    /// moving effect produced (those still in the zone they went to) and moves it to `to`.
    TakeMoved { filter: ObjFilter, to: ZoneKind, #[serde(default)] optional: bool },
    /// Sets power and toughness of the objects to `x`/`x`, for `until`.
    SetPTX { objs: Objs, x: Expr, until: Until },
}

fn pref_you() -> PRef {
    PRef::You
}

impl Effect {
    pub fn nothing() -> Effect {
        Effect::Seq(Vec::new())
    }
}

// ---------------------------------------------------------------------------------------------
// Abilities
// ---------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum CostItem {
    TapSelf,
    Mana(ManaCost),
    PayLife(u8),
    SacrificeSelf,
    /// Sacrifice a permanent you control matching the filter (chosen at cast time).
    Sacrifice(ObjFilter),
    /// Discard a card from hand matching the filter (the spell itself is never a candidate).
    Discard(ObjFilter),
    /// Exile another card from your hand matching the filter.
    ExileFromHand(ObjFilter),
    /// Exile a card from your graveyard matching the filter.
    ExileFromGraveyard(ObjFilter),
    /// Return a permanent you control matching the filter to its owner's hand.
    ReturnToHand(ObjFilter),
    PayEnergy(u8),
    /// Exile any number of other cards from your graveyard with at least this many card types
    /// among them (escape).
    ExileTypes(u8),
    /// Discard your hand (Lion's Eye Diamond).
    DiscardHand,
    /// Discard this card (activated from hand: cycling, channel).
    DiscardSelf,
    /// A loyalty cost (CR 606): add (positive) or remove (negative, at most the loyalty there is)
    /// loyalty counters. Only one loyalty ability per permanent per turn, at sorcery speed.
    Loyalty(i8),
}

impl CostItem {
    /// Does paying this cost involve choosing an object?
    pub fn needs_pick(&self) -> bool {
        matches!(self, CostItem::Sacrifice(_) | CostItem::Discard(_) | CostItem::ExileFromHand(_) | CostItem::ExileFromGraveyard(_) | CostItem::ReturnToHand(_))
    }
}

/// An alternative way to cast a spell (CR 118.9): replaces the mana cost. `zone` is where the
/// card must be (hand by default; graveyard for flashback/escape; exile for warp recasts).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AltCost {
    pub label: String,
    /// Stable name used by scenario scripts (`return_island`, `force_of_will`, ...).
    #[serde(default)]
    pub key: String,
    pub cost: Vec<CostItem>,
    /// Checked at cast time with the caster as `You`.
    #[serde(default)]
    pub cond: Option<Cond>,
    #[serde(default = "zone_hand")]
    pub zone: ZoneKind,
    /// Exile the spell instead of putting it into the graveyard when it leaves the stack
    /// (flashback, escape-like).
    #[serde(default)]
    pub exile_after: bool,
}

fn zone_hand() -> ZoneKind {
    ZoneKind::Hand
}

/// A cost added on top of the base or alternative cost, chosen at cast time (CR 601.2b):
/// exactly one of `options` is paid; if `optional`, paying none is also allowed (kicker).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AddCostDef {
    pub options: Vec<Vec<CostItem>>,
    #[serde(default)]
    pub optional: bool,
}

/// A static cost change applying to spells matching `applies` (a filter on the spell; zone and
/// controller are ignored) cast by `who` (relative to the permanent's controller).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct CostModDef {
    pub applies: ObjFilter,
    #[serde(default)]
    pub who: Rel,
    /// Positive = tax, negative = reduction (generic only, never below zero).
    pub generic: i8,
    #[serde(default)]
    pub cond: Option<Cond>,
    /// Applies only when the spell is cast outside its caster's own turn (Defense Grid).
    #[serde(default)]
    pub not_casters_turn: bool,
    /// Applies only to spells cast from a zone other than the hand (Bilbo).
    #[serde(default)]
    pub not_from_hand: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Timing {
    Instant,
    Sorcery,
}

/// Targets plus the effect that uses them: one mode of a spell, or the whole of an ability.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Body {
    pub label: String,
    pub targets: Vec<TargetSpec>,
    pub effect: Option<Effect>,
}

/// A room of a dungeon card; `exits` index the dungeon's rooms (none = the bottommost room).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct RoomDef {
    pub name: String,
    #[serde(default)]
    pub exits: Vec<u8>,
}

/// The rooms of a dungeon card, topmost first. Room `i`'s ability is the card's ability `i`
/// (a triggered ability with `on: Delayed`, `zone: Command`).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct DungeonDef {
    pub rooms: Vec<RoomDef>,
}

/// A spell ability. Non-modal spells may be written inline (`targets`/`effect`); loading moves
/// them into a single mode, so after `CardDb::add` `modes` is always populated and `targets`
/// and `effect` are empty.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SpellDef {
    pub modes: Vec<Body>,
    pub choose: (u8, u8),
    pub targets: Vec<TargetSpec>,
    pub effect: Option<Effect>,
}

impl Default for SpellDef {
    fn default() -> Self {
        SpellDef { modes: Vec::new(), choose: (1, 1), targets: Vec::new(), effect: None }
    }
}

impl SpellDef {
    pub fn single(body: Body) -> SpellDef {
        SpellDef { modes: vec![body], choose: (1, 1), targets: Vec::new(), effect: None }
    }

    /// Moves an inline single-mode body into `modes`.
    pub fn normalize(&mut self) {
        if self.modes.is_empty() {
            let body = Body { label: String::new(), targets: std::mem::take(&mut self.targets), effect: self.effect.take() };
            self.modes.push(body);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ActivatedDef {
    /// The Oracle sentence of this ability (shown to players, matched by scenario `text_contains`).
    #[serde(default)]
    pub text: String,
    pub cost: Vec<CostItem>,
    #[serde(default)]
    pub targets: Vec<TargetSpec>,
    #[serde(default)]
    pub effect: Option<Effect>,
    /// A mana ability (CR 605): no target, adds mana, does not use the stack.
    #[serde(default)]
    pub is_mana: bool,
    #[serde(default = "timing_instant")]
    pub timing: Timing,
    /// The zone the source must be in to activate (hand for cycling and channel).
    #[serde(default = "zone_bf")]
    pub zone: ZoneKind,
    /// "This ability costs {1} less to activate for each <permanent> you control."
    #[serde(default)]
    pub cost_reduce: Option<ObjFilter>,
}

fn timing_instant() -> Timing {
    Timing::Instant
}

/// Continuous-effect layer (doc 01 section 10.2). Only the layers the pool needs exist.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Layer {
    L4,
    L6,
    L7a,
    L7b,
    L7c,
}

/// How long a resolved spell's continuous effect lasts.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Until {
    EndOfTurn,
    /// Forever (a permanent change, such as a token's size).
    Forever,
    /// Until the effect controller's next turn begins.
    YourNextTurn,
}

/// What kind of second face a card has.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum FaceKind {
    /// A double-faced card: the back face is only ever on the battlefield (transformed).
    Transform,
    /// A modal double-faced card: the back face is played or cast instead of the front.
    Modal,
    /// An Adventure: the back face is an instant or sorcery cast from hand; the card is then
    /// exiled and may be cast as the front face.
    Adventure,
}

/// Effects that apply to a player rather than to objects.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PlayerFx {
    /// Spells the player controls can't be countered.
    SpellsUncounterable,
    /// The player and their permanents have hexproof from the given colors.
    HexproofFrom(Colors),
    /// The player can't cast spells (Orim's Chant).
    CantCast,
    /// Creatures the player controls can't attack (Orim's Chant, kicked: applied to each player).
    CreaturesCantAttack,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ContEffect {
    /// A creature can't attack (Twisted Caverns-style, "until your next turn").
    CantAttack,
    ModifyPT(i16, i16),
    SetPT(i16, i16),
    GrantKeywords(Keywords),
    AddTypes(Types),
    AddSubtypes(SubtypeSet),
    /// Characteristic-defining power and toughness: `x` and `x + t_plus` (layer 7a).
    SetPTExpr { x: Expr, t_plus: i16 },
    HexproofFrom(Colors),
    /// Colors become exactly these.
    SetColors(Colors),
    /// Creature types become exactly this one (other subtypes are kept).
    SetCreatureType(u8),
    /// The land becomes the given basic land type (305.7): it loses its land types and its
    /// abilities, and has the intrinsic mana ability of the type. Layer 4.
    BecomeBasicLand(u8),
    /// The permanent's card types and subtypes become exactly these ("he's a 3/4 Ninja creature"). Layer 4.
    SetTypes(Types, SubtypeSet),
    /// The permanent loses these card types ("isn't a creature"). Layer 4.
    RemoveTypes(Types),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct StaticDef {
    pub layer: Layer,
    pub applies_to: ObjFilter,
    pub effect: ContEffect,
    /// Condition on the source's controller (`You`) for the static to apply (Delirium).
    #[serde(default)]
    pub cond: Option<Cond>,
}

/// What a triggered ability listens for (doc 03 section 3.6).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EventPat {
    /// An object enters the battlefield (the filter is checked against the entering object;
    /// `controller` is relative to the trigger's controller).
    Enters { filter: ObjFilter },
    /// An object moves between zones. `from`/`to` of `None` mean "any". The filter's `zone` is
    /// ignored; characteristics are those the object had in the old zone (last-known).
    ZoneChange { filter: ObjFilter, from: Option<ZoneKind>, to: Option<ZoneKind> },
    /// A spell is cast. `caster` is relative to the trigger's controller.
    /// `nth`: only the caster's n-th spell this turn (flurry).
    SpellCast { caster: Rel, #[serde(default)] filter: ObjFilter, #[serde(default)] nth: Option<u8> },
    /// A player draws a card.
    /// `not_first_in_draw_step`: the first card drawn in each of their draw steps does not count.
    /// `self_only`: only the source card itself being drawn; `first_this_turn`: it is the first card
    /// the player drew this turn (miracle).
    Draw { who: Rel, #[serde(default)] not_first_in_draw_step: bool, #[serde(default)] self_only: bool, #[serde(default)] first_this_turn: bool },
    /// A creature matching the filter deals combat damage to a player (`EventAmount` = the damage).
    CombatDamageToPlayer { filter: ObjFilter },
    /// The beginning of a step of the given player's turn.
    BeginStep { step: Step, whose: Rel },
    /// A player plays a land (not put onto the battlefield by an effect).
    LandPlayed { who: Rel, #[serde(default)] filter: ObjFilter },
    /// A creature matching the filter attacks (declared as an attacker).
    Attacks { filter: ObjFilter },
    /// A creature matching the filter becomes blocked by a creature (`EventObj` = the blocker).
    BecomesBlocked { filter: ObjFilter },
    /// This permanent becomes the target of a spell or ability controlled by a player matching
    /// `by` (ward). `EventObj` = that spell or ability, `EventPlayer` = its controller.
    BecomesTarget { by: Rel },
    /// The controller declares one or more attackers ("whenever you attack").
    YouAttack,
    /// Never matches an event: the body of a delayed trigger created by `Delay`/`ExileUntil`.
    Delayed,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct TriggeredDef {
    /// The Oracle sentence of this ability (shown to players, matched by scenario `text_contains`).
    #[serde(default)]
    pub text: String,
    pub on: EventPat,
    /// Triggers only once each turn (per source).
    #[serde(default)]
    pub once_per_turn: bool,
    /// "One or more": simultaneous matching events (still pending) trigger it only once.
    #[serde(default)]
    pub batch: bool,
    /// The zone the source must be in for this to trigger (battlefield by default; graveyard for
    /// "when this is put into a graveyard"-style cards; stack for "when you cast this spell").
    #[serde(default = "zone_bf")]
    pub zone: ZoneKind,
    /// Intervening "if": checked when the event happens and again on resolution (CR 603.4).
    #[serde(default)]
    pub cond: Option<Cond>,
    /// `cond` belongs to the trigger event itself ("whenever you draw your third card each turn"),
    /// not to an intervening "if": it is checked only when the event happens (CR 603.4 applies
    /// only to an "if" that follows the trigger condition).
    #[serde(default)]
    pub event_cond: bool,
    #[serde(default)]
    pub targets: Vec<TargetSpec>,
    #[serde(default)]
    pub effect: Option<Effect>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum AbilityDef {
    Spell(SpellDef),
    Activated(ActivatedDef),
    Triggered(TriggeredDef),
    Static(StaticDef),
    Alt(AltCost),
    AddCost(AddCostDef),
    /// Escalate: this cost is paid once for each mode chosen beyond the first.
    Escalate(Vec<CostItem>),
    CostMod(CostModDef),
    /// A replacement effect on entering the battlefield.
    Enters(EntersDef),
    /// Replicate (CR 702.56): the cost may be paid any number of times as an additional cost.
    Replicate(ManaCost),
    /// Delve (CR 702.66): cards exiled from the graveyard pay generic mana.
    Delve,
    /// A "can't" restriction on casting spells or activating abilities.
    Restrict(RestrictDef),
    /// A permission to cast spells in a way the rules do not normally allow (Aluren, Omniscience).
    Permit(PermitDef),
    /// "As this enters, choose ...": the choice is made as the permanent spell resolves.
    EntersChoice(ChoiceKind),
    /// Ascend (CR 702.131): with ten or more permanents its controller gets the city's blessing.
    Ascend,
    /// Quantum Riddler: as long as its controller has one or fewer cards in hand, each draw
    /// instruction for them draws one more card.
    DrawPlusOne,
    /// "If you would draw a card while your library has no cards in it, you win the game instead."
    EmptyDrawWins,
    /// "You have no maximum hand size." (on an emblem)
    NoHandLimit,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ChoiceKind {
    CardName,
    CreatureType,
}

/// A static "can't" ability. `who` is relative to the controller of the permanent.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct RestrictDef {
    #[serde(default)]
    pub who: Rel,
    /// Evaluated with the permanent's controller as `You`.
    #[serde(default)]
    pub cond: Option<Cond>,
    pub what: Restriction,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Restriction {
    /// The restricted players can't cast spells matching `applies`.
    CantCast {
        #[serde(default)]
        applies: ObjFilter,
        /// Only spells with mana value greater than the number of lands the caster controls.
        #[serde(default)]
        cmc_over_lands: bool,
        /// Only a spell cast after the caster already cast a noncreature spell this turn.
        #[serde(default)]
        second_noncreature: bool,
        /// Only spells cast from these zones (all zones if empty).
        #[serde(default)]
        from: Vec<ZoneKind>,
    },
    /// Activated abilities of permanents matching `sources` can't be activated.
    CantActivate {
        #[serde(default)]
        sources: ObjFilter,
        #[serde(default)]
        except_mana: bool,
    },
}

/// "You may cast <spells> without paying their mana costs (and with flash)". Casting this way is
/// a virtual way numbered `128 + index of key in CardDb::permit_keys`.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct PermitDef {
    pub key: String,
    #[serde(default)]
    pub who: Rel,
    #[serde(default)]
    pub applies: ObjFilter,
    #[serde(default)]
    pub flash: bool,
}

/// "If X would enter the battlefield ..." and "enters tapped"/"enters with counters". The filter
/// describes the entering object (`self_only` for the card's own); `controller` is relative to
/// the replacement's source and is the entering object's would-be controller.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct EntersDef {
    #[serde(default)]
    pub filter: ObjFilter,
    /// `Some(false)`: only if it wasn't cast; `Some(true)`: only if it was.
    #[serde(default)]
    pub cast: Option<bool>,
    #[serde(default)]
    pub from: Option<ZoneKind>,
    #[serde(default)]
    pub cond: Option<Cond>,
    pub effect: EntersEffect,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EntersEffect {
    Tapped,
    /// Exile it instead.
    Exile,
    /// It can't enter (it stays where it is).
    Prevent,
    Counters(CounterKind, u8),
    /// One counter per instant or sorcery card exiled with delve while casting it.
    CountersDelved(CounterKind),
    /// Registers a delayed trigger (ability index) for the next end step as the permanent enters.
    /// (ability, cast way required: 1 = first alternative cost).
    EndStepAbility(u8, u8),
    /// Enters with `n` counters of the kind only if it was cast with this alternative cost (impending).
    CountersIfWay(CounterKind, u8, u8),
}

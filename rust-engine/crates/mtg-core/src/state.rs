//! `State`: plain, clonable game data (doc 01 section 4). No pointers, no trait objects, no
//! `Rc`/`Arc`; the card DB is held by the owning `Game`/`Cx`, never by `State`.

use crate::card::CardDb;
use crate::decision::*;
use crate::event::Event;
use crate::frame::Frame;
use crate::ids::*;
use crate::ir::{ContEffect, DelayWhen, Layer, PlayerFx, Until};
use crate::mana::ManaPool;
use crate::rng::Pcg64;
use crate::types::*;
use smallvec::SmallVec;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ObjKind {
    Card,
    Token,
    /// An activated or triggered ability on the stack (Stifle needs something to target).
    Ability,
    SpellCopy,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Obj {
    pub kind: ObjKind,
    pub card: Option<CardId>,
    pub def: CardDefId,
    pub owner: Seat,
    pub controller: Seat,
    pub zone: ZoneKind,
    pub gen: u16,
    pub tapped: bool,
    pub phased: bool,
    pub damage: u16,
    /// Dealt damage by a deathtouch source since the last SBA pass (CR 704.5h).
    pub dt_damage: bool,
    pub counters: SmallVec<[(CounterKind, u16); 2]>,
    pub attached_to: Option<ObjRef>,
    pub timestamp: u32,
    /// Turn number at which the current controller gained control (summoning sickness).
    pub entered_turn: u16,
    /// The zone the object came from when it last changed zones (`Gone` for objects that never moved).
    pub prev_zone: ZoneKind,
    pub x_value: u8,
    /// Zone it was cast from (`Gone` = not cast) and the way (alternative cost index) it was cast.
    pub cast_from: ZoneKind,
    pub cast_way: u8,
    /// The name (card def index + 1) or creature type chosen as it entered; 0 = none.
    pub chosen: u16,
}

impl Obj {
    pub fn counter(&self, k: CounterKind) -> u16 {
        self.counters.iter().find(|c| c.0 == k).map(|c| c.1).unwrap_or(0)
    }
}

/// Derived characteristics (after layers). Valid for battlefield objects while the cache is clean.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Chars {
    pub types: Types,
    pub supertypes: Supertypes,
    pub subtypes: SubtypeSet,
    pub colors: Colors,
    pub keywords: Keywords,
    /// Hexproof from these colors.
    pub hexproof_from: Colors,
    pub power: i32,
    pub toughness: i32,
    /// The object has lost all abilities (printed rules text); Magus of the Moon and the like.
    pub lost: bool,
    /// A continuous effect says the creature can't attack.
    pub cant_attack: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StackKind {
    Spell,
    Ability { source: ObjRef, ability: u8 },
}

/// Facts about the triggering event captured when a triggered ability triggers ("that much",
/// "that creature"), plus last-known information of a departed object.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Captured {
    pub amount: i32,
    pub obj: Option<ObjRef>,
    pub player: Option<Seat>,
    pub lki: Option<crate::event::Lki>,
    /// Objects remembered by a delayed trigger (`ExileUntil`).
    pub objs: SmallVec<[ObjRef; 4]>,
}

/// How a spell was cast (CR 601.2): zone, alternative cost, mana spent. Kept on the stack entry
/// for "was cast from", "no mana was spent" and converge effects.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CastInfo {
    pub from: ZoneKind,
    /// 0 = normal cost; i = the i-th alternative cost.
    pub way: u8,
    pub mana_spent: u8,
    /// Bit c set if mana of color c (W U B R G) was spent.
    pub colors_spent: u8,
    /// Instant and sorcery cards exiled with delve while casting it.
    pub delve_is: u8,
    /// Times the replicate cost was paid.
    pub replicate: u8,
    /// Restricted mana (Cavern of Souls) was spent on it: it can't be countered.
    pub uncounterable: bool,
}

impl Default for CastInfo {
    fn default() -> Self {
        CastInfo { from: ZoneKind::Hand, way: 0, mana_spent: 0, colors_spent: 0, delve_is: 0, replicate: 0, uncounterable: false }
    }
}

/// A continuous effect created by a resolved spell or ability, applied in `derive::recompute`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContInst {
    pub ts: u32,
    pub layer: Layer,
    pub objs: SmallVec<[ObjRef; 4]>,
    pub effect: ContEffect,
    pub until: Until,
    /// Who the effect belongs to (for "until your next turn").
    pub controller: Seat,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerFxInst {
    pub ts: u32,
    pub player: Seat,
    pub fx: PlayerFx,
    pub until: Until,
    pub controller: Seat,
}

/// A trigger that has triggered but is not on the stack yet (CR 603.3).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PendingTrigger {
    pub source: ObjRef,
    pub def: CardDefId,
    pub ability: u8,
    pub controller: Seat,
    pub cap: Captured,
}

/// What a scripted random outcome is for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RandKind {
    /// A library shuffle: the whole library, top first.
    Shuffle,
    /// Cards "put on the bottom of the library in a random order": their order, top first.
    BottomOrder,
}

/// Random outcomes a scenario fixes in advance: (player, kind, order top first).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptedRandom {
    pub queue: std::collections::VecDeque<(Seat, RandKind, Vec<CardDefId>)>,
    /// Shuffles the script did not cover (or that disagreed with the library).
    pub violations: u32,
}

/// An exiled card remembered by the permanent that exiled it: the owner and mana value are
/// those the card had when it was exiled.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Link {
    pub src: ObjRef,
    pub owner: Seat,
    pub cmc: u8,
}

/// "Exiled until this leaves the battlefield" (610.3): when `src` leaves, `card` returns to `from`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UntilLink {
    pub src: ObjRef,
    pub card: ObjRef,
    pub from: ZoneKind,
}

/// "Until your next turn, whenever a creature an opponent controls attacks you or a planeswalker
/// you control, ...": a delayed triggered ability (CR 603.7) that fires on each such attack.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttackWatch {
    pub source: ObjRef,
    pub def: CardDefId,
    pub ability: u8,
    pub controller: Seat,
    pub until: crate::ir::Until,
}

/// A delayed triggered ability waiting for a step (CR 603.7).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DelayedTrig {
    pub when: DelayWhen,
    pub source: ObjRef,
    pub def: CardDefId,
    pub ability: u8,
    pub controller: Seat,
    pub cap: Captured,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackEntry {
    pub obj: ObjRef,
    pub controller: Seat,
    pub kind: StackKind,
    pub targets: Targets,
    pub x: u8,
    pub def: CardDefId,
    /// Bitmask of the modes chosen at cast time (bit i = mode i); `1` for non-modal spells.
    pub modes: u16,
    pub cap: Captured,
    pub cast: CastInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PlayerTurn {
    pub spells_cast: u8,
    pub noncreature_cast: u8,
    pub lands_played: u8,
    pub cards_drawn: u8,
    /// Cards drawn during this turn's draw step.
    pub draw_step_draws: u8,
    pub life_lost: u16,
    pub life_gained: u16,
    /// A permanent this player controlled left the battlefield this turn (revolt).
    pub perm_left: bool,
    /// Colors (bitmask of `Colors`) of the spells this player cast this turn.
    pub cast_colors: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerState {
    pub life: i32,
    pub hand: Vec<ObjRef>,
    pub library: Vec<ObjRef>,
    pub graveyard: Vec<ObjRef>,
    pub sideboard: Vec<CardDefId>,
    pub pool: ManaPool,
    /// Restricted mana in the pool: (color index, creature type) usable only for creature spells
    /// of that type, which then can't be countered (Cavern of Souls).
    pub rpool: Vec<(u8, u8)>,
    pub energy: u32,
    /// The city's blessing (ascend).
    pub blessing: bool,
    /// The dungeon card in the command zone (if any) and the room the venture marker is in.
    pub dungeon: Option<ObjRef>,
    pub room: u8,
    /// Dungeons this player has completed.
    pub completed: Vec<CardDefId>,
    /// Emblems this player has (objects in the command zone with static abilities).
    pub emblems: Vec<ObjRef>,
    /// How many times the Ring has tempted this player (0..=4) and their Ring-bearer.
    pub ring: u8,
    pub ring_bearer: Option<ObjRef>,
    pub lost: bool,
    /// Attempted to draw from an empty library (SBA CR 704.5b).
    pub drew_from_empty: bool,
    pub mulligans: u8,
    pub kept: bool,
    pub turn: PlayerTurn,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TurnMode {
    /// Run the turn-based actions of the current step.
    Begin,
    /// Turn-based action frame(s) have finished.
    AfterTba,
    /// Run SBA and trigger placement, then ask for priority.
    Stabilize,
    AskPriority,
    AwaitPriority,
    /// Step is over: move to the next step or turn.
    End,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TurnState {
    pub number: u16,
    pub active: Seat,
    pub step: Step,
    pub priority: Seat,
    pub passes: u8,
    pub mode: TurnMode,
    pub first: Seat,
    /// Turn number of each seat's most recent turn (summoning sickness, CR 302.6).
    pub own_turn: [u16; 2],
    pub cleanup_priority: bool,
    pub stabilized_changed: bool,
    pub extra_turns: SmallVec<[Seat; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttackerInfo {
    pub obj: ObjRef,
    pub target: AttackTarget,
    pub blockers: SmallVec<[ObjRef; 2]>,
    pub blocked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CombatState {
    pub attackers: Vec<AttackerInfo>,
    /// Creatures that dealt first-strike damage (so they do not deal regular damage again unless
    /// they have double strike).
    pub first_struck: Vec<ObjRef>,
    pub damage_step_done: bool,
}

/// Which seats know a card's identity (doc 02 section 6).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CardKnow {
    pub known_to: u8,
    pub pos_known_to: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct GameConfig {
    pub first_player: Seat,
    pub starting_life: i32,
    pub hand_size: u8,
    /// Offer the mana abilities of ordinary sources as explicit actions (scenario tests). Off in
    /// play: payment is automatic and only non-trivial mana abilities are explicit.
    pub explicit_mana: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig { first_player: Seat::P0, starting_life: 20, hand_size: 7, explicit_mana: false }
    }
}

#[derive(Clone)]
pub struct State {
    pub(crate) objs: Vec<Obj>,
    pub(crate) free: Vec<u16>,
    pub(crate) players: [PlayerState; 2],
    pub(crate) battlefield: Vec<ObjRef>,
    pub(crate) stack: Vec<StackEntry>,
    /// Triggers waiting to be put on the stack at the next stabilization point.
    pub(crate) pending_triggers: Vec<PendingTrigger>,
    /// Delayed triggered abilities not yet triggered.
    pub(crate) delayed: Vec<DelayedTrig>,
    /// Delayed abilities that trigger on opposing creatures attacking their controller.
    pub(crate) attack_watch: Vec<AttackWatch>,
    /// Exiled cards linked to the permanent that exiled them (Skyclave Apparition).
    pub(crate) links: Vec<Link>,
    pub(crate) until_links: Vec<UntilLink>,
    /// X of the spell or ability whose targets are being chosen (`ObjFilter::cmc_x`).
    pub(crate) cur_x: Option<u8>,
    /// The player a `ForEachPlayer` body is currently running for (scratch).
    pub(crate) each_player: Option<Seat>,
    /// Scenario runs: the shuffle outcomes the script dictates (None = truly random).
    pub(crate) scripted: Option<ScriptedRandom>,
    /// Abilities (source, ability index) that already did their once-each-turn thing this turn.
    pub(crate) used: Vec<(ObjRef, u8)>,
    /// Exiled cards their owner may play (Runestone Caverns) for as long as they stay exiled.
    pub(crate) exile_plays: Vec<(ObjRef, Seat)>,
    /// Scratch: the objects (new identities) moved by the latest move effect. Not hashed.
    pub(crate) moved: SmallVec<[ObjRef; 4]>,
    /// Scratch: the card name (def id + 1) chosen by the resolving effect. 0 = none.
    pub(crate) name_choice: u16,
    /// Permanents that entered and still owe their "as this enters, choose" decision.
    pub(crate) enter_choices: Vec<ObjRef>,
    /// Simultaneous zone changes in progress (CR 603.10, 614.12): triggers match at the end.
    pub(crate) batch: crate::batch::Batch,
    /// Continuous effects from resolved spells and abilities.
    pub(crate) effects: Vec<ContInst>,
    pub(crate) player_fx: Vec<PlayerFxInst>,
    pub(crate) exile: Vec<ObjRef>,
    pub(crate) turn: TurnState,
    pub(crate) combat: Option<CombatState>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) pending: Option<Pending>,
    pub(crate) next_decision: [u32; 2],
    pub(crate) derived: Vec<Chars>,
    pub(crate) derived_dirty: bool,
    pub(crate) knowledge: Vec<CardKnow>,
    pub(crate) vids: Vec<[ViewId; 2]>,
    pub(crate) next_vid: [u32; 2],
    pub(crate) rng: Pcg64,
    pub(crate) ts_counter: u32,
    pub(crate) result: Option<GameResult>,
    pub(crate) cfg: GameConfig,
    pub(crate) n_cards: u16,
    /// Scratch: drained by the owning `Game` after each step. Not part of hash or equality.
    pub(crate) events: Vec<Event>,
    /// Count of engine steps taken (for budgets); not hashed.
    pub(crate) steps: u64,
    /// Scenario runs only: sources automatic payment should use first (`pay: {tap: [...]}`).
    pub(crate) pay_hint: Vec<ObjRef>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Doc 02 section 3 item 3: no contents unless the diff-harness feature is on.
        #[cfg(feature = "diff-harness")]
        {
            write!(f, "State{{turn:{} step:{:?} life:{:?} ..}}", self.turn.number, self.turn.step, [self.players[0].life, self.players[1].life])
        }
        #[cfg(not(feature = "diff-harness"))]
        {
            write!(f, "State{{<redacted>}}")
        }
    }
}

/// Live objects (cards, tokens, abilities, spells) at which a game is declared a draw; the arena
/// has 65,535 slots and the rest is headroom for the stack while the game winds down.
pub const RUNAWAY_OBJECTS: usize = 60_000;

impl State {
    pub(crate) fn live_objects(&self) -> usize {
        self.objs.len() - self.free.len()
    }

    /// Builds a game. `main[s]` are the 60-card main decks in decklist order (CardIds follow
    /// this order, never library order). Libraries are shuffled by the Pregame frame so RNG
    /// consumption is part of the replayable game flow.
    pub fn new(db: &CardDb, main: [&[CardDefId]; 2], side: [&[CardDefId]; 2], seed: u64, cfg: GameConfig) -> State {
        let mut objs = Vec::new();
        let mut libs: [Vec<ObjRef>; 2] = [Vec::new(), Vec::new()];
        for s in 0..2usize {
            for &d in main[s] {
                let slot = objs.len() as u16;
                assert!(db.defs.len() > d.0 as usize, "bad CardDefId");
                objs.push(Obj {
                    kind: ObjKind::Card,
                    card: Some(CardId(slot)),
                    def: d,
                    owner: Seat(s as u8),
                    controller: Seat(s as u8),
                    zone: ZoneKind::Library,
                    gen: 0,
                    tapped: false,
                    phased: false,
                    damage: 0,
                    dt_damage: false,
                    counters: SmallVec::new(),
                    attached_to: None,
                    timestamp: 0,
                    entered_turn: 0,
                    prev_zone: ZoneKind::Gone,
                    x_value: 0,
                    cast_from: ZoneKind::Gone,
                    cast_way: 0,
                    chosen: 0,
                });
                libs[s].push(ObjRef { slot, gen: 0 });
            }
        }
        let n = objs.len();
        assert!(n < u16::MAX as usize / 2, "too many cards");
        let mk = |s: usize, lib: Vec<ObjRef>| PlayerState {
            life: cfg.starting_life,
            hand: Vec::new(),
            library: lib,
            graveyard: Vec::new(),
            sideboard: side[s].to_vec(),
            pool: ManaPool::default(),
            rpool: Vec::new(),
            energy: 0,
            blessing: false,
            dungeon: None,
            room: 0,
            completed: Vec::new(),
            emblems: Vec::new(),
            ring: 0,
            ring_bearer: None,
            lost: false,
            drew_from_empty: false,
            mulligans: 0,
            kept: false,
            turn: PlayerTurn::default(),
        };
        let [l0, l1] = libs;
        State {
            objs,
            free: Vec::new(),
            players: [mk(0, l0), mk(1, l1)],
            battlefield: Vec::new(),
            stack: Vec::new(),
            pending_triggers: Vec::new(),
            delayed: Vec::new(),
            attack_watch: Vec::new(),
            links: Vec::new(),
            until_links: Vec::new(),
            cur_x: None,
            each_player: None,
            scripted: None,
            used: Vec::new(),
            exile_plays: Vec::new(),
            moved: SmallVec::new(),
            name_choice: 0,
            enter_choices: Vec::new(),
            batch: Default::default(),
            effects: Vec::new(),
            player_fx: Vec::new(),
            exile: Vec::new(),
            turn: TurnState {
                number: 0,
                active: cfg.first_player,
                step: Step::Untap,
                priority: cfg.first_player,
                passes: 0,
                mode: TurnMode::Begin,
                first: cfg.first_player,
                own_turn: [0, 0],
                cleanup_priority: false,
                stabilized_changed: false,
                extra_turns: SmallVec::new(),
            },
            combat: None,
            frames: vec![Frame::Turn, Frame::Pregame(crate::frame::PregameFrame::new())],
            pending: None,
            next_decision: [1, 1],
            derived: vec![Chars::default(); n],
            derived_dirty: true,
            knowledge: vec![CardKnow::default(); n],
            vids: vec![[ViewId::NONE; 2]; n],
            next_vid: [1, 1],
            rng: Pcg64::from_seed(seed),
            ts_counter: 1,
            result: None,
            cfg,
            n_cards: n as u16,
            events: Vec::new(),
            steps: 0,
            pay_hint: Vec::new(),
        }
    }

    // ---- basic accessors ------------------------------------------------------------------

    #[inline]
    pub fn is_live(&self, r: ObjRef) -> bool {
        (r.slot as usize) < self.objs.len() && self.objs[r.slot as usize].gen == r.gen && self.objs[r.slot as usize].zone != ZoneKind::Gone
    }

    #[inline]
    pub(crate) fn obj(&self, r: ObjRef) -> &Obj {
        debug_assert!(self.is_live(r), "stale ObjRef {r:?}");
        &self.objs[r.slot as usize]
    }

    #[inline]
    pub(crate) fn obj_mut(&mut self, r: ObjRef) -> &mut Obj {
        debug_assert!(self.is_live(r), "stale ObjRef {r:?}");
        &mut self.objs[r.slot as usize]
    }

    /// Mutable access ignoring liveness (for objects being set up, e.g. fresh ability objects).
    pub(crate) fn obj_mut_raw(&mut self, r: ObjRef) -> &mut Obj {
        &mut self.objs[r.slot as usize]
    }

    #[inline]
    pub(crate) fn cur_ref(&self, slot: u16) -> ObjRef {
        ObjRef { slot, gen: self.objs[slot as usize].gen }
    }

    pub(crate) fn next_ts(&mut self) -> u32 {
        let t = self.ts_counter;
        self.ts_counter += 1;
        t
    }

    pub(crate) fn player(&self, s: Seat) -> &PlayerState {
        &self.players[s.idx()]
    }

    pub(crate) fn player_mut(&mut self, s: Seat) -> &mut PlayerState {
        &mut self.players[s.idx()]
    }

    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    pub fn turn_number(&self) -> u16 {
        self.turn.number
    }

    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// Allocates a non-card object slot (token, ability object, spell copy).
    pub(crate) fn alloc_slot(&mut self, kind: ObjKind, def: CardDefId, owner: Seat) -> ObjRef {
        debug_assert!(kind != ObjKind::Card);
        let ts = self.next_ts();
        let o = Obj {
            kind,
            card: None,
            def,
            owner,
            controller: owner,
            zone: ZoneKind::Gone,
            gen: 0,
            tapped: false,
            phased: false,
            damage: 0,
            dt_damage: false,
            counters: SmallVec::new(),
            attached_to: None,
            timestamp: ts,
            entered_turn: self.turn.number,
            prev_zone: ZoneKind::Gone,
            x_value: 0,
            cast_from: ZoneKind::Gone,
            cast_way: 0,
            chosen: 0,
        };
        if let Some(slot) = self.free.pop() {
            let gen = self.objs[slot as usize].gen.wrapping_add(1);
            self.objs[slot as usize] = Obj { gen, ..o };
            self.vids[slot as usize] = [ViewId::NONE; 2];
            ObjRef { slot, gen }
        } else {
            // Object references are 16-bit slots: refuse to wrap into a real card's slot. A game
            // that grows this far is a runaway loop; drivers bound games by decisions long before.
            assert!(self.objs.len() < u16::MAX as usize, "object arena exhausted (runaway game)");
            let slot = self.objs.len() as u16;
            self.objs.push(o);
            self.derived.push(Chars::default());
            self.knowledge.push(CardKnow::default());
            self.vids.push([ViewId::NONE; 2]);
            ObjRef { slot, gen: 0 }
        }
    }

    /// Frees a non-card slot. Its generation is bumped so stale references are detected.
    pub(crate) fn free_slot(&mut self, r: ObjRef) {
        let o = &mut self.objs[r.slot as usize];
        debug_assert!(o.kind != ObjKind::Card);
        o.gen = o.gen.wrapping_add(1);
        o.zone = ZoneKind::Gone;
        self.vids[r.slot as usize] = [ViewId::NONE; 2];
        self.free.push(r.slot);
    }
}

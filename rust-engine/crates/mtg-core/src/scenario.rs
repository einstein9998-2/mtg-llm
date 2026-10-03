//! Explicit-state construction for scenario tests (doc 04 section 4.2): build a game directly in a
//! given turn/step with chosen zones, bypassing the pre-game. Cards are real physical cards of
//! the decks implied by the setup, so every invariant (conservation, ids) still holds.

use crate::card::CardDb;
use crate::cx::Cx;
use crate::frame::Frame;
use crate::ids::*;
use crate::ops::MoveOpts;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Debug, Default)]
pub struct PermSetup {
    pub name: String,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub counters: Vec<(CounterKind, u16)>,
    pub damage: u16,
    /// Defaults to the listing player.
    pub controller: Option<Seat>,
    /// Name (card def index + 1) chosen as it entered; 0 = none.
    pub chosen: u16,
}

impl PermSetup {
    pub fn new(name: &str) -> PermSetup {
        PermSetup { name: name.to_string(), ..Default::default() }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlayerSetup {
    pub life: i32,
    pub energy: u32,
    pub hand: Vec<String>,
    pub battlefield: Vec<PermSetup>,
    pub graveyard: Vec<String>,
    pub exile: Vec<String>,
    /// Top first.
    pub library: Vec<String>,
    pub lands_played: u8,
    pub spells_cast: u8,
    /// Mana floating in the pool (W U B R G C).
    pub pool: [u8; 6],
}

#[derive(Clone, Debug)]
pub struct ScenarioSetup {
    pub turn: u16,
    pub active: Seat,
    pub step: Step,
    pub players: [PlayerSetup; 2],
    pub seed: u64,
}

impl State {
    /// Builds a state at `step` of `turn` with the given zones; the first decision is the
    /// priority decision of the active player (turn-based actions of that step are skipped).
    pub fn from_scenario(db: &CardDb, sc: &ScenarioSetup) -> State {
        let id = |n: &str| db.id(n).unwrap_or_else(|| panic!("scenario: unknown card {n}"));
        let mut decks: [Vec<CardDefId>; 2] = [vec![], vec![]];
        // Creation order per seat: hand, battlefield, graveyard, exile, then library bottom-first.
        for s in 0..2 {
            let p = &sc.players[s];
            decks[s].extend(p.hand.iter().map(|n| id(n)));
            decks[s].extend(p.battlefield.iter().map(|b| id(&b.name)));
            decks[s].extend(p.graveyard.iter().map(|n| id(n)));
            decks[s].extend(p.exile.iter().map(|n| id(n)));
            decks[s].extend(p.library.iter().rev().map(|n| id(n)));
        }
        let cfg = GameConfig { first_player: sc.active, ..GameConfig::default() };
        let mut st = State::new(db, [&decks[0], &decks[1]], [&[], &[]], sc.seed, cfg);
        st.frames = vec![Frame::Turn];
        st.turn.number = sc.turn;
        st.turn.active = sc.active;
        st.turn.priority = sc.active;
        st.turn.step = sc.step;
        st.turn.mode = TurnMode::Stabilize;
        st.turn.own_turn = [0, 0];
        st.turn.own_turn[sc.active.idx()] = sc.turn;
        st.turn.own_turn[sc.active.other().idx()] = sc.turn.saturating_sub(1);
        if sc.step.is_combat() && sc.step != Step::BeginCombat {
            st.combat = Some(CombatState::default());
        } else if sc.step == Step::BeginCombat {
            st.combat = Some(CombatState::default());
        }
        for p in 0..2 {
            st.players[p].life = sc.players[p].life;
            st.players[p].energy = sc.players[p].energy;
            st.players[p].kept = true;
            st.players[p].turn.lands_played = sc.players[p].lands_played;
            st.players[p].turn.spells_cast = sc.players[p].spells_cast;
            st.players[p].pool = crate::mana::ManaPool(sc.players[p].pool);
        }
        {
            let mut cx = Cx::new(&mut st, db);
            for s in 0..2usize {
                let seat = Seat(s as u8);
                let p = &sc.players[s];
                let mut order: Vec<(ZoneKind, usize)> = Vec::new();
                for i in 0..p.hand.len() {
                    order.push((ZoneKind::Hand, i));
                }
                for i in 0..p.battlefield.len() {
                    order.push((ZoneKind::Battlefield, i));
                }
                for i in 0..p.graveyard.len() {
                    order.push((ZoneKind::Graveyard, i));
                }
                for i in 0..p.exile.len() {
                    order.push((ZoneKind::Exile, i));
                }
                // Creation order == placement order, so entry k is physical slot base+k.
                let base: u16 = if s == 0 { 0 } else { decks[0].len() as u16 };
                for (k, (zone, i)) in order.iter().enumerate() {
                    let r = cx.s.cur_ref(base + k as u16);
                    let nr = match zone {
                        ZoneKind::Battlefield => {
                            let b = &p.battlefield[*i];
                            let ctrl = b.controller.unwrap_or(seat);
                            // A back-face name sets the permanent up already transformed.
                            let cur = cx.s.obj(r).def;
                            let face = db.def(cur).is_back;
                            if face {
                                cx.s.obj_mut_raw(r).def = db.def(cur).front_id.expect("back face has a front");
                            }
                            let nr = cx.commit_move(r, ZoneKind::Battlefield, MoveOpts { controller: Some(ctrl), tapped: b.tapped, face, ..Default::default() }, None).unwrap();
                            let o = cx.s.obj_mut(nr);
                            o.damage = b.damage;
                            o.chosen = b.chosen;
                            o.entered_turn = if b.summoning_sick { sc.turn } else { 0 };
                            for &(k2, n) in &b.counters {
                                o.counters.push((k2, n));
                            }
                            nr
                        }
                        z => cx.commit_move(r, *z, MoveOpts::default(), None).unwrap(),
                    };
                    let _ = nr;
                }
            }
            cx.s.derived_dirty = true;
        }
        st.events.clear();
        st.enter_choices.clear();
        st.pending_triggers.clear(); // setup is not play: nothing triggers from placing the board
        // Fresh view-id numbering is irrelevant; keep the allocation made during placement.
        st
    }
}

/// Creation order of the physical cards of a scenario: slot k holds the k-th entry. Per seat:
/// hand, battlefield, graveyard, exile, then the library bottom-first.
pub fn creation_order(sc: &ScenarioSetup) -> Vec<(Seat, ZoneKind, usize)> {
    let mut v = Vec::new();
    for s in 0..2usize {
        let p = &sc.players[s];
        let seat = Seat(s as u8);
        v.extend((0..p.hand.len()).map(|i| (seat, ZoneKind::Hand, i)));
        v.extend((0..p.battlefield.len()).map(|i| (seat, ZoneKind::Battlefield, i)));
        v.extend((0..p.graveyard.len()).map(|i| (seat, ZoneKind::Graveyard, i)));
        v.extend((0..p.exile.len()).map(|i| (seat, ZoneKind::Exile, i)));
        v.extend((0..p.library.len()).rev().map(|i| (seat, ZoneKind::Library, i)));
    }
    v
}

impl State {
    /// Scenario setup helper: puts a token onto the battlefield (no triggers).
    pub fn scenario_token(&mut self, db: &CardDb, def: CardDefId, seat: Seat, sick: bool) -> ObjRef {
        let mut cx = Cx::new(self, db);
        let r = cx.create_token_with(def, seat, false).expect("setup token enters");
        if !sick {
            cx.s.obj_mut(r).entered_turn = 0;
        }
        cx.s.events.clear();
        cx.s.pending_triggers.clear();
        r
    }

    /// Scenario setup helper: `seat` has `def` in the command zone with the marker in `room`.
    pub fn scenario_dungeon(&mut self, db: &CardDb, def: CardDefId, seat: Seat, room: u8) -> ObjRef {
        let mut cx = Cx::new(self, db);
        cx.start_dungeon(seat, def);
        cx.s.pending_triggers.clear();
        cx.s.players[seat.idx()].room = room;
        cx.s.players[seat.idx()].dungeon.expect("dungeon started")
    }

    /// Scenario setup helper: puts ability `ability` of `src` on the stack with the given targets.
    pub fn scenario_push_ability(&mut self, db: &CardDb, src: ObjRef, ability: u8, controller: Seat, targets: Vec<crate::decision::Target>) {
        let def = self.obj(src).def;
        let mut cx = Cx::new(self, db);
        cx.push_ability(src, ability, def, controller, targets.into_iter().collect(), Default::default());
    }

    /// Scenario setup helper: from now on shuffles follow the script (library orders, top first).
    pub fn scenario_script_random(&mut self, queue: Vec<(Seat, RandKind, Vec<CardDefId>)>) {
        self.scripted = Some(ScriptedRandom { queue: queue.into(), violations: 0 });
    }

    /// (shuffles the script did not cover, scripted shuffles that never happened).
    pub fn scenario_random_report(&self) -> (u32, usize) {
        self.scripted.as_ref().map(|s| (s.violations, s.queue.len())).unwrap_or((0, 0))
    }

    /// Scenario setup helper: `seat` has already completed `def`.
    pub fn scenario_completed(&mut self, seat: Seat, def: CardDefId) {
        self.players[seat.idx()].completed.push(def);
    }

    /// Scenario setup helper: attaches `a` to `to` (Equipment to creature, Aura to permanent).
    pub fn scenario_attach(&mut self, a: ObjRef, to: ObjRef) {
        self.obj_mut(a).attached_to = Some(to);
        self.derived_dirty = true;
    }
}

impl State {
    /// Offer ordinary mana abilities as explicit actions (the spec scenarios activate them).
    pub fn scenario_explicit_mana(&mut self) {
        self.cfg.explicit_mana = true;
    }

    /// Prefer these sources when automatic payment picks what to tap (empty clears the hint).
    pub fn scenario_pay_hint(&mut self, srcs: Vec<ObjRef>) {
        self.pay_hint = srcs;
    }

    pub fn scenario_set_priority(&mut self, seat: Seat) {
        self.turn.priority = seat;
    }
}

impl State {
    pub fn scenario_set_tapped(&mut self, r: ObjRef, tapped: bool) {
        self.obj_mut(r).tapped = tapped;
    }

    pub fn scenario_set_counters_damage(&mut self, r: ObjRef, counters: &[(CounterKind, u16)], damage: u16) {
        let o = self.obj_mut(r);
        for &(k, n) in counters {
            o.counters.push((k, n));
        }
        o.damage = damage;
        self.derived_dirty = true;
    }
}

//! `Game`: owns the engine state and card DB.

use mtg_core::card::CardDb;
use mtg_core::cx::Cx;
use mtg_core::decision::*;
use mtg_core::engine;
use mtg_core::ids::*;
use mtg_core::state::{GameConfig, State};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DeckList {
    pub main: Vec<CardDefId>,
    pub side: Vec<CardDefId>,
}

#[derive(Clone)]
pub struct Game {
    state: State,
    db: Arc<CardDb>,
    keep_events: bool,
    events: Vec<mtg_core::event::Event>,
    /// Redacted per-seat event logs and the index each seat has "acted past" (doc 02 section 2.3).
    ev_log: [Vec<crate::view::ViewEvent>; 2],
    ev_mark: [usize; 2],
    track_views: bool,
}

impl Game {
    pub fn new(db: Arc<CardDb>, decks: [&DeckList; 2], seed: u64, cfg: GameConfig) -> Game {
        let state = State::new(&db, [&decks[0].main, &decks[1].main], [&decks[0].side, &decks[1].side], seed, cfg);
        Game { state, db, keep_events: false, events: Vec::new(), ev_log: [Vec::new(), Vec::new()], ev_mark: [0, 0], track_views: true }
    }

    /// Builds a game directly in a given state (scenario tests). Harness use only.
    #[cfg(feature = "diff-harness")]
    pub fn from_scenario(db: Arc<CardDb>, sc: &mtg_core::scenario::ScenarioSetup) -> Game {
        let state = State::from_scenario(&db, sc);
        Game { state, db, keep_events: false, events: Vec::new(), ev_log: [Vec::new(), Vec::new()], ev_mark: [0, 0], track_views: true }
    }

    /// Runs the rules machinery to the next decision or game end.
    pub fn advance(&mut self) -> Status {
        let mut cx = Cx::new(&mut self.state, &self.db);
        let st = engine::advance(&mut cx);
        self.drain_events();
        st
    }

    /// Core-internal pending decision (raw object slots, either seat). Agents never see this:
    /// they get decisions built in their own view-id space through a `SeatView`.
    pub(crate) fn pending_raw(&self) -> Option<&Pending> {
        self.state.pending()
    }

    /// Raw pending decision. Harness builds only (`diff-harness`).
    #[cfg(feature = "diff-harness")]
    pub fn pending(&self) -> Option<&Pending> {
        self.pending_raw()
    }

    pub fn apply(&mut self, id: DecisionId, idx: usize) -> Result<(), ApplyError> {
        // The seat has now seen everything logged so far, but only if the action is accepted: a
        // rejected apply (stale id, bad index) must not discard its unseen events.
        let mark = self.state.pending().map(|p| (p.seat, self.ev_log[p.seat.idx()].len()));
        let mut cx = Cx::new(&mut self.state, &self.db);
        let r = engine::apply(&mut cx, id, idx);
        if r.is_ok() {
            if let Some((seat, n)) = mark {
                self.ev_mark[seat.idx()] = n;
            }
        }
        self.drain_events();
        r
    }

    fn drain_events(&mut self) {
        let ev = self.state.take_events();
        for e in ev.iter().filter(|_| self.track_views) {
            for seat in 0..2u8 {
                if let Some(v) = crate::project::redact_with_state(&self.state, &self.db, e, Seat(seat)) {
                    self.ev_log[seat as usize].push(v);
                }
            }
        }
        if self.keep_events {
            self.events.extend(ev);
        }
    }

    pub(crate) fn view_events_for(&self, seat: Seat) -> Vec<crate::view::ViewEvent> {
        self.ev_log[seat.idx()][self.ev_mark[seat.idx()]..].to_vec()
    }

    pub(crate) fn determinize_in_place(&mut self, seat: Seat, asg: &mtg_core::fork::HiddenAssignment, seed: u64) -> Result<(), mtg_core::fork::ForkError> {
        self.state.determinize(&self.db, seat, asg, seed)?;
        self.events.clear();
        self.ev_log[seat.other().idx()].clear();
        self.ev_mark[seat.other().idx()] = 0;
        Ok(())
    }

    pub(crate) fn core_state(&self) -> &State {
        &self.state
    }

    /// What `seat` may see (doc 02 section 2.1). Any-seat access is a harness privilege
    /// (`diff-harness`); an agent is handed a `SeatView`, which is bound to one seat.
    #[cfg(feature = "diff-harness")]
    pub fn observe(&self, seat: Seat) -> crate::view::Observation {
        crate::project::observe(self, seat)
    }

    /// Agent-facing pending decision for `seat`, if it is the decider. Harness builds only.
    #[cfg(feature = "diff-harness")]
    pub fn decision_for(&self, seat: Seat) -> Option<crate::view::Decision> {
        crate::project::observe(self, seat).decision
    }

    /// The agent's handle on this game: bound to `seat`, nothing of the other seat or the engine
    /// is reachable through it.
    pub fn seat_view(&self, seat: Seat) -> SeatView<'_> {
        SeatView { g: self, seat }
    }

    /// Per-seat redacted event logs cost memory and time; throughput harnesses can turn them off
    /// (observations then carry empty `events`).
    pub fn set_track_view_events(&mut self, on: bool) {
        self.track_views = on;
    }

    /// Harness option: retain core events for `take_events` (default: dropped each step).
    #[cfg(feature = "diff-harness")]
    pub fn set_keep_events(&mut self, keep: bool) {
        self.keep_events = keep;
    }

    #[cfg(feature = "diff-harness")]
    pub fn take_events(&mut self) -> Vec<mtg_core::event::Event> {
        std::mem::take(&mut self.events)
    }

    pub fn result(&self) -> Option<GameResult> {
        self.state.result()
    }

    /// Full-state hash for replay checkpoints and fork-consistency checks. Harness use only: it
    /// covers hidden state, so it must never be handed to an agent.
    #[cfg(feature = "diff-harness")]
    pub fn state_hash(&self) -> u64 {
        self.state.hash_full()
    }

    pub(crate) fn hash_raw(&self) -> u64 {
        self.state.hash_full()
    }

    /// Agent-facing error text. Never mentions game objects.
    pub fn describe_error(&self, e: ApplyError) -> String {
        let base = match e {
            ApplyError::StaleDecision => "stale decision id",
            ApplyError::BadIndex => "option index out of range",
            ApplyError::GameOver => "game is over",
            ApplyError::NoDecision => "no decision pending",
        };
        if mtg_core::canary::on(mtg_core::canary::ERROR_NAMES_CARD) {
            // Deliberate leak: names the top card of the opponent's library.
            #[cfg(feature = "diff-harness")]
            if let Some(&top) = self.state.library(Seat(1)).first() {
                return format!("{base} (while {} is on top)", self.db.def(self.state.def_of(top)).name);
            }
        }
        base.to_string()
    }

    pub fn db(&self) -> &Arc<CardDb> {
        &self.db
    }

    /// Names the permanents `seat` wants to pay with (601.2h), by that seat's own view ids: the
    /// automatic payment prefers them over everything else and falls back to the rest if they
    /// cannot pay. Ids that are not permanents on the battlefield are ignored; an empty list
    /// clears the preference. Lasts until the caller clears it.
    pub fn set_pay_hint(&mut self, seat: Seat, vids: &[ViewId]) {
        let refs: Vec<ObjRef> = if vids.is_empty() {
            Vec::new()
        } else {
            self.state.battlefield().iter().copied().filter(|&r| vids.contains(&self.state.view_id(r, seat))).collect()
        };
        self.state.scenario_pay_hint(refs);
    }

    #[cfg(feature = "diff-harness")]
    pub fn raw_state(&self) -> &State {
        &self.state
    }

    #[cfg(feature = "diff-harness")]
    pub fn raw_state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

/// A game as one seat may see it. This is the only thing a search or learning agent holds: it can
/// observe its own view, read its own pending decision and fork worlds consistent with what it
/// knows, but it cannot name the other seat, engine objects, hashes or raw events.
#[derive(Clone, Copy)]
pub struct SeatView<'a> {
    g: &'a Game,
    seat: Seat,
}

impl<'a> SeatView<'a> {
    pub fn seat(&self) -> Seat {
        self.seat
    }

    pub fn observe(&self) -> crate::view::Observation {
        crate::project::observe(self.g, self.seat)
    }

    /// The decision this seat has to make now, if any.
    pub fn decision(&self) -> Option<crate::view::Decision> {
        crate::project::observe(self.g, self.seat).decision
    }

    /// A runnable world consistent with what this seat knows (see `Game::fork`).
    pub fn fork(&self, seed: u64, model: &dyn crate::fork::BeliefModel) -> Result<Game, mtg_core::fork::ForkError> {
        self.g.fork(self.seat, seed, model)
    }
}

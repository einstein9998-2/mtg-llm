//! A batched self-play environment: N games stepped in lockstep, one observation per game that is
//! waiting for a decision, the client (a trainer) answers with option indices. Both seats are
//! controlled by the client, so this serves self-play directly. Trivial decisions (one option) are
//! applied internally.
//!
//! Each observation is exactly what the acting seat may see (`SeatView::observe`), encoded with
//! `mtg_view::encode_state` / `encode_options`; the environment never exposes the other seat.

use mtg_core::card::CardDb;
use mtg_core::decision::{GameResult, Status};
use mtg_core::ids::{DecisionId, Seat};
use mtg_core::state::GameConfig;
use mtg_view::{encode_options, encode_state, DeckList, Game, OptionFeat};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct EnvConfig {
    pub n_games: usize,
    pub seed0: u64,
    /// Deck pairs (indices into the deck list) to rotate through; empty means all ordered pairs.
    pub pairs: Vec<(usize, usize)>,
    /// Start a fresh game in a slot as soon as its game ends.
    pub autoreset: bool,
    /// A game with more non-trivial decisions than this is cut off as a draw.
    pub max_decisions: u32,
}

impl Default for EnvConfig {
    fn default() -> EnvConfig {
        EnvConfig { n_games: 16, seed0: 1, pairs: Vec::new(), autoreset: true, max_decisions: 4000 }
    }
}

/// One decision waiting for an answer.
#[derive(Clone, Debug)]
pub struct Obs {
    pub game: usize,
    pub seat: Seat,
    pub id: DecisionId,
    /// Nonzero entries of the state encoding (index, value).
    pub state: Vec<(u32, f32)>,
    pub options: Vec<OptionFeat>,
}

/// A finished game.
#[derive(Clone, Debug)]
pub struct Done {
    pub game: usize,
    /// +1 if seat 0 won, -1 if seat 1 won, 0 for a draw or a cut-off game.
    pub result: i8,
    pub decks: (usize, usize),
    pub first: u8,
    pub decisions: u32,
    pub truncated: bool,
}

#[derive(Clone, Debug)]
pub enum Out {
    Obs(Obs),
    Done(Done),
}

struct Slot {
    game: Game,
    decks: (usize, usize),
    first: u8,
    decisions: u32,
    /// (seat, decision id, option count) of the decision waiting for an answer.
    waiting: Option<(Seat, DecisionId, usize)>,
    finished: bool,
}

pub struct BatchEnv {
    db: Arc<CardDb>,
    decks: Vec<DeckList>,
    cfg: EnvConfig,
    slots: Vec<Slot>,
    started: u64,
    n_defs: usize,
    buf: Vec<f32>,
}

impl BatchEnv {
    pub fn new(db: Arc<CardDb>, decks: Vec<DeckList>, mut cfg: EnvConfig) -> BatchEnv {
        if cfg.pairs.is_empty() {
            for a in 0..decks.len() {
                for b in 0..decks.len() {
                    cfg.pairs.push((a, b));
                }
            }
        }
        let n_defs = db.defs.len();
        BatchEnv { db, decks, cfg, slots: Vec::new(), started: 0, n_defs, buf: Vec::new() }
    }

    pub fn n_defs(&self) -> usize {
        self.n_defs
    }

    fn new_game(&mut self) -> Slot {
        let k = self.started;
        self.started += 1;
        let (a, b) = self.cfg.pairs[(k as usize) % self.cfg.pairs.len()];
        let first = (k % 2) as u8;
        let game = Game::new(self.db.clone(), [&self.decks[a], &self.decks[b]], self.cfg.seed0 + k, GameConfig { first_player: Seat(first), ..GameConfig::default() });
        Slot { game, decks: (a, b), first, decisions: 0, waiting: None, finished: false }
    }

    /// Creates the games and returns their first observations.
    pub fn start(&mut self) -> Vec<Out> {
        self.slots.clear();
        self.started = 0;
        for _ in 0..self.cfg.n_games {
            let s = self.new_game();
            self.slots.push(s);
        }
        let mut out = Vec::new();
        for i in 0..self.slots.len() {
            self.advance_slot(i, &mut out);
        }
        out
    }

    /// Applies `(game, option)` answers and returns the next observations (and finished games).
    /// Nothing is applied if any answer is invalid.
    pub fn step(&mut self, actions: &[(usize, usize)]) -> Result<Vec<Out>, String> {
        let mut seen = std::collections::BTreeSet::new();
        for &(g, a) in actions {
            let s = self.slots.get(g).ok_or_else(|| format!("no game {g}"))?;
            let (_, _, n) = s.waiting.ok_or_else(|| format!("game {g} is not waiting for a decision"))?;
            if a >= n {
                return Err(format!("game {g}: option {a} out of range (0..{n})"));
            }
            if !seen.insert(g) {
                return Err(format!("game {g} answered twice"));
            }
        }
        let mut out = Vec::new();
        for &(g, a) in actions {
            let (_, id, _) = self.slots[g].waiting.take().unwrap();
            let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.slots[g].game.apply(id, a)));
            match applied {
                Ok(r) => r.map_err(|e| format!("game {g}: {e:?}"))?,
                Err(_) => {
                    self.drop_panicked(g, &mut out);
                    continue;
                }
            }
            self.advance_slot(g, &mut out);
        }
        Ok(out)
    }

    /// Games currently waiting for an answer.
    pub fn waiting(&self) -> Vec<usize> {
        self.slots.iter().enumerate().filter(|(_, s)| s.waiting.is_some()).map(|(i, _)| i).collect()
    }

    /// An engine panic (a runaway game) ends only that game: it is reported as a truncated draw
    /// and the slot restarts, so a long training run is not lost.
    fn drop_panicked(&mut self, i: usize, out: &mut Vec<Out>) {
        let s = &self.slots[i];
        eprintln!("envserver: game {i} (decks {} vs {}) panicked after {} decisions and was dropped", s.decks.0, s.decks.1, s.decisions);
        out.push(Out::Done(Done { game: i, result: 0, decks: s.decks, first: s.first, decisions: s.decisions, truncated: true }));
        if self.cfg.autoreset {
            self.slots[i] = self.new_game();
            self.advance_slot(i, out);
        } else {
            self.slots[i].waiting = None;
            self.slots[i].finished = true;
        }
    }

    fn advance_slot(&mut self, i: usize, out: &mut Vec<Out>) {
        loop {
            let adv = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.slots[i].game.advance()));
            let Ok(st) = adv else {
                self.drop_panicked(i, out);
                return;
            };
            match st {
                Status::GameOver(r) => {
                    let s = &self.slots[i];
                    let result = match r {
                        GameResult::Win(w) if w == Seat(0) => 1,
                        GameResult::Win(_) => -1,
                        GameResult::Draw => 0,
                    };
                    out.push(Out::Done(Done { game: i, result, decks: s.decks, first: s.first, decisions: s.decisions, truncated: false }));
                    if self.cfg.autoreset {
                        self.slots[i] = self.new_game();
                        continue;
                    }
                    self.slots[i].finished = true;
                    return;
                }
                Status::NeedDecision(seat) => {
                    let o = self.slots[i].game.seat_view(seat).observe();
                    let d = o.decision.clone().expect("decision pending");
                    let n = d.options.len();
                    if n == 1 {
                        self.slots[i].game.apply(d.id, 0).expect("forced option");
                        continue;
                    }
                    self.slots[i].decisions += 1;
                    if self.slots[i].decisions > self.cfg.max_decisions {
                        let s = &self.slots[i];
                        out.push(Out::Done(Done { game: i, result: 0, decks: s.decks, first: s.first, decisions: s.decisions, truncated: true }));
                        if self.cfg.autoreset {
                            self.slots[i] = self.new_game();
                            continue;
                        }
                        self.slots[i].finished = true;
                        return;
                    }
                    encode_state(&o, self.n_defs, &mut self.buf);
                    let state: Vec<(u32, f32)> = self.buf.iter().enumerate().filter(|(_, v)| **v != 0.0).map(|(k, v)| (k as u32, *v)).collect();
                    let options = encode_options(&o);
                    self.slots[i].waiting = Some((seat, d.id, n));
                    out.push(Out::Obs(Obs { game: i, seat, id: d.id, state, options }));
                    return;
                }
            }
        }
    }
}

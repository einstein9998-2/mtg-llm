//! The match loop: best of three with sideboarding between games.

use crate::model::ExpectedModel;
use crate::plan::{board, PlanBook};
use mtg_core::card::CardDb;
use mtg_core::decision::GameResult;
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::state::GameConfig;
use mtg_view::{playout, BeliefModel, DeckList, Game, Policy};
use std::sync::Arc;

/// What a player believes about the opponent's 60 cards in games after the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OppView {
    /// The opponent's game-one main deck. Cards from their sideboard are surprises when first seen.
    Main,
    /// The opponent's main deck with the standard plan for this matchup applied: the prepared-player
    /// assumption (the plans are fixed public knowledge, not something read from the opponent's
    /// hidden choices). Equal to `Main` for a seat that does not board.
    Plan,
}

#[derive(Clone, Debug)]
pub struct MatchConfig {
    /// Per seat: sideboard from game two on (needs a plan; no plan means no change).
    pub board: [bool; 2],
    pub opp_view: OppView,
    /// Who plays first in game one; later games the loser of the previous game plays first.
    pub first: Seat,
    /// A match is over when a seat has two wins; drawn games (a runaway state or the decision cap)
    /// are replayed up to this many games in total.
    pub max_games: u32,
    /// Decision cap per game; a game that reaches it is a draw.
    pub max_decisions: u32,
    pub seed: u64,
}

impl Default for MatchConfig {
    fn default() -> Self {
        MatchConfig { board: [true, true], opp_view: OppView::Plan, first: Seat(0), max_games: 5, max_decisions: 6000, seed: 1 }
    }
}

#[derive(Clone, Debug)]
pub struct GameLog {
    pub first: Seat,
    pub boarded: [bool; 2],
    /// `None`: drawn or truncated.
    pub winner: Option<Seat>,
    pub panicked: bool,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub games: Vec<GameLog>,
    pub wins: [u32; 2],
}

impl MatchResult {
    /// The match winner, if a seat reached two wins.
    pub fn winner(&self) -> Option<Seat> {
        (0..2u8).find(|&s| self.wins[s as usize] >= 2).map(Seat)
    }
}

/// Builds the policy of one seat for one game. `model` is that seat's belief model for the game
/// and lives as long as the policy.
pub trait PlayerFactory {
    fn make<'m>(&mut self, seat: Seat, game_no: u32, model: &'m dyn BeliefModel, seed: u64) -> Box<dyn Policy + 'm>;
}

/// The two decklists played in game `game_no` (0-based) and which seats actually boarded.
pub fn game_decks(db: &CardDb, names: [&str; 2], base: [&DeckList; 2], book: Option<&PlanBook>, cfg: &MatchConfig, game_no: u32) -> Result<([DeckList; 2], [bool; 2]), String> {
    let mut decks = [base[0].clone(), base[1].clone()];
    let mut boarded = [false; 2];
    if game_no > 0 && book.is_some() {
        for s in 0..2 {
            if !cfg.board[s] {
                continue;
            }
            if let Some(plan) = book.unwrap().plan(names[s], names[1 - s]) {
                decks[s] = board(db, base[s], plan).map_err(|e| format!("{} vs {}: {e}", names[s], names[1 - s]))?;
                boarded[s] = true;
            }
        }
    }
    Ok((decks, boarded))
}

/// Plays one match between `base[0]` (seat 0) and `base[1]` (seat 1).
pub fn play_match(db: &Arc<CardDb>, names: [&str; 2], base: [&DeckList; 2], book: Option<&PlanBook>, cfg: &MatchConfig, fac: &mut dyn PlayerFactory) -> Result<MatchResult, String> {
    let mut res = MatchResult { games: Vec::new(), wins: [0, 0] };
    let mut first = cfg.first;
    let mut game_no = 0u32;
    while res.wins[0] < 2 && res.wins[1] < 2 && game_no < cfg.max_games {
        let (decks, boarded) = game_decks(db, names, base, book, cfg, game_no)?;
        // What each seat believes about the other's deck.
        let expected = |s: usize| -> Vec<CardDefId> {
            let opp = 1 - s;
            match cfg.opp_view {
                OppView::Plan => decks[opp].main.clone(),
                OppView::Main => base[opp].main.clone(),
            }
        };
        let models = [0usize, 1].map(|s| {
            let mut d: [Vec<CardDefId>; 2] = [Vec::new(), Vec::new()];
            d[s] = decks[s].main.clone();
            d[1 - s] = expected(s);
            ExpectedModel { decks: d, exact: [s == 0, s == 1] }
        });
        let seed = cfg.seed.wrapping_mul(1_000_003).wrapping_add(game_no as u64);
        let mut pols = [fac.make(Seat(0), game_no, &models[0], seed ^ 0x51), fac.make(Seat(1), game_no, &models[1], seed ^ 0xA7)];
        let mut g = Game::new(db.clone(), [&decks[0], &decks[1]], seed, GameConfig { first_player: first, ..GameConfig::default() });
        let [p0, p1] = &mut pols;
        // A runaway engine state (a panic) ends the game as a draw instead of the process.
        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            playout(&mut g, &mut [&mut **p0, &mut **p1], cfg.max_decisions)
        }));
        let (result, panicked) = match out {
            Ok(r) => (r, false),
            Err(_) => (None, true),
        };
        let winner = match result {
            Some(GameResult::Win(s)) => Some(s),
            _ => None,
        };
        res.games.push(GameLog { first, boarded, winner, panicked });
        if let Some(w) = winner {
            res.wins[w.idx()] += 1;
            first = w.other(); // the loser chooses to play first
        }
        game_no += 1;
    }
    Ok(res)
}

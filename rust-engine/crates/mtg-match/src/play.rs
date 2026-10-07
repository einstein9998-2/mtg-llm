//! The match loop: best of three with sideboarding between games.

use crate::model::ExpectedModel;
use crate::plan::{board, PlanBook};
use mtg_core::card::CardDb;
use mtg_core::decision::GameResult;
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::state::GameConfig;
use mtg_core::decision::Status;
use mtg_view::{BeliefModel, DeckList, Game, Policy};
use std::sync::Arc;

/// What a player believes about the opponent's 60 cards in games after the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OppView {
    /// The opponent's game-one main deck. Cards from their sideboard are surprises when first seen.
    Main,
    /// The opponent's base 75 with the *public* plan for this matchup applied (the `public` books
    /// given to `play_match`): the prepared-player assumption. The plans are fixed public knowledge;
    /// the opponent's actual list is never read, so a variant under test is not leaked to the
    /// searching seat. Equal to `Main` for a seat that does not board.
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
    /// Everything needed to replay the game bit for bit (harness use): the engine seed, the two
    /// 60-card lists as played, and the (decision id, option index) of every action.
    pub seed: u64,
    pub decks: [DeckList; 2],
    pub actions: Vec<(u32, u32)>,
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
pub fn game_decks(db: &CardDb, names: [&str; 2], base: [&DeckList; 2], books: [Option<&PlanBook>; 2], cfg: &MatchConfig, game_no: u32, first: Seat) -> Result<([DeckList; 2], [bool; 2]), String> {
    let mut decks = [base[0].clone(), base[1].clone()];
    let mut boarded = [false; 2];
    if game_no > 0 {
        for s in 0..2 {
            let Some(book) = books[s] else { continue };
            if !cfg.board[s] {
                continue;
            }
            if let Some(plan) = book.plan_for(names[s], names[1 - s], Some(first.idx() == s)) {
                decks[s] = board(db, base[s], plan).map_err(|e| format!("{} vs {}: {e}", names[s], names[1 - s]))?;
                boarded[s] = true;
            }
        }
    }
    Ok((decks, boarded))
}

/// Each seat's belief model for a game: its own list exactly, the opponent's list as expected. The
/// expectation comes from the public plan (`pub_decks`) or the opponent's base main deck, never from
/// `decks`, the lists actually played, which would leak what a plan under test brought in.
pub fn beliefs(base: [&DeckList; 2], decks: &[DeckList; 2], pub_decks: &[DeckList; 2], view: OppView) -> [ExpectedModel; 2] {
    [0usize, 1].map(|s| {
        let opp = 1 - s;
        let mut d: [Vec<CardDefId>; 2] = [Vec::new(), Vec::new()];
        d[s] = decks[s].main.clone();
        d[opp] = match view {
            OppView::Plan => pub_decks[opp].main.clone(),
            OppView::Main => base[opp].main.clone(),
        };
        ExpectedModel { decks: d, exact: [s == 0, s == 1] }
    })
}

/// Plays one match between `base[0]` (seat 0) and `base[1]` (seat 1). Each seat boards from its own
/// plan book (the same book for both seats, or two books to play one plan against another).
/// `public` is each seat's *standard* plan book, the one the other seat is assumed to know under
/// `OppView::Plan`. It is separate from `books` so that a plan under test never reaches the other
/// seat's belief; pass the same books as `books` when no plan is under test.
pub fn play_match(db: &Arc<CardDb>, names: [&str; 2], base: [&DeckList; 2], books: [Option<&PlanBook>; 2], public: [Option<&PlanBook>; 2], cfg: &MatchConfig, fac: &mut dyn PlayerFactory) -> Result<MatchResult, String> {
    let mut res = MatchResult { games: Vec::new(), wins: [0, 0] };
    let mut first = cfg.first;
    let mut game_no = 0u32;
    while res.wins[0] < 2 && res.wins[1] < 2 && game_no < cfg.max_games {
        let (decks, boarded) = game_decks(db, names, base, books, cfg, game_no, first)?;
        let (pub_decks, _) = game_decks(db, names, base, public, cfg, game_no, first)?;
        let models = beliefs(base, &decks, &pub_decks, cfg.opp_view);
        let seed = cfg.seed.wrapping_mul(1_000_003).wrapping_add(game_no as u64);
        let mut pols = [fac.make(Seat(0), game_no, &models[0], seed ^ 0x51), fac.make(Seat(1), game_no, &models[1], seed ^ 0xA7)];
        let mut g = Game::new(db.clone(), [&decks[0], &decks[1]], seed, GameConfig { first_player: first, ..GameConfig::default() });
        let [p0, p1] = &mut pols;
        let mut actions: Vec<(u32, u32)> = Vec::new();
        // A runaway engine state (a panic) ends the game as a draw instead of the process.
        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..cfg.max_decisions {
                match g.advance() {
                    Status::GameOver(r) => return Some(r),
                    Status::NeedDecision(seat) => {
                        let sv = g.seat_view(seat);
                        let d = sv.decision().expect("decision pending");
                        let k = d.options.len();
                        let i = if seat.0 == 0 { p0.choose(&sv, k) } else { p1.choose(&sv, k) };
                        actions.push((d.id.0, i as u32));
                        g.apply(d.id, i).expect("policy chose an in-range index");
                    }
                }
            }
            match g.advance() {
                Status::GameOver(r) => Some(r),
                _ => None,
            }
        }));
        let (result, panicked) = match out {
            Ok(r) => (r, false),
            Err(_) => (None, true),
        };
        let winner = match result {
            Some(GameResult::Win(s)) => Some(s),
            _ => None,
        };
        res.games.push(GameLog { first, boarded, seed, decks: decks.clone(), actions, winner, panicked });
        if let Some(w) = winner {
            res.wins[w.idx()] += 1;
            first = w.other(); // the loser chooses to play first
        }
        game_no += 1;
    }
    Ok(res)
}

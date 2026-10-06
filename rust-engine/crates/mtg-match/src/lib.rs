//! Best-of-three matches with between-game sideboarding, layered above the frozen core.
//!
//! The core plays single games: `Game::new` takes a `DeckList` (main + side) per seat. A match is
//! therefore a loop that builds a new decklist per game and starts a fresh game, so sideboarding
//! needs no engine change (and no RFC).
//!
//! * `plan`: sideboard plan files (`sideboard-plans/<deck>.txt`), validation and application.
//! * `model`: a belief model for games where the opponent may have sideboarded.
//! * `play`: the match loop (who plays first, which seat boards, how an agent is plugged in).

pub mod model;
pub mod plan;
pub mod play;

pub use model::*;
pub use plan::*;
pub use play::*;

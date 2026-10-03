//! Phase 3 hooks: a determinized tree search with a pluggable evaluator, and a batched
//! environment a trainer can drive. Nothing here sees hidden state: it only holds `SeatView`s and
//! forks worlds consistent with what the acting seat knows.

pub mod decks;
pub mod env;
pub mod mcts;
pub mod net;

pub use decks::*;
pub use env::*;
pub use mcts::*;
pub use net::*;

//! Belief model for games after sideboarding.
//!
//! `UniformConsistentModel` needs each seat's decklist exactly: the cards the observer cannot
//! identify must be the decklist minus the identified ones, and the counts must match. That is
//! right for the observer's own deck and for the opponent in game 1. After sideboarding the
//! observer knows the opponent's 75 but not the 60 they chose, so the opponent's list is only an
//! *expectation*: `ExpectedModel` takes a list of the same size and, whenever the observer has
//! seen a card that is not in it (a surprise from the sideboard), lets that card replace one
//! uniformly random not-yet-seen card of the expected list. With no surprises it is exactly the
//! uniform model.

use mtg_core::fork::{ForkError, HiddenAssignment, HiddenRequest};
use mtg_core::ids::CardDefId;
use mtg_core::rng::Pcg64;
use mtg_view::BeliefModel;

pub struct ExpectedModel {
    /// Per seat: the list the observer believes that seat plays (its exact list for the observer
    /// itself, an expectation for the opponent).
    pub decks: [Vec<CardDefId>; 2],
    /// Per seat: the list is exact, so an unknown card is an error instead of a surprise.
    pub exact: [bool; 2],
}

impl BeliefModel for ExpectedModel {
    fn sample(&self, req: &HiddenRequest, rng: &mut Pcg64) -> Result<HiddenAssignment, ForkError> {
        let mut out: [Vec<CardDefId>; 2] = [Vec::new(), Vec::new()];
        for p in 0..2 {
            let mut rest = self.decks[p].clone();
            let mut surprises = 0usize;
            for k in &req.known[p] {
                match rest.iter().position(|d| d == k) {
                    Some(i) => {
                        rest.swap_remove(i);
                    }
                    None if !self.exact[p] => surprises += 1,
                    None => return Err(ForkError::Inconsistent(format!("seat {p}: a known card is not in the decklist"))),
                }
            }
            // The decklist size is fixed: each surprise displaces one unseen expected card.
            if rest.len() < surprises + req.unknown[p] {
                // The expected list is too short for what is hidden.
                return Err(ForkError::Inconsistent(format!("seat {p}: expected list leaves {} unseen cards for {} hidden", rest.len().saturating_sub(surprises), req.unknown[p])));
            }
            rest.sort();
            rng.shuffle(&mut rest);
            rest.truncate(rest.len() - surprises);
            if rest.len() != req.unknown[p] {
                return Err(ForkError::Inconsistent(format!("seat {p}: expected list leaves {} unseen cards but {} are hidden", rest.len(), req.unknown[p])));
            }
            rng.shuffle(&mut rest);
            out[p] = rest;
        }
        Ok(HiddenAssignment { defs: out })
    }
}

//! Belief model for games after sideboarding.
//!
//! `UniformConsistentModel` needs each seat's decklist exactly: the cards the observer cannot
//! identify must be the decklist minus the identified ones, and the counts must match. That is
//! right for the observer's own deck and for the opponent in game 1. After sideboarding the
//! observer knows the opponent's 75 but not the 60 they chose, so the opponent's list is only an
//! *expectation*: `ExpectedModel` takes a list of the same size and, whenever the observer has
//! seen a card that is not in it (a surprise from the sideboard), lets that card replace one
//! uniformly random not-yet-seen card of the expected list: the unseen cards are the expected list
//! minus the identified ones, cut down to the number actually hidden (so a card fetched from the
//! sideboard mid-game, like Burning Wish does, does not make the fork fail). With no surprises it is
//! exactly the uniform model.

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
    /// Per seat: the sideboard the observer believes that seat holds. Cards the sideboard can put
    /// into a game (Burning Wish) are hidden objects beyond the 60 of the list; when more cards are
    /// hidden than the list leaves unseen, the extra ones are drawn from here. May be empty.
    pub side: [Vec<CardDefId>; 2],
}

impl BeliefModel for ExpectedModel {
    fn sample(&self, req: &HiddenRequest, rng: &mut Pcg64) -> Result<HiddenAssignment, ForkError> {
        let mut out: [Vec<CardDefId>; 2] = [Vec::new(), Vec::new()];
        for p in 0..2 {
            let mut rest = self.decks[p].clone();
            for k in &req.known[p] {
                // A known card outside the list is a surprise (a card boarded in) or an extra card
                // from the sideboard (Burning Wish): the owner then holds more than 60 objects, so
                // the unseen part is whatever the hidden count leaves, not "list minus surprises".
                if let Some(i) = rest.iter().position(|d| d == k) {
                    rest.swap_remove(i);
                }
            }
            let mut padded = false;
            if rest.len() < req.unknown[p] {
                padded = true;
                // More cards are hidden than the list leaves: sideboard cards fetched into a hand.
                let mut pool = self.side[p].clone();
                pool.sort();
                rng.shuffle(&mut pool);
                let need = req.unknown[p] - rest.len();
                rest.extend(pool.into_iter().take(need));
            }
            if rest.len() < req.unknown[p] {
                return Err(ForkError::Inconsistent(format!("seat {p}: expected list leaves {} unseen cards for {} hidden", rest.len(), req.unknown[p])));
            }
            if self.exact[p] && !padded && rest.len() != req.unknown[p] {
                return Err(ForkError::Inconsistent(format!("seat {p}: decklist leaves {} unseen cards but {} are hidden", rest.len(), req.unknown[p])));
            }
            rest.sort();
            rng.shuffle(&mut rest);
            rest.truncate(req.unknown[p]);
            rng.shuffle(&mut rest);
            out[p] = rest;
        }
        Ok(HiddenAssignment { defs: out })
    }
}

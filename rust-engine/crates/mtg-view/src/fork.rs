//! Forking and determinization (doc 02 section 5.3): a runnable copy of the game in which
//! everything the observer is not entitled to know has been replaced by a sample consistent with
//! what it does know. The true hidden state is never read by sampling code: a `BeliefModel` sees
//! only a `HiddenRequest` (counts and the identities the observer already knows).

use crate::game::Game;
use mtg_core::fork::{ForkError, HiddenAssignment, HiddenRequest};
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::rng::Pcg64;

pub trait BeliefModel {
    fn sample(&self, req: &HiddenRequest, rng: &mut Pcg64) -> Result<HiddenAssignment, ForkError>;
}

/// Uniform over worlds consistent with the observer's knowledge: each player's unseen cards are
/// their decklist minus every card the observer can identify, dealt out in random order.
pub struct UniformConsistentModel {
    pub decks: [Vec<CardDefId>; 2],
}

impl BeliefModel for UniformConsistentModel {
    fn sample(&self, req: &HiddenRequest, rng: &mut Pcg64) -> Result<HiddenAssignment, ForkError> {
        let mut out: [Vec<CardDefId>; 2] = [Vec::new(), Vec::new()];
        for p in 0..2 {
            let mut rest = self.decks[p].clone();
            for k in &req.known[p] {
                match rest.iter().position(|d| d == k) {
                    Some(i) => {
                        rest.swap_remove(i);
                    }
                    None => return Err(ForkError::Inconsistent(format!("seat {p}: a known card is not in the decklist"))),
                }
            }
            if rest.len() != req.unknown[p] {
                return Err(ForkError::Inconsistent(format!("seat {p}: decklist leaves {} unseen cards but {} are hidden", rest.len(), req.unknown[p])));
            }
            // Sort first so the result depends only on the multiset, not on list order or the
            // swap_remove above.
            rest.sort();
            rng.shuffle(&mut rest);
            out[p] = rest;
        }
        Ok(HiddenAssignment { defs: out })
    }
}

impl Game {
    /// A complete, runnable game as `seat` could imagine it (at `seat`'s own decision point, and only there): hidden cards sampled by `model`,
    /// unknown library positions shuffled, RNG reseeded from `seed`. Event logs of the other
    /// seat and retained raw events are dropped.
    pub fn fork(&self, seat: Seat, seed: u64, model: &dyn BeliefModel) -> Result<Game, ForkError> {
        if self.pending_raw().map_or(true, |p| p.seat != seat) {
            return Err(ForkError::NotObserversDecision);
        }
        let mut g = self.clone();
        let req = g.core_state().hidden_request(g.db(), seat);
        let mut rng = Pcg64::from_seed(seed ^ 0x3C6E_F372_FE94_F82B);
        let asg = model.sample(&req, &mut rng)?;
        g.determinize_in_place(seat, &asg, seed)?;
        Ok(g)
    }
}

//! Policies, playouts and flat Monte Carlo search over forks (doc 02 section 5.3, step 5).
//!
//! Nothing here reads hidden state: a search agent forks the game for its own seat (so every
//! world it simulates is consistent with what it knows) and plays the fork out with explicit
//! policies for both seats; the true game is only ever stepped by the caller.

use crate::fork::BeliefModel;
use crate::game::{Game, SeatView};
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_core::decision::GameResult;

/// Chooses an index into the pending decision's options. Called only when a decision is pending.
pub trait Policy {
    /// `v` is the acting seat's own view of the game.
    fn choose(&mut self, v: &SeatView, n_options: usize) -> usize;
}

/// Uniform over the legal options (the baseline opponent model and rollout policy).
pub struct RandomPolicy(pub Pcg64);

impl RandomPolicy {
    pub fn new(seed: u64) -> RandomPolicy {
        RandomPolicy(Pcg64::from_seed(seed))
    }
}

impl Policy for RandomPolicy {
    fn choose(&mut self, _v: &SeatView, n: usize) -> usize {
        self.0.below(n as u64) as usize
    }
}

/// Always takes option 0, which is Pass at priority and the first canonical option elsewhere.
pub struct FirstOption;

impl Policy for FirstOption {
    fn choose(&mut self, _v: &SeatView, _n: usize) -> usize {
        0
    }
}

/// Plays `g` to the end (or `max_decisions` more decisions) with one policy per seat.
pub fn playout(g: &mut Game, pol: &mut [&mut dyn Policy; 2], max_decisions: u32) -> Option<GameResult> {
    for _ in 0..max_decisions {
        match g.advance() {
            Status::GameOver(r) => return Some(r),
            Status::NeedDecision(_) => {
                let (id, seat, n) = {
                    let p = g.pending_raw().expect("decision pending");
                    (p.id, p.seat, p.options.len())
                };
                let i = pol[seat.idx()].choose(&g.seat_view(seat), n);
                g.apply(id, i).expect("policy chose an in-range index");
            }
        }
    }
    match g.advance() {
        Status::GameOver(r) => Some(r),
        _ => None,
    }
}

/// Score of a result for `seat`: 1 win, 0.5 draw, 0 loss.
pub fn score(r: Option<GameResult>, seat: Seat) -> f64 {
    match r {
        Some(GameResult::Win(s)) if s == seat => 1.0,
        Some(GameResult::Win(_)) => 0.0,
        _ => 0.5,
    }
}

/// Flat Monte Carlo (determinized): for every option of the pending decision, fork `samples`
/// worlds, take that option and play both seats at random to the end; pick the best mean score.
pub struct FlatMc<'m> {
    pub model: &'m dyn BeliefModel,
    pub samples: u32,
    pub max_rollout: u32,
    pub seed: u64,
    counter: u64,
}

impl<'m> FlatMc<'m> {
    pub fn new(model: &'m dyn BeliefModel, samples: u32, max_rollout: u32, seed: u64) -> FlatMc<'m> {
        FlatMc { model, samples, max_rollout, seed, counter: 0 }
    }

    /// Mean score per option for the seat to act.
    pub fn evaluate(&mut self, v: &SeatView) -> Vec<f64> {
        let d = v.decision().expect("decision pending");
        let (id, seat, n) = (d.id, v.seat(), d.options.len());
        let mut tot = vec![0.0; n];
        for _ in 0..self.samples {
            self.counter += 1;
            let world = match v.fork(self.seed ^ self.counter.wrapping_mul(0x9E37_79B9_7F4A_7C15), self.model) {
                Ok(w) => w,
                Err(_) => return vec![0.0; n],
            };
            for (i, t) in tot.iter_mut().enumerate() {
                let mut w = world.clone();
                // The fork and the real game offer the same options (non-interference), so index i
                // names the same action in both.
                if w.apply(id, i).is_err() {
                    continue;
                }
                let mut a = RandomPolicy::new(self.counter * 977 + i as u64);
                let mut b = RandomPolicy::new(self.counter * 983 + i as u64 + 1);
                *t += score(playout(&mut w, &mut [&mut a, &mut b], self.max_rollout), seat);
            }
        }
        tot.iter().map(|t| t / self.samples as f64).collect()
    }
}

impl Policy for FlatMc<'_> {
    fn choose(&mut self, sv: &SeatView, n: usize) -> usize {
        if n == 1 {
            return 0;
        }
        let v = self.evaluate(sv);
        let mut best = 0;
        for i in 1..n {
            if v[i] > v[best] {
                best = i;
            }
        }
        best
    }
}

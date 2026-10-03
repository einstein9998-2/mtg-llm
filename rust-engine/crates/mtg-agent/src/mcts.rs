//! Single-observer information-set MCTS (determinized): every iteration forks one world that is
//! consistent with what the acting seat knows (`SeatView::fork`), descends a shared tree keyed by
//! option indices, and ends in a leaf scored by an `Evaluator` (random rollouts now, a policy/value
//! network later). The tree is shared across worlds, so statistics average over the belief.
//!
//! Option indices name the same action in every fork of the acting seat's own decisions (the
//! non-interference property). The opponent's decisions inside a fork can have different option
//! counts in different worlds (a conditional decision); a node whose seat or option count does not
//! match is treated as a leaf, never descended.

use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::rng::Pcg64;
use mtg_view::{playout, score, BeliefModel, Game, RandomPolicy, SeatView};

/// What an evaluator says about a world at a decision.
#[derive(Clone, Debug)]
pub struct Eval {
    /// Prior over the `n` options of the decision (sums to about 1).
    pub priors: Vec<f32>,
    /// Expected result for the seat to act, in [-1, 1].
    pub value: f32,
}

pub trait Evaluator {
    /// `world` is a (forked) game with a pending decision of `seat` that has `n_options` options.
    fn eval(&mut self, world: &Game, seat: Seat, n_options: usize) -> Eval;
}

/// Uniform priors and a random-playout value: the no-network baseline.
pub struct RolloutEvaluator {
    pub max_rollout: u32,
    counter: u64,
    seed: u64,
}

impl RolloutEvaluator {
    pub fn new(max_rollout: u32, seed: u64) -> RolloutEvaluator {
        RolloutEvaluator { max_rollout, counter: 0, seed }
    }
}

impl Evaluator for RolloutEvaluator {
    fn eval(&mut self, world: &Game, seat: Seat, n: usize) -> Eval {
        self.counter += 1;
        let mut w = world.clone();
        let mut a = RandomPolicy::new(self.seed ^ self.counter.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut b = RandomPolicy::new(self.seed ^ self.counter.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ 1);
        let r = playout(&mut w, &mut [&mut a, &mut b], self.max_rollout);
        Eval { priors: vec![1.0 / n as f32; n], value: (score(r, seat) * 2.0 - 1.0) as f32 }
    }
}

#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub iterations: u32,
    pub c_puct: f32,
    /// Decisions (not engine steps) descended before a leaf is forced.
    pub max_depth: u32,
    /// Mix of random noise into the root priors (exploration for self-play); 0 disables it.
    pub root_noise: f32,
    pub noise_alpha: f32,
    pub seed: u64,
}

impl Default for SearchConfig {
    fn default() -> SearchConfig {
        SearchConfig { iterations: 64, c_puct: 1.4, max_depth: 120, root_noise: 0.0, noise_alpha: 0.3, seed: 1 }
    }
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    /// Visit count per option of the root decision (the policy target).
    pub visits: Vec<u32>,
    /// Mean value per option for the acting seat.
    pub q: Vec<f32>,
    /// Mean value of the root for the acting seat.
    pub value: f32,
    pub iterations: u32,
    /// Forks that failed (should be 0).
    pub fork_errors: u32,
}

impl SearchResult {
    pub fn best(&self) -> usize {
        let mut b = 0;
        for i in 1..self.visits.len() {
            if self.visits[i] > self.visits[b] || (self.visits[i] == self.visits[b] && self.q[i] > self.q[b]) {
                b = i;
            }
        }
        b
    }

    /// Visit distribution, optionally sharpened or flattened by `temperature` (1 = proportional).
    pub fn policy(&self, temperature: f32) -> Vec<f32> {
        let t = temperature.max(1e-3);
        let w: Vec<f32> = self.visits.iter().map(|&v| (v as f32).powf(1.0 / t)).collect();
        let s: f32 = w.iter().sum::<f32>().max(1e-9);
        w.into_iter().map(|x| x / s).collect()
    }
}

struct Node {
    seat: Seat,
    n: usize,
    prior: Vec<f32>,
    visits: Vec<u32>,
    wsum: Vec<f32>,
    children: Vec<Option<usize>>,
}

impl Node {
    fn new(seat: Seat, n: usize, e: &Eval) -> Node {
        Node { seat, n, prior: e.priors.clone(), visits: vec![0; n], wsum: vec![0.0; n], children: vec![None; n] }
    }

    fn select(&self, c: f32) -> usize {
        let total: u32 = self.visits.iter().sum();
        let mean = if total > 0 { self.wsum.iter().sum::<f32>() / total as f32 } else { 0.0 };
        let sq = (total.max(1) as f32).sqrt();
        let mut best = (f32::NEG_INFINITY, 0);
        for a in 0..self.n {
            let q = if self.visits[a] > 0 { self.wsum[a] / self.visits[a] as f32 } else { mean };
            let u = c * self.prior[a] * sq / (1.0 + self.visits[a] as f32);
            if q + u > best.0 {
                best = (q + u, a);
            }
        }
        best.1
    }
}

fn unit(rng: &mut Pcg64) -> f32 {
    ((rng.next_u64() >> 40) as f32 + 0.5) / (1u64 << 24) as f32
}

/// Rough Dirichlet(alpha) noise (gamma via the small-alpha power trick; exploration only).
fn noise(n: usize, alpha: f32, rng: &mut Pcg64) -> Vec<f32> {
    let g: Vec<f32> = (0..n).map(|_| unit(rng).powf(1.0 / alpha)).collect();
    let s: f32 = g.iter().sum::<f32>().max(1e-12);
    g.into_iter().map(|x| x / s).collect()
}

/// Searches from `v`'s pending decision and returns visit statistics. `model` describes what the
/// acting seat believes about the opponent's hidden cards.
pub fn search(v: &SeatView, model: &dyn BeliefModel, ev: &mut dyn Evaluator, cfg: &SearchConfig) -> SearchResult {
    let d = v.decision().expect("search needs a pending decision");
    let (root_seat, n) = (v.seat(), d.options.len());
    let mut rng = Pcg64::from_seed(cfg.seed);
    if n == 1 {
        return SearchResult { visits: vec![1], q: vec![0.0], value: 0.0, iterations: 0, fork_errors: 0 };
    }
    let mut nodes: Vec<Node> = Vec::new();
    let mut fork_errors = 0;
    for it in 0..cfg.iterations {
        let mut world = match v.fork(cfg.seed ^ (it as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15), model) {
            Ok(w) => w,
            Err(_) => {
                fork_errors += 1;
                continue;
            }
        };
        if nodes.is_empty() {
            world.advance();
            let mut e = ev.eval(&world, root_seat, n);
            if cfg.root_noise > 0.0 {
                let z = noise(n, cfg.noise_alpha, &mut rng);
                for (p, z) in e.priors.iter_mut().zip(z) {
                    *p = (1.0 - cfg.root_noise) * *p + cfg.root_noise * z;
                }
            }
            nodes.push(Node::new(root_seat, n, &e));
        }
        let mut path: Vec<(usize, usize)> = Vec::new();
        let mut cur = Some(0usize);
        let mut depth = 0;
        // (value for `seat`, seat) of the leaf, or a terminal result.
        enum Leaf {
            Value(f32, Seat),
            Terminal(mtg_core::decision::GameResult),
        }
        let leaf = loop {
            match world.advance() {
                Status::GameOver(r) => break Leaf::Terminal(r),
                Status::NeedDecision(seat) => {
                    let d = world.seat_view(seat).decision().expect("decision pending");
                    let n = d.options.len();
                    if n == 1 {
                        world.apply(d.id, 0).expect("forced option");
                        continue;
                    }
                    depth += 1;
                    let usable = cur.filter(|&ni| nodes[ni].seat == seat && nodes[ni].n == n);
                    match (cur, usable) {
                        (_, Some(ni)) if depth <= cfg.max_depth => {
                            let a = nodes[ni].select(cfg.c_puct);
                            path.push((ni, a));
                            cur = nodes[ni].children[a];
                            world.apply(d.id, a).expect("legal option");
                        }
                        (None, _) => {
                            let e = ev.eval(&world, seat, n);
                            let value = e.value;
                            let id = nodes.len();
                            nodes.push(Node::new(seat, n, &e));
                            if let Some(&(pn, pa)) = path.last() {
                                nodes[pn].children[pa] = Some(id);
                            }
                            break Leaf::Value(value, seat);
                        }
                        _ => {
                            // Seat or option count differs from the tree (a conditional decision in
                            // this world) or the depth limit: score here without expanding.
                            let e = ev.eval(&world, seat, n);
                            break Leaf::Value(e.value, seat);
                        }
                    }
                }
            }
        };
        for &(ni, a) in &path {
            let s = nodes[ni].seat;
            let val = match &leaf {
                Leaf::Value(v, ls) => {
                    if *ls == s {
                        *v
                    } else {
                        -*v
                    }
                }
                Leaf::Terminal(r) => (score(Some(*r), s) * 2.0 - 1.0) as f32,
            };
            nodes[ni].visits[a] += 1;
            nodes[ni].wsum[a] += val;
        }
    }
    let root = &nodes[0];
    let total: u32 = root.visits.iter().sum();
    let q: Vec<f32> = (0..n).map(|a| if root.visits[a] > 0 { root.wsum[a] / root.visits[a] as f32 } else { 0.0 }).collect();
    let value = if total > 0 { root.wsum.iter().sum::<f32>() / total as f32 } else { 0.0 };
    SearchResult { visits: root.visits.clone(), q, value, iterations: total, fork_errors }
}

/// A `Policy` that searches at every decision with more than one option.
pub struct MctsPolicy<'m> {
    pub model: &'m dyn BeliefModel,
    pub ev: Box<dyn Evaluator + 'm>,
    pub cfg: SearchConfig,
    counter: u64,
}

impl<'m> MctsPolicy<'m> {
    pub fn new(model: &'m dyn BeliefModel, ev: Box<dyn Evaluator + 'm>, cfg: SearchConfig) -> MctsPolicy<'m> {
        MctsPolicy { model, ev, cfg, counter: 0 }
    }
}

impl mtg_view::Policy for MctsPolicy<'_> {
    fn choose(&mut self, v: &SeatView, n: usize) -> usize {
        if n == 1 {
            return 0;
        }
        self.counter += 1;
        let mut cfg = self.cfg.clone();
        cfg.seed = cfg.seed.wrapping_add(self.counter.wrapping_mul(0xA24B_AED4_963E_E407));
        search(v, self.model, &mut *self.ev, &cfg).best()
    }
}

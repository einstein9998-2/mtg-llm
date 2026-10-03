//! A small policy/value network evaluated on the CPU inside the search, loaded from a weights file
//! a trainer exports (`python/mtg_net.py`). The trainer runs on the GPU; the search only needs the
//! forward pass, which for a sparse-input MLP of this size costs microseconds.
//!
//! Architecture (v1), `x` = sparse state encoding (`mtg_view::encode_state`), all weights f32:
//!   h1 = relu(b1 + sum_i scale[i] * x[i] * W1[i])        W1: state_len x hidden
//!   h2 = relu(b2 + h1 W2)                                 W2: hidden x hidden
//!   value = tanh(bv + wv . h2)
//!   q = bp + h2 Wp                                        Wp: hidden x emb
//!   o_j = E_kind[kind] + E_dec[dec] + E_def[def] + E_zone[zone] + w_val * ln(1 + value_j)
//!   logit_j = q . o_j / sqrt(emb);   priors = softmax(logit)
//!
//! File: 8 bytes "MTGNET01", then u32 little-endian state_len, hidden, emb, n_kind, n_dec, n_def,
//! n_zone, then the tensors as f32 little-endian row-major in the order listed in `Net::TENSORS`.

use crate::mcts::{Eval, Evaluator};
use mtg_core::ids::Seat;
use mtg_view::{encode_options, encode_state, Game, OptionFeat};

pub const MAGIC: &[u8; 8] = b"MTGNET01";

#[derive(Clone, Debug)]
pub struct Net {
    pub state_len: usize,
    pub hidden: usize,
    pub emb: usize,
    pub n_kind: usize,
    pub n_dec: usize,
    pub n_def: usize,
    pub n_zone: usize,
    scale: Vec<f32>,
    w1: Vec<f32>,
    b1: Vec<f32>,
    w2: Vec<f32>,
    b2: Vec<f32>,
    wv: Vec<f32>,
    bv: f32,
    wp: Vec<f32>,
    bp: Vec<f32>,
    e_kind: Vec<f32>,
    e_dec: Vec<f32>,
    e_def: Vec<f32>,
    e_zone: Vec<f32>,
    w_val: Vec<f32>,
}

impl Net {
    /// Tensor names in file order (the Python exporter uses the same list).
    pub const TENSORS: [&'static str; 14] = ["scale", "w1", "b1", "w2", "b2", "wv", "bv", "wp", "bp", "e_kind", "e_dec", "e_def", "e_zone", "w_val"];

    pub fn sizes(state_len: usize, hidden: usize, emb: usize, n_kind: usize, n_dec: usize, n_def: usize, n_zone: usize) -> [usize; 14] {
        [state_len, state_len * hidden, hidden, hidden * hidden, hidden, hidden, 1, hidden * emb, emb, n_kind * emb, n_dec * emb, n_def * emb, n_zone * emb, emb]
    }

    /// A deterministic random net (tests and pipeline checks; not trained).
    pub fn random(state_len: usize, hidden: usize, emb: usize, n_kind: usize, n_dec: usize, n_def: usize, n_zone: usize, seed: u64) -> Net {
        let mut rng = mtg_core::rng::Pcg64::from_seed(seed);
        let mut gen = |n: usize, s: f32| -> Vec<f32> { (0..n).map(|_| ((rng.next_u64() >> 40) as f32 / (1u64 << 24) as f32 - 0.5) * 2.0 * s).collect() };
        let z = Net::sizes(state_len, hidden, emb, n_kind, n_dec, n_def, n_zone);
        Net {
            state_len, hidden, emb, n_kind, n_dec, n_def, n_zone,
            scale: vec![1.0; z[0]],
            w1: gen(z[1], 0.3), b1: gen(z[2], 0.1), w2: gen(z[3], 0.3), b2: gen(z[4], 0.1), wv: gen(z[5], 0.3), bv: 0.0,
            wp: gen(z[7], 0.3), bp: gen(z[8], 0.1), e_kind: gen(z[9], 0.5), e_dec: gen(z[10], 0.5), e_def: gen(z[11], 0.5), e_zone: gen(z[12], 0.5), w_val: gen(z[13], 0.2),
        }
    }

    pub fn load(path: &std::path::Path) -> Result<Net, String> {
        Net::from_bytes(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
    }

    pub fn from_bytes(b: &[u8]) -> Result<Net, String> {
        if b.len() < 36 || &b[..8] != MAGIC {
            return Err("not an MTGNET01 file".into());
        }
        let u = |i: usize| u32::from_le_bytes(b[8 + 4 * i..12 + 4 * i].try_into().unwrap()) as usize;
        let (state_len, hidden, emb, n_kind, n_dec, n_def, n_zone) = (u(0), u(1), u(2), u(3), u(4), u(5), u(6));
        let sizes = Net::sizes(state_len, hidden, emb, n_kind, n_dec, n_def, n_zone);
        let total: usize = sizes.iter().sum();
        if b.len() != 36 + 4 * total {
            return Err(format!("file has {} bytes, header implies {}", b.len(), 36 + 4 * total));
        }
        let mut off = 36;
        let mut t: Vec<Vec<f32>> = Vec::new();
        for &n in &sizes {
            t.push(b[off..off + 4 * n].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect());
            off += 4 * n;
        }
        if t.iter().flatten().any(|x| !x.is_finite()) {
            return Err("weights contain NaN or infinity".into());
        }
        let mut it = t.into_iter();
        let mut next = || it.next().unwrap();
        let (scale, w1, b1, w2, b2, wv, bv, wp, bp, e_kind, e_dec, e_def, e_zone, w_val) = (next(), next(), next(), next(), next(), next(), next(), next(), next(), next(), next(), next(), next(), next());
        Ok(Net { state_len, hidden, emb, n_kind, n_dec, n_def, n_zone, scale, w1, b1, w2, b2, wv, bv: bv[0], wp, bp, e_kind, e_dec, e_def, e_zone, w_val })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        for v in [self.state_len, self.hidden, self.emb, self.n_kind, self.n_dec, self.n_def, self.n_zone] {
            out.extend((v as u32).to_le_bytes());
        }
        let bv = [self.bv];
        for t in [&self.scale[..], &self.w1, &self.b1, &self.w2, &self.b2, &self.wv, &bv[..], &self.wp, &self.bp, &self.e_kind, &self.e_dec, &self.e_def, &self.e_zone, &self.w_val] {
            for x in t {
                out.extend(x.to_le_bytes());
            }
        }
        out
    }

    /// Forward pass on a sparse state and option features: (value in [-1,1], priors).
    pub fn forward(&self, state: &[(u32, f32)], options: &[OptionFeat]) -> (f32, Vec<f32>) {
        let h = self.hidden;
        let mut h1 = self.b1.clone();
        for &(i, v) in state {
            let i = i as usize;
            if i >= self.state_len {
                continue;
            }
            let s = self.scale[i] * v;
            for (a, w) in h1.iter_mut().zip(&self.w1[i * h..(i + 1) * h]) {
                *a += s * w;
            }
        }
        for a in &mut h1 {
            *a = a.max(0.0);
        }
        let mut h2 = self.b2.clone();
        for (k, &x) in h1.iter().enumerate() {
            if x != 0.0 {
                for (a, w) in h2.iter_mut().zip(&self.w2[k * h..(k + 1) * h]) {
                    *a += x * w;
                }
            }
        }
        for a in &mut h2 {
            *a = a.max(0.0);
        }
        let value = (self.bv + h2.iter().zip(&self.wv).map(|(a, b)| a * b).sum::<f32>()).tanh();
        let e = self.emb;
        let mut q = self.bp.clone();
        for (k, &x) in h2.iter().enumerate() {
            if x != 0.0 {
                for (a, w) in q.iter_mut().zip(&self.wp[k * e..(k + 1) * e]) {
                    *a += x * w;
                }
            }
        }
        let norm = (e as f32).sqrt();
        // Out-of-range indices (a vocabulary the net was not trained on) share the last row.
        let row = |i: usize, n: usize| -> usize { i.min(n.saturating_sub(1)) * e };
        let mut logits: Vec<f32> = options
            .iter()
            .map(|o| {
                let (k, d, f, z) = (row(o.kind as usize, self.n_kind), row(o.decision as usize, self.n_dec), row(o.subject_def as usize, self.n_def), row(o.subject_zone as usize, self.n_zone));
                let lv = (1.0 + o.value as f32).ln();
                let mut dot = 0.0;
                for j in 0..e {
                    dot += q[j] * (self.e_kind[k + j] + self.e_dec[d + j] + self.e_def[f + j] + self.e_zone[z + j] + self.w_val[j] * lv);
                }
                dot / norm
            })
            .collect();
        let m = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mut s = 0.0;
        for l in &mut logits {
            *l = (*l - m).exp();
            s += *l;
        }
        for l in &mut logits {
            *l /= s;
        }
        (value, logits)
    }
}

/// Evaluator backed by a `Net`.
pub struct NetEvaluator {
    pub net: std::sync::Arc<Net>,
    buf: Vec<f32>,
    n_defs: usize,
}

impl NetEvaluator {
    pub fn new(net: std::sync::Arc<Net>, n_defs: usize) -> NetEvaluator {
        NetEvaluator { net, buf: Vec::new(), n_defs }
    }
}

impl Evaluator for NetEvaluator {
    fn eval(&mut self, world: &Game, seat: Seat, n: usize) -> Eval {
        let o = world.seat_view(seat).observe();
        encode_state(&o, self.n_defs, &mut self.buf);
        let state: Vec<(u32, f32)> = self.buf.iter().enumerate().filter(|(_, v)| **v != 0.0).map(|(i, v)| (i as u32, *v)).collect();
        let opts = encode_options(&o);
        debug_assert_eq!(opts.len(), n);
        let (value, priors) = self.net.forward(&state, &opts);
        Eval { priors, value }
    }
}

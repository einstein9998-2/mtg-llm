//! Mana costs, the mana pool, and the payability solver (doc 01 section 6.4).

use crate::types::ManaColor;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// A mana cost. `pips[c]` counts colored (and colorless-specific) symbols in W U B R G C order.
/// `phy[c]` counts Phyrexian symbols {c/P}: each is paid with one `c` mana or 2 life, chosen at
/// cast time. Hybrid symbols are not in the pool yet and are rejected by the parser.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct ManaCost {
    pub generic: u8,
    pub pips: [u8; 6],
    /// Number of {X} symbols.
    pub x: u8,
    pub phy: [u8; 6],
    /// Hybrid symbols {a/b}, indexed by `hybrid_index(a, b)`.
    pub hyb: [u8; 10],
}

/// The ten unordered pairs of the five colors, W U B R G.
pub const HYBRID_PAIRS: [(usize, usize); 10] = [(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)];

pub fn hybrid_index(a: usize, b: usize) -> Option<usize> {
    let (a, b) = if a < b { (a, b) } else { (b, a) };
    HYBRID_PAIRS.iter().position(|&p| p == (a, b))
}

impl ManaCost {
    pub const ZERO: ManaCost = ManaCost { generic: 0, pips: [0; 6], x: 0, phy: [0; 6], hyb: [0; 10] };

    pub fn generic(n: u8) -> ManaCost {
        ManaCost { generic: n, ..ManaCost::ZERO }
    }

    pub fn try_parse(s: &str) -> Result<ManaCost, String> {
        let mut c = ManaCost::ZERO;
        let mut rest = s.trim();
        let bad = || format!("bad mana cost {s:?}");
        while !rest.is_empty() {
            if !rest.starts_with('{') {
                return Err(bad());
            }
            let end = rest.find('}').ok_or_else(bad)?;
            let sym = &rest[1..end];
            let color = |ch: &str| ["W", "U", "B", "R", "G", "C"].iter().position(|x| *x == ch);
            match sym {
                "X" => c.x += 1,
                n if n.chars().all(|ch| ch.is_ascii_digit()) && !n.is_empty() => c.generic += n.parse::<u8>().map_err(|_| bad())?,
                n if color(n).is_some() => c.pips[color(n).unwrap()] += 1,
                n if n.len() == 3 && n.ends_with("/P") && color(&n[..1]).is_some() => c.phy[color(&n[..1]).unwrap()] += 1,
                n if n.len() == 3 && n.as_bytes()[1] == b'/' && color(&n[..1]).map(|a| a < 5).unwrap_or(false) && color(&n[2..]).map(|b| b < 5).unwrap_or(false) => {
                    let (a, b) = (color(&n[..1]).unwrap(), color(&n[2..]).unwrap());
                    c.hyb[hybrid_index(a, b).ok_or_else(bad)?] += 1;
                }
                other => return Err(format!("unsupported mana symbol {{{other}}} in {s:?}")),
            }
            rest = &rest[end + 1..];
        }
        Ok(c)
    }

    /// Parses strings like "{2}{U}{U}", "{X}{R}", "{C}", "{1}{B/P}". Panics on bad input (card
    /// data errors must fail loudly at load time).
    pub fn parse(s: &str) -> ManaCost {
        ManaCost::try_parse(s).unwrap_or_else(|e| panic!("{e}"))
    }

    /// Mana value off the stack (X = 0). Phyrexian symbols count as one each.
    pub fn mana_value(&self) -> u32 {
        self.generic as u32 + self.pips.iter().map(|&p| p as u32).sum::<u32>() + self.phy.iter().map(|&p| p as u32).sum::<u32>() + self.hyb.iter().map(|&p| p as u32).sum::<u32>()
    }

    pub fn colors(&self) -> crate::types::Colors {
        let mut r = crate::types::Colors::empty();
        for c in ManaColor::ALL {
            if c != ManaColor::C && (self.pips[c.idx()] > 0 || self.phy[c.idx()] > 0) {
                r |= c.to_color();
            }
        }
        for (i, &(a, b)) in HYBRID_PAIRS.iter().enumerate() {
            if self.hyb[i] > 0 {
                r |= ManaColor::ALL[a].to_color() | ManaColor::ALL[b].to_color();
            }
        }
        r
    }

    pub fn is_zero(&self) -> bool {
        *self == ManaCost::ZERO
    }

    pub fn has_phyrexian(&self) -> bool {
        self.phy != [0; 6]
    }

    /// Total number of Phyrexian symbols.
    pub fn phyrexian_count(&self) -> u32 {
        self.phy.iter().map(|&p| p as u32).sum()
    }
}

impl std::fmt::Display for ManaCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.x {
            write!(f, "{{X}}")?;
        }
        if self.generic > 0 {
            write!(f, "{{{}}}", self.generic)?;
        }
        for c in ManaColor::ALL {
            for _ in 0..self.pips[c.idx()] {
                write!(f, "{{{}}}", c.symbol())?;
            }
            for _ in 0..self.phy[c.idx()] {
                write!(f, "{{{}/P}}", c.symbol())?;
            }
        }
        for (i, &(a, b)) in HYBRID_PAIRS.iter().enumerate() {
            for _ in 0..self.hyb[i] {
                write!(f, "{{{}/{}}}", ManaColor::ALL[a].symbol(), ManaColor::ALL[b].symbol())?;
            }
        }
        Ok(())
    }
}

impl Serialize for ManaCost {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ManaCost {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        ManaCost::try_parse(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct ManaPool(pub [u8; 6]);

impl ManaPool {
    pub fn total(&self) -> u32 {
        self.0.iter().map(|&x| x as u32).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.0 == [0; 6]
    }
    pub fn add(&mut self, c: ManaColor, n: u8) {
        self.0[c.idx()] = self.0[c.idx()].saturating_add(n);
    }
    pub fn clear(&mut self) {
        self.0 = [0; 6];
    }
}

/// A permanent (or other object) that can produce one mana by tapping, as seen by the solver.
#[derive(Copy, Clone, Debug)]
pub struct ManaSource {
    /// Caller-defined index (position in the caller's source list).
    pub tag: u32,
    /// Bit `c` set if it can produce `ManaColor` with index `c`.
    pub colors: u8,
    /// Lower is tapped first. Lets the caller preserve lands with relevant extra abilities.
    pub pref: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Payment {
    pub pool_used: [u8; 6],
    /// Part of `pool_used` that is restricted mana.
    pub restricted: [u8; 6],
    /// (source tag, color produced)
    pub taps: SmallVec<[(u32, ManaColor); 8]>,
}

/// Decides whether `cost` (with `generic` total generic mana, X already folded in) can be paid
/// from `pool` plus `sources`, and returns the canonical payment.
///
/// Policy (documented, deterministic): spend pool mana first; match colored pips to sources by
/// augmenting paths trying sources in `(pref, number of colors, tag)` order; pay generic with the
/// least valuable remaining sources in the same order.
///
/// Exactness: colored pips are payable iff a matching saturating them exists, and generic is
/// payable iff enough unmatched sources remain, so this is a complete decision procedure.
pub fn plan_payment(pips: [u8; 6], hyb: [u8; 10], generic: u32, pool: &ManaPool, sources: &[ManaSource]) -> Option<Payment> {
    let mut pay = Payment::default();
    let mut need = pips;
    let mut hneed = hyb;
    let mut gen = generic;
    // 1. Pool mana: exact-color first, then hybrid (either color), then generic.
    for c in 0..6 {
        let use_n = need[c].min(pool.0[c]);
        need[c] -= use_n;
        pay.pool_used[c] += use_n;
    }
    for (i, &(a, b)) in HYBRID_PAIRS.iter().enumerate() {
        for c in [a, b] {
            let avail = pool.0[c] - pay.pool_used[c];
            let use_n = hneed[i].min(avail);
            hneed[i] -= use_n;
            pay.pool_used[c] += use_n;
        }
    }
    for c in 0..6 {
        if gen == 0 {
            break;
        }
        let avail = pool.0[c] - pay.pool_used[c];
        let use_n = (avail as u32).min(gen) as u8;
        pay.pool_used[c] += use_n;
        gen -= use_n as u32;
    }
    let remaining_pips: u32 = need.iter().map(|&x| x as u32).sum::<u32>() + hneed.iter().map(|&x| x as u32).sum::<u32>();
    if remaining_pips + gen == 0 {
        return Some(pay);
    }
    if (sources.len() as u32) < remaining_pips + gen {
        return None;
    }
    // 2. Order sources by preference.
    let mut order: SmallVec<[usize; 16]> = (0..sources.len()).collect();
    order.sort_by_key(|&i| (sources[i].pref, sources[i].colors.count_ones(), sources[i].tag));
    // 3. Bipartite matching of pips (as color masks) to sources (Kuhn). Single-color pips go
    // first so that hybrid symbols take what is left.
    let mut pip_list: SmallVec<[u8; 12]> = SmallVec::new();
    for c in 0..6u8 {
        for _ in 0..need[c as usize] {
            pip_list.push(1 << c);
        }
    }
    for (i, &(a, b)) in HYBRID_PAIRS.iter().enumerate() {
        for _ in 0..hneed[i] {
            pip_list.push((1 << a) | (1 << b));
        }
    }
    let mut owner: SmallVec<[i16; 16]> = SmallVec::from_elem(-1, sources.len());
    fn try_assign(
        p: usize,
        pip_list: &[u8],
        sources: &[ManaSource],
        order: &[usize],
        owner: &mut [i16],
        seen: &mut [bool],
    ) -> bool {
        let mask = pip_list[p];
        for &s in order {
            if sources[s].colors & mask == 0 || seen[s] {
                continue;
            }
            seen[s] = true;
            if owner[s] < 0 || try_assign(owner[s] as usize, pip_list, sources, order, owner, seen) {
                owner[s] = p as i16;
                return true;
            }
        }
        false
    }
    for p in 0..pip_list.len() {
        let mut seen: SmallVec<[bool; 16]> = SmallVec::from_elem(false, sources.len());
        if !try_assign(p, &pip_list, &sources, &order, &mut owner, &mut seen) {
            return None;
        }
    }
    let mut used: SmallVec<[bool; 16]> = SmallVec::from_elem(false, sources.len());
    for &s in &order {
        if owner[s] >= 0 {
            used[s] = true;
            let mask = pip_list[owner[s] as usize] & sources[s].colors;
            let color = ManaColor::ALL[(mask.trailing_zeros() as usize).min(5)];
            pay.taps.push((sources[s].tag, color));
        }
    }
    // 4. Generic from the least valuable unused sources.
    for &s in &order {
        if gen == 0 {
            break;
        }
        if !used[s] {
            used[s] = true;
            let first = sources[s].colors.trailing_zeros() as usize;
            pay.taps.push((sources[s].tag, ManaColor::ALL[first.min(5)]));
            gen -= 1;
        }
    }
    if gen > 0 {
        return None;
    }
    pay.taps.sort_by_key(|t| t.0);
    Some(pay)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(tag: u32, colors: &[ManaColor]) -> ManaSource {
        let mut m = 0u8;
        for c in colors {
            m |= 1 << c.idx();
        }
        ManaSource { tag, colors: m, pref: 0 }
    }

    #[test]
    fn parse_costs() {
        let c = ManaCost::parse("{2}{U}{U}");
        assert_eq!(c.generic, 2);
        assert_eq!(c.pips[1], 2);
        assert_eq!(c.mana_value(), 4);
        assert_eq!(ManaCost::parse("{X}{R}").x, 1);
    }

    #[test]
    fn dual_land_flexibility() {
        use ManaColor::*;
        // {U}{R} with Island and a U/R dual: dual must pay R.
        let s = [src(0, &[U]), src(1, &[U, R])];
        let p = plan_payment([0, 1, 0, 1, 0, 0], [0; 10], 0, &ManaPool::default(), &s).unwrap();
        assert_eq!(p.taps.len(), 2);
        // Not payable: {U}{U} with Island and Mountain.
        let s = [src(0, &[U]), src(1, &[R])];
        assert!(plan_payment([0, 2, 0, 0, 0, 0], [0; 10], 0, &ManaPool::default(), &s).is_none());
    }

    #[test]
    fn generic_uses_cheapest_first() {
        use ManaColor::*;
        let mut a = src(0, &[U, R]);
        a.pref = 1;
        let b = src(1, &[U]);
        let p = plan_payment([0; 6], [0; 10], 1, &ManaPool::default(), &[a, b]).unwrap();
        assert_eq!(p.taps[0].0, 1);
    }

    #[test]
    fn pool_is_spent_first() {
        use ManaColor::*;
        let mut pool = ManaPool::default();
        pool.add(R, 1);
        let p = plan_payment([0, 1, 0, 0, 0, 0], [0; 10], 1, &pool, &[src(0, &[U])]).unwrap();
        assert_eq!(p.pool_used[R.idx()], 1);
        assert_eq!(p.taps.len(), 1);
    }
}

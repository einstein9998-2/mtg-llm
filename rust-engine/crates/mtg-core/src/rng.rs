//! Pinned, portable RNG (doc 01 section 12). PCG-XSL-RR 128/64 ("pcg64"), implemented here so no
//! dependency can change the stream. Changing anything in this file is a CORE CHANGE and
//! invalidates golden replays.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pcg64 {
    state: u128,
    inc: u128,
}

const MULT: u128 = 0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645;

impl Pcg64 {
    /// Seeds from a 64-bit seed using SplitMix64 to expand into state and stream.
    pub fn from_seed(seed: u64) -> Pcg64 {
        let mut sm = seed;
        let mut next = || {
            sm = sm.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        let s = ((next() as u128) << 64) | next() as u128;
        let i = ((next() as u128) << 64) | next() as u128;
        let mut r = Pcg64 { state: 0, inc: (i << 1) | 1 };
        r.state = r.state.wrapping_add(s);
        r.step();
        r
    }

    #[inline]
    fn step(&mut self) {
        self.state = self.state.wrapping_mul(MULT).wrapping_add(self.inc);
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.step();
        let rot = (self.state >> 122) as u32;
        let xsl = ((self.state >> 64) as u64) ^ (self.state as u64);
        xsl.rotate_right(rot)
    }

    /// Uniform in `0..n` (n > 0) by Lemire's multiply-shift with rejection. Pinned algorithm.
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0);
        let mut x = self.next_u64();
        let mut m = (x as u128) * (n as u128);
        let mut l = m as u64;
        if l < n {
            let t = n.wrapping_neg() % n;
            while l < t {
                x = self.next_u64();
                m = (x as u128) * (n as u128);
                l = m as u64;
            }
        }
        (m >> 64) as u64
    }

    /// Fisher-Yates, from the end towards the front. Pinned algorithm.
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        let mut i = v.len();
        while i > 1 {
            let j = self.below(i as u64) as usize;
            i -= 1;
            v.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_is_stable() {
        // Golden values: if this test fails, the RNG changed and every golden replay is invalid.
        let mut r = Pcg64::from_seed(42);
        let got: Vec<u64> = (0..4).map(|_| r.next_u64()).collect();
        assert_eq!(got, GOLDEN, "update GOLDEN only as part of a reviewed core change");
    }

    #[test]
    fn below_and_shuffle_are_stable() {
        // Pinned outputs of the derived operations (the golden replays only catch a change late).
        let mut r = Pcg64::from_seed(42);
        let below: Vec<u64> = (0..8).map(|i| r.below(3 + i)).collect();
        let mut v: Vec<u32> = (0..10).collect();
        r.shuffle(&mut v);
        assert_eq!((below, v), (BELOW_GOLDEN.to_vec(), SHUFFLE_GOLDEN.to_vec()), "update only as part of a reviewed core change");
    }

    const BELOW_GOLDEN: [u64; 8] = [1, 3, 2, 3, 0, 4, 7, 0];
    const SHUFFLE_GOLDEN: [u32; 10] = [3, 9, 8, 1, 4, 6, 2, 7, 0, 5];

    const GOLDEN: [u64; 4] = [6675540348766943785, 14005769090691728085, 9103385074472456323, 9552996099599578253];

    #[test]
    fn below_in_range_and_shuffle_perm() {
        let mut r = Pcg64::from_seed(7);
        for n in 1..200u64 {
            assert!(r.below(n) < n);
        }
        let mut v: Vec<u32> = (0..60).collect();
        r.shuffle(&mut v);
        let mut s = v.clone();
        s.sort();
        assert_eq!(s, (0..60).collect::<Vec<_>>());
        assert_ne!(v, (0..60).collect::<Vec<_>>());
    }
}

//! Deterministic hashing (no random seeds). Used for state hashes, DB hashes and signatures.

use std::hash::Hasher;

// `Hash` for integer slices and arrays feeds native-endian bytes to `write`; the pinned hashes
// (goldens, records) assume little-endian.
#[cfg(not(target_endian = "little"))]
compile_error!("state hashes are defined for little-endian targets only");

/// FxHash-style 64-bit multiplicative hasher with a final avalanche. Portable and pinned.
#[derive(Clone, Copy, Debug)]
pub struct Fx64(u64);

impl Default for Fx64 {
    fn default() -> Self {
        Fx64(0x243F_6A88_85A3_08D3)
    }
}

const K: u64 = 0x517C_C1B7_2722_0A95;

impl Fx64 {
    #[inline]
    fn mix(&mut self, w: u64) {
        self.0 = (self.0.rotate_left(5) ^ w).wrapping_mul(K);
    }
}

impl Hasher for Fx64 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for c in &mut chunks {
            self.mix(u64::from_le_bytes(c.try_into().unwrap()));
        }
        let rem = chunks.remainder();
        if !rem.is_empty() {
            let mut buf = [0u8; 8];
            buf[..rem.len()].copy_from_slice(rem);
            self.mix(u64::from_le_bytes(buf) ^ ((rem.len() as u64) << 56));
        }
    }
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.mix(i as u64)
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.mix(i as u64)
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.mix(i as u64)
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.mix(i)
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.mix(i as u64);
        self.mix((i >> 64) as u64)
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.mix(i as u64)
    }
    #[inline]
    fn finish(&self) -> u64 {
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

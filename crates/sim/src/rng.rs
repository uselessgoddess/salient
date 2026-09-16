//! The simulation's only source of randomness.
//!
//! SplitMix64, chosen because it is five lines, has no hidden state, and produces the same
//! stream from the same seed on every platform forever. The point is not statistical quality;
//! it is that the sequence is a pure function of the seed and the number of draws taken.

use crate::Fx;

#[derive(Clone, Copy, Debug, Default)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Rng(u64);

impl Rng {
    pub const fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform value in `[0, 1)`.
    pub fn unit(&mut self) -> Fx {
        Fx::from_frac_bits((self.next_u64() >> 32) as u32)
    }

    /// Uniform value in `[lo, hi)`.
    pub fn range(&mut self, lo: Fx, hi: Fx) -> Fx {
        lo + (hi - lo).mul(self.unit())
    }
}

//! The wide companion to [`Fx`], for quantities that do not fit in 64 bits.
//!
//! Squared distance is the motivating case and it is not a corner case: it is the hottest
//! quantity in the game, read once per candidate in every target-acquisition and collision
//! test. At map scale a squared distance in Q32.32 is about 1.4e19 against an `i64` ceiling of
//! 9.2e18, so keeping it in `Fx` would be a silent wrong answer in the busiest code there is.
//!
//! `Fx2` is Q64.64, which is exactly the product of two Q32.32 values with nothing discarded.
//! Its square root lands back in Q32.32 with no shifting, because halving 64 fractional bits
//! gives 32.

use super::{FRAC, Fx, sqrt};

/// Q64.64 fixed point. The exact product of two [`Fx`] values.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct Fx2(i128);

impl Fx2 {
    pub const ZERO: Fx2 = Fx2(0);
    pub const ONE: Fx2 = Fx2(1 << (FRAC * 2));

    #[inline]
    pub const fn from_bits(bits: i128) -> Fx2 {
        Fx2(bits)
    }

    #[inline]
    pub const fn to_bits(self) -> i128 {
        self.0
    }

    /// Widen a narrow value without loss.
    #[inline]
    pub const fn widen(v: Fx) -> Fx2 {
        Fx2((v.to_bits() as i128) << FRAC)
    }

    /// Square root, landing back in [`Fx`]. Negative input yields zero.
    #[inline]
    pub const fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        Fx::from_bits(sqrt::isqrt(self.0 as u128) as i64)
    }

    /// Narrow back to [`Fx`], saturating. Only correct where the caller knows the magnitude
    /// fits; prefer comparing two `Fx2` values directly over narrowing either of them.
    #[inline]
    pub const fn narrow(self) -> Fx {
        Fx::from_bits(super::narrow(self.0 >> FRAC))
    }
}

impl core::ops::Add for Fx2 {
    type Output = Fx2;
    #[inline]
    fn add(self, rhs: Fx2) -> Fx2 {
        Fx2(self.0 + rhs.0)
    }
}

impl core::ops::Sub for Fx2 {
    type Output = Fx2;
    #[inline]
    fn sub(self, rhs: Fx2) -> Fx2 {
        Fx2(self.0 - rhs.0)
    }
}

impl core::fmt::Debug for Fx2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}~", self.narrow())
    }
}

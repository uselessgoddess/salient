//! Q32.32 fixed point: a signed 64-bit integer with 32 fractional bits.
//!
//! This is the only arithmetic the simulation performs. Floating point is banned here because
//! `sin`, `cos`, `atan2` and friends resolve to a platform library that differs between
//! operating systems and versions, and one differing bit is a desync discovered months later.
//! Integer arithmetic is identical everywhere by construction rather than by discipline.
//!
//! Range is about +/- 2.1e9 against a 40 km map, and resolution is about 2.3e-10. Both are
//! chosen with room to spare: a long match is tens of thousands of integration steps, and the
//! headroom is what keeps accumulated drift irrelevant.
//!
//! Note that determinism is not accuracy. Results here drift from the real-valued answer, and
//! that is fine, because every participant drifts identically.

use core::{fmt, ops};

mod consts;
pub mod sqrt;
pub mod trig;
pub mod vec;
mod wide;

#[cfg(test)]
mod tests;

pub use vec::Vec2;
pub use wide::Fx2;

/// Q32.32 fixed-point scalar.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct Fx(pub(in crate::fx) i64);

/// Narrow a wide intermediate, saturating rather than wrapping.
///
/// Reaching either bound means a defect, not a representable answer. Saturation makes that
/// defect visible as something pinned at the edge of the world instead of teleported to the
/// far side of it, and unlike a raw `as` cast it is a defined operation.
const fn narrow(v: i128) -> i64 {
    if v > i64::MAX as i128 {
        i64::MAX
    } else if v < i64::MIN as i128 {
        i64::MIN
    } else {
        v as i64
    }
}

impl Fx {
    /// Fractional bits.
    pub const FRAC: u32 = 32;

    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(1 << Fx::FRAC);
    pub const HALF: Fx = Fx(1 << (Fx::FRAC - 1));
    pub const MIN: Fx = Fx(i64::MIN);
    pub const MAX: Fx = Fx(i64::MAX);
    pub const EPS: Fx = Fx(1);

    #[inline]
    pub const fn from_bits(bits: i64) -> Fx {
        Fx(bits)
    }

    /// A value in `[0, 1)` built from 32 bits of randomness. The bits become the fractional
    /// part directly: one representable value per input, no rounding, no rejected range.
    #[inline]
    pub const fn from_frac_bits(bits: u32) -> Fx {
        Fx(bits as i64)
    }

    #[inline]
    pub const fn to_bits(self) -> i64 {
        self.0
    }

    #[inline]
    pub const fn from_int(n: i32) -> Fx {
        Fx((n as i64) << Fx::FRAC)
    }

    /// Exact ratio, rounded toward negative infinity.
    #[inline]
    pub const fn from_ratio(num: i32, den: i32) -> Fx {
        Fx(narrow(((num as i128) << Fx::FRAC) / den as i128))
    }

    /// Largest integer not greater than this value.
    #[inline]
    pub const fn floor(self) -> i32 {
        (self.0 >> Fx::FRAC) as i32
    }

    /// Fractional part, always in `[0, 1)`, including for negatives.
    #[inline]
    pub const fn frac(self) -> Fx {
        Fx(self.0 & ((1 << Fx::FRAC) - 1))
    }

    #[inline]
    pub const fn abs(self) -> Fx {
        if self.0 < 0 { Fx(-self.0) } else { self }
    }

    #[inline]
    pub const fn signum(self) -> i32 {
        if self.0 > 0 {
            1
        } else if self.0 < 0 {
            -1
        } else {
            0
        }
    }

    #[inline]
    pub const fn min(self, rhs: Fx) -> Fx {
        if self.0 < rhs.0 { self } else { rhs }
    }

    #[inline]
    pub const fn max(self, rhs: Fx) -> Fx {
        if self.0 > rhs.0 { self } else { rhs }
    }

    #[inline]
    pub const fn clamp(self, lo: Fx, hi: Fx) -> Fx {
        self.max(lo).min(hi)
    }

    /// Multiply, rounding toward negative infinity.
    ///
    /// The shift is arithmetic, so rounding is floor rather than truncation toward zero. That
    /// is a deliberate choice: floor is one branchless instruction and is consistent for
    /// negatives, and consistency is what matters here.
    #[inline]
    pub const fn mul(self, rhs: Fx) -> Fx {
        Fx(narrow((self.0 as i128 * rhs.0 as i128) >> Fx::FRAC))
    }

    /// Divide, rounding toward negative infinity. Division by zero yields a saturated value
    /// rather than panicking, because the tick loop has no panicking paths.
    #[inline]
    pub const fn div(self, rhs: Fx) -> Fx {
        if rhs.0 == 0 {
            return if self.0 < 0 { Fx::MIN } else { Fx::MAX };
        }
        Fx(narrow(((self.0 as i128) << Fx::FRAC) / rhs.0 as i128))
    }

    /// Full-precision product, kept wide. See [`Fx2`].
    #[inline]
    pub const fn wide_mul(self, rhs: Fx) -> Fx2 {
        Fx2::from_bits(self.0 as i128 * rhs.0 as i128)
    }

    /// This value squared, kept wide. At map scale a squared distance does not fit in `Fx`.
    #[inline]
    pub const fn sq(self) -> Fx2 {
        self.wide_mul(self)
    }

    #[inline]
    pub const fn lerp(self, to: Fx, t: Fx) -> Fx {
        Fx(self.0 + narrow(((to.0 - self.0) as i128 * t.0 as i128) >> Fx::FRAC))
    }

    /// Square root, or zero for negative input. See [`sqrt`].
    #[inline]
    pub const fn sqrt(self) -> Fx {
        sqrt::sqrt(self)
    }
}

impl ops::Add for Fx {
    type Output = Fx;
    #[inline]
    fn add(self, rhs: Fx) -> Fx {
        Fx(self.0 + rhs.0)
    }
}

impl ops::Sub for Fx {
    type Output = Fx;
    #[inline]
    fn sub(self, rhs: Fx) -> Fx {
        Fx(self.0 - rhs.0)
    }
}

impl ops::Neg for Fx {
    type Output = Fx;
    #[inline]
    fn neg(self) -> Fx {
        Fx(-self.0)
    }
}

impl ops::Mul for Fx {
    type Output = Fx;
    #[inline]
    fn mul(self, rhs: Fx) -> Fx {
        Fx::mul(self, rhs)
    }
}

impl ops::Div for Fx {
    type Output = Fx;
    #[inline]
    fn div(self, rhs: Fx) -> Fx {
        Fx::div(self, rhs)
    }
}

impl ops::AddAssign for Fx {
    #[inline]
    fn add_assign(&mut self, rhs: Fx) {
        self.0 += rhs.0;
    }
}

impl ops::SubAssign for Fx {
    #[inline]
    fn sub_assign(&mut self, rhs: Fx) {
        self.0 -= rhs.0;
    }
}

/// Rendered as a decimal with nine fractional digits, computed in integers so that printing a
/// value never involves floating point.
impl fmt::Display for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.0 < 0;
        let m = self.abs().0 as u64;
        let whole = m >> Fx::FRAC;
        let frac = ((m & 0xffff_ffff) as u128 * 1_000_000_000) >> Fx::FRAC;
        if neg {
            f.write_str("-")?;
        }
        write!(f, "{whole}.{frac:09}")
    }
}

impl fmt::Debug for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

//! Two-dimensional vectors over [`Fx`].
//!
//! `len_sq` returns [`Fx2`] rather than `Fx` on purpose, and callers are expected to compare
//! squared lengths against squared thresholds instead of taking roots. At map scale the square
//! does not fit in 64 bits, and this is the hottest quantity in the simulation.

use super::{Fx, Fx2, trig};

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Debug,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct Vec2 {
    pub x: Fx,
    pub y: Fx,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: Fx::ZERO, y: Fx::ZERO };

    #[inline]
    pub const fn new(x: Fx, y: Fx) -> Vec2 {
        Vec2 { x, y }
    }

    #[inline]
    pub const fn splat(v: Fx) -> Vec2 {
        Vec2 { x: v, y: v }
    }

    #[inline]
    pub const fn scale(self, k: Fx) -> Vec2 {
        Vec2 { x: self.x.mul(k), y: self.y.mul(k) }
    }

    #[inline]
    pub const fn dot(self, rhs: Vec2) -> Fx2 {
        Fx2::from_bits(self.x.wide_mul(rhs.x).to_bits() + self.y.wide_mul(rhs.y).to_bits())
    }

    /// Squared length, kept wide. Compare these directly rather than rooting them.
    #[inline]
    pub const fn len_sq(self) -> Fx2 {
        self.dot(self)
    }

    #[inline]
    pub const fn len(self) -> Fx {
        self.len_sq().sqrt()
    }

    /// Unit vector in the same direction. The zero vector maps to itself.
    #[inline]
    pub fn norm(self) -> Vec2 {
        let len = self.len();
        if len == Fx::ZERO { Vec2::ZERO } else { Vec2 { x: self.x / len, y: self.y / len } }
    }

    /// Direction in turns, counter-clockwise from `+x`.
    #[inline]
    pub fn angle(self) -> Fx {
        trig::atan2(self.y, self.x)
    }
}

impl core::ops::Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl core::ops::Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl core::ops::AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

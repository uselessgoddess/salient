//! Trigonometry in fixed point, with angles measured in **turns**.
//!
//! One turn is a full revolution, so the fractional part of the value *is* the angle and range
//! reduction is a bit mask rather than a division by a transcendental constant. Radians are
//! avoided deliberately: they would put pi in the reduction path, where its rounding error
//! accumulates, and nothing in a game is easier to read in radians.
//!
//! Accuracy is around 1e-9 for sine and cosine and around 1e-6 for the arc tangent. The second
//! number looks loose and is not: a thousandth of a degree of facing error changes nothing, and
//! determinism is not accuracy — every participant computes the same wrong digit.

use super::consts::{
    INV_3, INV_5, INV_6, INV_7, INV_9, INV_11, INV_13, INV_24, INV_120, INV_720, INV_5040,
    INV_40320, INV_362880,
};
use super::{Fx, Vec2};

/// Taylor series for sine on `[0, pi/4]`, evaluated by Horner in `x^2`.
fn taylor_sin(x: Fx) -> Fx {
    let x2 = x * x;
    let p = INV_362880;
    let p = p * x2 - INV_5040;
    let p = p * x2 + INV_120;
    let p = p * x2 - INV_6;
    let p = p * x2 + Fx::ONE;
    p * x
}

/// Taylor series for cosine on `[0, pi/4]`, evaluated by Horner in `x^2`.
fn taylor_cos(x: Fx) -> Fx {
    let x2 = x * x;
    let p = INV_40320;
    let p = p * x2 - INV_720;
    let p = p * x2 + INV_24;
    let p = p * x2 - Fx::HALF;
    p * x2 + Fx::ONE
}

/// `sin(u * pi/2)` for `u` in `[0, 1]`.
///
/// The halves are served by different series so that neither is evaluated outside `[0, pi/4]`,
/// where the truncated tail stops being negligible.
fn quarter_sin(u: Fx) -> Fx {
    if u <= Fx::HALF {
        taylor_sin(u * Fx::FRAC_PI_2)
    } else {
        taylor_cos((Fx::ONE - u) * Fx::FRAC_PI_2)
    }
}

/// Sine of an angle in turns.
pub fn sin(turns: Fx) -> Fx {
    let t4 = turns.frac().to_bits() << 2;
    let quad = (t4 >> super::FRAC) & 3;
    let r = Fx::from_bits(t4 & ((1i64 << super::FRAC) - 1));
    match quad {
        0 => quarter_sin(r),
        1 => quarter_sin(Fx::ONE - r),
        2 => -quarter_sin(r),
        _ => -quarter_sin(Fx::ONE - r),
    }
}

/// Cosine of an angle in turns.
#[inline]
pub fn cos(turns: Fx) -> Fx {
    sin(turns + Fx::ratio(1, 4))
}

/// Unit vector pointing along an angle in turns.
#[inline]
pub fn unit(turns: Fx) -> Vec2 {
    Vec2 { x: cos(turns), y: sin(turns) }
}

/// Taylor series for arc tangent on `[-tan(pi/8), tan(pi/8)]`, in radians.
fn taylor_atan(z: Fx) -> Fx {
    let z2 = z * z;
    let p = INV_13;
    let p = p * z2 - INV_11;
    let p = p * z2 + INV_9;
    let p = p * z2 - INV_7;
    let p = p * z2 + INV_5;
    let p = p * z2 - INV_3;
    let p = p * z2 + Fx::ONE;
    p * z
}

/// Arc tangent of `z` in `[0, 1]`, in radians.
///
/// Above `tan(pi/8)` the series is folded through the half-angle identity, which brings the
/// argument back under that bound and keeps the truncated tail small everywhere.
fn atan_unit(z: Fx) -> Fx {
    if z <= Fx::TAN_PI_8 {
        taylor_atan(z)
    } else {
        Fx::FRAC_PI_4 + taylor_atan((z - Fx::ONE) / (z + Fx::ONE))
    }
}

/// Angle of the vector `(x, y)` in turns, in `[0, 1)`, measured counter-clockwise from `+x`.
///
/// The zero vector has no angle; it returns zero rather than panicking.
pub fn atan2(y: Fx, x: Fx) -> Fx {
    let (ax, ay) = (x.abs(), y.abs());
    if ax == Fx::ZERO && ay == Fx::ZERO {
        return Fx::ZERO;
    }

    // Reduce to the first octant so the series never sees an argument above 1, then reflect.
    let octant = if ax >= ay {
        atan_unit(ay / ax) * Fx::INV_TAU
    } else {
        Fx::ratio(1, 4) - atan_unit(ax / ay) * Fx::INV_TAU
    };

    let turns = match (x.to_bits() < 0, y.to_bits() < 0) {
        (false, false) => octant,
        (true, false) => Fx::HALF - octant,
        (true, true) => Fx::HALF + octant,
        (false, true) => Fx::ONE - octant,
    };
    turns.frac()
}

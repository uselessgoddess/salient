//! Reference tests for the fixed-point primitives.
//!
//! These are the one place in the project where a test per function is right rather than a
//! test per system. A wrong bit here does not surface here — it surfaces months later as a
//! desync in a subsystem that looks unrelated, and the golden replay test would point at the
//! symptom instead of the cause. This is the base case of the determinism argument, so the
//! base case gets pinned directly.
//!
//! References are exact rounded values computed off-line and written as raw bits, because the
//! simulation may not touch floating point even to check itself.

use super::{Fx, Fx2, Vec2, trig};

/// (turns, sin) as raw Q32.32 bits.
const SIN: &[(i64, i64)] = &[
    (0, 0),
    (134217728, 837906553),
    (268435456, 1643612827),
    (402653184, 2386155981),
    (536870912, 3037000500),
    (671088640, 3571134792),
    (805306368, 3968032378),
    (939524096, 4212440704),
    (1073741824, 4294967296),
    (1207959552, 4212440704),
    (1342177280, 3968032378),
    (1476395008, 3571134792),
    (1610612736, 3037000500),
    (1744830464, 2386155981),
    (1879048192, 1643612827),
    (2013265920, 837906553),
    (2147483648, 0),
    (2281701376, -837906553),
    (2415919104, -1643612827),
    (2550136832, -2386155981),
    (2684354560, -3037000500),
    (2818572288, -3571134792),
    (2952790016, -3968032378),
    (3087007744, -4212440704),
    (3221225472, -4294967296),
    (3355443200, -4212440704),
    (3489660928, -3968032378),
    (3623878656, -3571134792),
    (3758096384, -3037000500),
    (3892314112, -2386155981),
    (4026531840, -1643612827),
    (4160749568, -837906553),
    (1431655765, 3719550787),
    (2505397589, -2147483648),
    (3634203097, -3534688789),
    (613566757, 3357940648),
    (4290672329, -26985898),
];

/// (x, y, turns) as raw Q32.32 bits.
const ATAN2: &[(i64, i64, i64)] = &[
    (4294967296, 0, 0),
    (4294967296, 4294967296, 536870912),
    (0, 4294967296, 1073741824),
    (-4294967296, 4294967296, 1610612736),
    (-4294967296, 0, 2147483648),
    (-4294967296, -4294967296, 2684354560),
    (0, -4294967296, 3221225472),
    (4294967296, -4294967296, 3758096384),
    (12884901888, 4294967296, 219937506),
    (4294967296, 12884901888, 853804318),
    (-21474836480, 8589934592, 1887382701),
    (8589934592, -30064771072, 3411461455),
    (429496729600, 4294967296, 6835425),
    (4294967296, 429496729600, 1066906399),
];

/// (value, sqrt) as raw Q32.32 bits.
const SQRT: &[(i64, i64)] = &[
    (0, 0),
    (4294967296, 4294967296),
    (8589934592, 6074000999),
    (12884901888, 7439101573),
    (17179869184, 8589934592),
    (38654705664, 12884901888),
    (68719476736, 17179869184),
    (4294967296000, 135818791312),
    (530239482494976, 1509092852134),
    (1073741824, 2147483648),
];

fn near(got: Fx, want: i64, tol: i64) -> bool {
    (got.to_bits() - want).abs() <= tol
}

#[test]
fn sin_matches_reference() {
    // Ten chained multiplications each floor by up to one unit, so a few dozen units of slack
    // is the series working rather than failing. That is still far past anything the
    // simulation can observe.
    for &(t, want) in SIN {
        let got = trig::sin(Fx::from_bits(t));
        assert!(near(got, want, 256), "sin of raw {t} gave {got:?}, want raw {want}");
    }
}

#[test]
fn cos_leads_sin() {
    for &(t, _) in SIN {
        let a = Fx::from_bits(t);
        assert_eq!(trig::cos(a), trig::sin(a + Fx::from_ratio(1, 4)));
    }
}

#[test]
fn atan2_matches_reference() {
    for &(x, y, want) in ATAN2 {
        let got = trig::atan2(Fx::from_bits(y), Fx::from_bits(x));
        assert!(near(got, want, 20_000), "atan2 gave {got:?}, want raw {want}");
    }
    assert_eq!(trig::atan2(Fx::ZERO, Fx::ZERO), Fx::ZERO);
}

#[test]
fn sqrt_matches_reference() {
    for &(v, want) in SQRT {
        assert_eq!(Fx::from_bits(v).sqrt().to_bits(), want, "sqrt of raw {v}");
    }
    assert_eq!(Fx::from_int(-4).sqrt(), Fx::ZERO);
}

#[test]
fn wide_squares_exactly() {
    // The motivating case for Fx2: a squared distance at map scale overflows i64, so keeping
    // it narrow would be a silent wrong answer in the hottest path in the game.
    let far = Vec2::new(Fx::from_int(40_000), Fx::from_int(40_000));
    let sq = far.len_sq();
    assert!(sq.to_bits() > i64::MAX as i128, "map-scale square must exceed i64 range");
    assert_eq!(sq.sqrt().to_bits(), 242960039998083); // 40000 * sqrt(2)
}

#[test]
fn unit_vectors_normalise() {
    for &(t, _) in SIN {
        let len = trig::unit(Fx::from_bits(t)).len();
        assert!(near(len, Fx::ONE.to_bits(), 256), "unit vector had length {len:?}");
    }
}

#[test]
fn saturates_not_wraps() {
    // Reaching a bound is a defect, but it must be a defined one: saturation pins a value at
    // the edge of the world where a wrapping cast would teleport it to the far side.
    assert_eq!(Fx::MAX.mul(Fx::from_int(2)), Fx::MAX);
    assert_eq!(Fx::MIN.mul(Fx::from_int(2)), Fx::MIN);
    assert_eq!(Fx::ONE.div(Fx::ZERO), Fx::MAX);
    assert_eq!((-Fx::ONE).div(Fx::ZERO), Fx::MIN);
}

#[test]
fn round_trips_ratio() {
    assert_eq!(Fx::from_ratio(1, 2), Fx::HALF);
    assert_eq!(Fx::from_ratio(3, 1), Fx::from_int(3));
    assert_eq!(Fx::from_int(7).floor(), 7);
    assert_eq!(Fx::from_int(-7).floor(), -7);
    assert_eq!((Fx::from_int(-7) - Fx::HALF).floor(), -8);
    assert_eq!(Fx::HALF.frac(), Fx::HALF);
    assert_eq!((-Fx::HALF).frac(), Fx::HALF);
    assert_eq!(Fx2::widen(Fx::ONE), Fx2::ONE);
}

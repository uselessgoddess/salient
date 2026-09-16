//! Integer square root by Newton's method.
//!
//! Written here rather than taken from a library for the reason given in the module docs of
//! [`super`]: the surface is fifteen lines, and owning it removes one more thing that could
//! change a single bit between releases.

use super::{FRAC, Fx};

/// Floor of the square root. Exact for every input.
///
/// The initial guess is a power of two at least as large as the answer, so the iteration
/// descends monotonically and terminates when it stops decreasing.
#[inline]
pub const fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let bits = 128 - n.leading_zeros();
    let mut x = 1u128 << bits.div_ceil(2);
    loop {
        let next = (x + n / x) >> 1;
        if next >= x {
            return x;
        }
        x = next;
    }
}

/// Square root of a [`Fx`] value. Negative input yields zero rather than panicking, because
/// the tick loop has no panicking paths.
///
/// Shifting left by `FRAC` before the root is what keeps the result in Q32.32: the root of
/// `a * 2^-32` is `sqrt(a) * 2^-16`, and `sqrt(a << 32)` is the same quantity scaled back.
#[inline]
pub const fn sqrt(v: Fx) -> Fx {
    if v.to_bits() <= 0 {
        return Fx::ZERO;
    }
    Fx::from_bits(isqrt((v.to_bits() as u128) << FRAC) as i64)
}

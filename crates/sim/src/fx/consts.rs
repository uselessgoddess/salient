//! Q32.32 constants, generated once and pinned.
//!
//! These are exact rounded values of the mathematical constants at this scale. They are
//! written out rather than computed so that nothing in the simulation can depend on a
//! platform library, and so that a change to any of them is visible in a diff.

use super::Fx;

impl Fx {
    /// 3.141592653589793
    pub const PI: Fx = Fx(13493037705);
    /// 1.5707963267948966
    pub const FRAC_PI_2: Fx = Fx(6746518852);
    /// 0.7853981633974483
    pub const FRAC_PI_4: Fx = Fx(3373259426);
    /// 6.283185307179586
    pub const TAU: Fx = Fx(26986075409);
    /// 0.41421356237309503
    pub const TAN_PI_8: Fx = Fx(1779033704);
    /// 0.15915494309189535
    pub const INV_TAU: Fx = Fx(683565276);
}

pub(super) const INV_3: Fx = Fx(1431655765); // 1/3
pub(super) const INV_5: Fx = Fx(858993459); // 1/5
pub(super) const INV_6: Fx = Fx(715827883); // 1/6
pub(super) const INV_7: Fx = Fx(613566757); // 1/7
pub(super) const INV_9: Fx = Fx(477218588); // 1/9
pub(super) const INV_11: Fx = Fx(390451572); // 1/11
pub(super) const INV_13: Fx = Fx(330382100); // 1/13
pub(super) const INV_24: Fx = Fx(178956971); // 1/24
pub(super) const INV_120: Fx = Fx(35791394); // 1/120
pub(super) const INV_720: Fx = Fx(5965232); // 1/720
pub(super) const INV_5040: Fx = Fx(852176); // 1/5040
pub(super) const INV_40320: Fx = Fx(106522); // 1/40320
pub(super) const INV_362880: Fx = Fx(11836); // 1/362880
#[allow(dead_code)]
pub(super) const INV_3628800: Fx = Fx(1184); // 1/3628800

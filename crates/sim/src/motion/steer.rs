//! Heading toward an ordered destination.
//!
//! Delete this and units keep whatever velocity they already have, keep moving, and simply
//! cannot be told where to go. Nothing else notices.
//!
//! Long-range routing is a later increment; for now a unit heads straight at its destination.
//! Every unit reads its own state and writes only its own, which is the phase shape that
//! parallelises later without any result depending on scheduling.

use crate::{Fx, Fx2, State, Vec2};

/// Metres per second. Per-class speeds arrive with the unit roster.
const SPEED: Fx = Fx::from_int(10);

/// Within this distance of its destination a unit has arrived. Without it a unit oscillates
/// across the target forever, one integration step at a time.
const ARRIVED: Fx = Fx::ONE;

/// Compared against squared distance, so no root is taken in the hot path.
const ARRIVED_SQ: Fx2 = ARRIVED.sq();

pub fn steer(state: &mut State) {
    let State { units, goals, .. } = state;

    goals.retain(|h, goal| {
        // A handle the arena no longer knows belongs to a unit that died, or to a slot since
        // reused. Either way this destination is not its business, and dropping it here is
        // what keeps a rebuilt unit from inheriting a dead one's orders.
        let Some(u) = units.get_mut(h) else { return false };

        let delta = *goal - u.pos;
        if delta.len_sq() <= ARRIVED_SQ {
            u.vel = Vec2::ZERO;
            false
        } else {
            u.vel = delta.norm().scale(SPEED);
            u.facing = delta.angle();
            true
        }
    });
}

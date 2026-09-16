//! Steering toward a goal and integrating position.
//!
//! Long-range routing is a later increment; for now a unit heads straight at its goal. What
//! matters here is the shape: every unit reads its own state and writes only its own, which is
//! the phase shape that can be parallelised later without any result depending on scheduling.

use crate::fx::{Fx, Vec2};
use crate::state::{DT, State};

/// Metres per second. Per-class speeds arrive with the unit roster.
const SPEED: Fx = Fx::from_bits(10 << 32);

/// Within this distance of the goal a unit has arrived. Without it a unit oscillates across
/// its target forever, one integration step at a time.
const ARRIVED: Fx = Fx::from_bits(1 << 32);

pub fn advance(state: &mut State) {
    let arrived_sq = ARRIVED.sq();

    for u in state.units.values_mut() {
        if let Some(goal) = u.goal {
            let delta = goal - u.pos;
            if delta.len_sq() <= arrived_sq {
                u.goal = None;
                u.vel = Vec2::ZERO;
            } else {
                u.vel = delta.norm().scale(SPEED);
                u.facing = delta.angle();
            }
        }

        u.pos += u.vel.scale(DT);
    }
}

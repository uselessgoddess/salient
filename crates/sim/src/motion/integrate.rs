//! Advancing position by velocity.
//!
//! This is the part of movement that is not deletable: without it nothing on the map moves at
//! all. It runs for every unit whether or not anything told it where to go.

use crate::State;

pub fn integrate(state: &mut State) {
    for u in state.units.values_mut() {
        u.pos += u.vel.scale(State::DT);
    }
}

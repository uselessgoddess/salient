//! Movement, in two phases.
//!
//! Steering decides velocity for the units that were told to go somewhere; integration applies
//! velocity to every unit. They are split because only the first is a removable subsystem, and
//! because steering costs what the commanded units cost rather than what the roster costs.

mod integrate;
mod steer;

pub use integrate::integrate;
pub use steer::steer;

use crate::State;

pub fn advance(state: &mut State) {
    steer(state);
    integrate(state);
}

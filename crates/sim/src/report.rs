//! What a tick did, for anyone who needs to react without diffing state.
//!
//! The report carries no authority: discarding it changes nothing about the simulation. It
//! exists so that a renderer does not have to compare two states to notice a death, and so a
//! recorder does not have to guess when a match ended.

use crate::arena::Handle;
use crate::unit::Player;

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub spawned: Vec<Handle>,
    pub died: Vec<Handle>,
    /// Set on the tick a defeat condition is first met.
    pub defeated: Option<Player>,
    /// True on the tick production was refused because the cap was reached. Visible failure
    /// rather than silent failure.
    pub at_cap: bool,
}

impl Report {
    pub fn is_quiet(&self) -> bool {
        self.spawned.is_empty() && self.died.is_empty() && self.defeated.is_none() && !self.at_cap
    }
}

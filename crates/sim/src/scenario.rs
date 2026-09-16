//! Starting arrangements.
//!
//! A scenario is match setup, exactly like the seed and the rules: it goes into a recording
//! and comes back out, so a replay reconstructs the same opening without anyone reselecting
//! it by hand.

use crate::fx::{Fx, Vec2};
use crate::state::State;
use crate::unit::{Domain, Unit};

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[repr(u8)]
pub enum Scenario {
    /// Nothing. The starting point for tests that build their own world.
    #[default]
    Empty = 0,
    /// A mixed-domain force on constant headings. Exists so the first increment has something
    /// to look at: it shows smooth motion at sixty frames from a ten-hertz simulation, and it
    /// puts all four domains on screen at once.
    Drift = 1,
}

impl Scenario {
    pub fn parse(s: &str) -> Option<Scenario> {
        match s {
            "empty" => Some(Scenario::Empty),
            "drift" => Some(Scenario::Drift),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Scenario::Empty => "empty",
            Scenario::Drift => "drift",
        }
    }
}

/// Populate a fresh state. Draws only from the seeded generator, so the arrangement is a pure
/// function of the seed.
pub fn spawn(state: &mut State, count: u32) {
    match state.scenario {
        Scenario::Empty => {}
        Scenario::Drift => drift(state, count),
    }
}

fn drift(state: &mut State, count: u32) {
    let extent = state.extent;
    for i in 0..count {
        let domain = Domain::ALL[(i % 4) as usize];
        let pos =
            Vec2::new(state.rng.range(Fx::ZERO, extent.x), state.rng.range(Fx::ZERO, extent.y));
        let heading = state.rng.unit();
        let speed = state.rng.range(Fx::from_int(5), Fx::from_int(40));

        let mut u = Unit::new(pos, domain, 0);
        u.vel = crate::fx::trig::unit(heading).scale(speed);
        u.facing = heading;
        state.units.insert(u);
    }
}

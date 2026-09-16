//! The whole of the simulation's state.
//!
//! Everything here participates in the fingerprint. Anything that could change an outcome but
//! is not here is a breach of the sim/render boundary — camera, selection, display filters and
//! the interpolation factor deliberately live on the other side of it.

use crate::arena::{Arena, Table};
use crate::rng::Rng;
use crate::{Fx, Rules, Scenario, Unit, Vec2};

#[derive(Clone, Debug)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct State {
    pub tick: u32,
    pub seed: u64,
    pub rng: Rng,
    pub rules: Rules,
    pub scenario: Scenario,
    pub extent: Vec2,
    pub units: Arena<Unit>,
    /// Where each unit has been told to go. Held beside the units rather than inside them:
    /// a destination is what a unit was ordered to do, not what a unit is. Delete steering
    /// and this field goes with it, touching nothing else.
    pub goals: Table<Vec2>,
}

impl State {
    /// Simulation updates per second. Fixed and low on purpose: it is what makes twenty
    /// thousand units affordable, and what keeps lockstep latency tolerable later. Visuals
    /// interpolate between ticks rather than following this rate.
    pub const TICK_RATE: u32 = 10;

    /// Seconds per tick, as the simulation sees it. The only notion of duration that exists
    /// in here.
    pub const DT: Fx = Fx::from_ratio(1, State::TICK_RATE as i32);

    pub fn new(seed: u64, rules: Rules, scenario: Scenario) -> State {
        State {
            tick: 0,
            seed,
            rng: Rng::new(seed),
            rules,
            scenario,
            extent: Vec2::new(Fx::from_int(40_000), Fx::from_int(40_000)),
            units: Arena::new(),
            goals: Table::new(),
        }
    }

    pub fn in_bounds(&self, p: Vec2) -> bool {
        p.x >= Fx::ZERO && p.y >= Fx::ZERO && p.x <= self.extent.x && p.y <= self.extent.y
    }
}

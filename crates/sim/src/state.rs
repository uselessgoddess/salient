//! The whole of the simulation's state.
//!
//! Everything here participates in the fingerprint. Anything that could change an outcome but
//! is not here is a breach of the sim/render boundary — camera, selection, display filters and
//! the interpolation factor deliberately live on the other side of it.

use crate::arena::Arena;
use crate::fx::{Fx, Vec2};
use crate::rng::Rng;
use crate::rules::Rules;
use crate::scenario::Scenario;
use crate::unit::Unit;

/// Simulation updates per second. Fixed and low on purpose: it is what makes twenty thousand
/// units affordable, and what keeps lockstep latency tolerable later. Visuals interpolate.
pub const TICK_RATE: u32 = 10;

/// Seconds per tick, as the simulation sees it. The only notion of duration that exists here.
pub const DT: Fx = Fx::ratio(1, TICK_RATE as i32);

#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct State {
    pub tick: u32,
    pub seed: u64,
    pub rng: Rng,
    pub rules: Rules,
    pub scenario: Scenario,
    /// World size. The map itself arrives in the next increment; until then this is the whole
    /// of the world's geometry.
    pub extent: Vec2,
    pub units: Arena<Unit>,
}

impl State {
    pub fn new(seed: u64, rules: Rules, scenario: Scenario) -> State {
        State {
            tick: 0,
            seed,
            rng: Rng::new(seed),
            rules,
            scenario,
            extent: Vec2::new(Fx::from_int(40_000), Fx::from_int(40_000)),
            units: Arena::new(),
        }
    }

    pub fn in_bounds(&self, p: Vec2) -> bool {
        p.x >= Fx::ZERO && p.y >= Fx::ZERO && p.x <= self.extent.x && p.y <= self.extent.y
    }
}

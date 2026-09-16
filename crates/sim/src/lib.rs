//! Salient's simulation.
//!
//! This crate knows nothing about engines, renderers, networks or clocks, and it performs no
//! I/O. Its whole surface is: apply orders, advance one tick, read state, read a fingerprint.
//! Orders are the only input; state is the only output.
//!
//! That single boundary is what makes replays, headless tests, dedicated servers, lockstep
//! multiplayer and opponent development fall out of one piece of work instead of being built
//! four times. A renderer, an opponent, a recorder and a network peer are indistinguishable
//! from in here.
//!
//! # Guarantees
//!
//! - **Pure.** [`Sim::step`] is a function of state and orders. Equal fingerprints stepped
//!   with equal orders produce equal fingerprints, on every platform and in every profile.
//! - **Total.** `step` never panics and never fails. Malformed input is refused by
//!   [`Sim::validate`] at the boundary.
//! - **Closed.** No I/O, no clock, no environment, no randomness but the seeded generator.
//! - **Fixed cadence.** One `step` is one tick. Real time, frames and speed are the caller's
//!   concern entirely.
//! - **Bounded work.** Work per step is a function of state, never of a time budget. Amortised
//!   computation is sliced by a fixed count, never by a timer.

pub mod arena;
pub mod command;
pub mod fx;
pub mod motion;
pub mod record;
pub mod report;
pub mod rng;
pub mod rules;
pub mod scenario;
pub mod state;
pub mod unit;

pub use arena::Handle;
pub use command::{Order, Reject};
pub use fx::{Fx, Fx2, Vec2};
pub use record::{LoadError, Recording};
pub use report::Report;
pub use rules::{Defeat, Rules};
pub use scenario::Scenario;
pub use state::State;
pub use unit::{Domain, Player, Unit};

/// Identifies the behaviour a recording was made by, so that one simulation never reads an
/// artifact produced by a different one.
///
/// Held fixed while the project is pre-release: nothing yet outlives the build that made it,
/// so there is no reader for the number. A behavioural change is carried by the regenerated
/// golden fingerprints instead. Handing out a build, keeping a replay, or playing a networked
/// match restores per-change bumps, and that same change must restore them.
pub const SIM_VERSION: u32 = 1;

/// A match in progress.
pub struct Sim {
    state: State,
}

impl Sim {
    /// Start a match. Seed, rules and scenario are the whole of the setup, which is why those
    /// three plus the order log are the whole of a recording.
    pub fn new(seed: u64, rules: Rules, scenario: Scenario, units: u32) -> Sim {
        let mut state = State::new(seed, rules, scenario);
        scenario::spawn(&mut state, units);
        Sim { state }
    }

    /// Advance exactly one tick.
    pub fn step(&mut self, orders: &[Order]) -> Report {
        let report = Report::default();

        command::apply(&mut self.state, orders);
        motion::advance(&mut self.state);

        self.state.tick += 1;
        report
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn tick(&self) -> u32 {
        self.state.tick
    }

    pub fn fingerprint(&self) -> u64 {
        record::fingerprint::of(&self.state)
    }

    pub fn snapshot(&self) -> Vec<u8> {
        record::snapshot::to_bytes(&self.state)
    }

    pub fn restore(bytes: &[u8]) -> Result<Sim, LoadError> {
        record::snapshot::from_bytes(bytes)
            .map(|state| Sim { state })
            .map_err(|_| LoadError::Corrupt)
    }

    /// Refuse an order before it is recorded. Rejected orders never reach the tick loop and
    /// never appear in a replay.
    pub fn validate(&self, order: &Order) -> Result<(), Reject> {
        command::validate(&self.state, order)
    }

    /// Replay a recording to its end, returning the fingerprint at every checkpoint it names.
    ///
    /// This is what the golden test and the divergence bisector both run, so neither needs to
    /// know anything about how a tick works.
    pub fn replay(rec: &Recording) -> Vec<(u32, u64)> {
        let mut sim = Sim::new(rec.seed, rec.rules.clone(), rec.scenario, rec.units);
        let last = rec.checkpoints.iter().map(|&(t, _)| t).max().unwrap_or(0);
        let mut out = Vec::with_capacity(rec.checkpoints.len());
        let mut want = rec.checkpoints.iter().peekable();

        while sim.tick() <= last {
            if let Some(&&(t, _)) = want.peek()
                && t == sim.tick()
            {
                out.push((t, sim.fingerprint()));
                want.next();
            }
            sim.step(rec.due(sim.tick()));
        }
        out
    }
}

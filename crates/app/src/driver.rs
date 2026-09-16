//! Drives the simulation and holds the two ticks the renderer interpolates between.
//!
//! This is the only place the two sides meet, and the traffic is one-way: the driver steps the
//! simulation and reads state out of it. Nothing here writes to the simulation except through
//! orders, which is what keeps the boundary in Principle II real rather than aspirational.
//!
//! The simulation runs at a fixed ten hertz and knows nothing about frames or real time.
//! Falling behind is detected here, not there.

use bevy::prelude::*;
use salient_sim::{Fx, Sim, State};

use crate::args::Args;

/// Seconds per simulation tick, on the rendering side of the boundary.
const TICK_SECS: f32 = 1.0 / State::TICK_RATE as f32;

/// Never advance more than this many ticks in one frame. Without a bound, a stalled frame
/// makes the next one try to catch up all at once and stall again.
const MAX_CATCH_UP: u32 = 5;

/// One unit as the renderer sees it.
#[derive(Clone, Copy, Default)]
pub struct Shot {
    pub pos: Vec2,
    pub facing: f32,
    pub domain: u8,
}

#[derive(Resource)]
pub struct Driver {
    sim: Sim,
    /// State at the previous tick and at the current one. Rendering shows a point between.
    pub prev: Vec<Shot>,
    pub next: Vec<Shot>,
    /// How far between `prev` and `next` the current frame sits, in `[0, 1]`.
    pub alpha: f32,
    /// True on the frames a tick actually happened, so the mesh rebuild can be skipped.
    pub stepped: bool,
    accum: f32,
}

impl Driver {
    pub fn new(args: &Args) -> Driver {
        let sim = Sim::new(args.seed, args.rules.clone(), args.scenario, args.units);
        let shot = snapshot(&sim);
        Driver { sim, prev: shot.clone(), next: shot, alpha: 0.0, stepped: true, accum: 0.0 }
    }

    pub fn tick(&self) -> u32 {
        self.sim.tick()
    }

    pub fn fingerprint(&self) -> u64 {
        self.sim.fingerprint()
    }
}

/// Convert a fixed-point value to the renderer's floating point.
///
/// One-way by design. Values never travel back: the simulation would stop being deterministic
/// the moment a float re-entered it.
#[inline]
pub fn world(v: Fx) -> f32 {
    v.to_bits() as f32 / (1u64 << 32) as f32
}

fn snapshot(sim: &Sim) -> Vec<Shot> {
    sim.state()
        .units
        .values()
        .map(|u| Shot {
            pos: Vec2::new(world(u.pos.x), world(u.pos.y)),
            facing: world(u.facing),
            domain: u.domain as u8,
        })
        .collect()
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (advance, title).chain());
}

/// Put the tick count in the window title.
///
/// Small, but it is what makes the first checkpoint answerable: you can see at a glance
/// whether the simulation is advancing at ten hertz while the picture moves at sixty.
fn title(driver: Res<Driver>, mut windows: Query<&mut Window>) {
    // Once a second, not once a tick: a fingerprint serializes the whole of state, which is
    // nothing at five hundred units and would be noticeable at twenty thousand.
    if !driver.stepped || !driver.tick().is_multiple_of(State::TICK_RATE) {
        return;
    }
    for mut w in &mut windows {
        w.title = format!(
            "salient — tick {} — {} units — {:016x}",
            driver.tick(),
            driver.next.len(),
            driver.fingerprint()
        );
    }
}

fn advance(mut driver: ResMut<Driver>, time: Res<Time>) {
    driver.stepped = false;
    driver.accum += time.delta_secs();

    let mut steps = 0;
    while driver.accum >= TICK_SECS && steps < MAX_CATCH_UP {
        driver.sim.step(&[]);
        driver.prev = std::mem::take(&mut driver.next);
        driver.next = snapshot(&driver.sim);
        driver.accum -= TICK_SECS;
        driver.stepped = true;
        steps += 1;
    }

    // Behind by more than the catch-up bound: drop the backlog rather than spiral. The
    // simulation is unaffected either way; only what the player sees is.
    if driver.accum >= TICK_SECS {
        driver.accum = 0.0;
    }

    driver.alpha = (driver.accum / TICK_SECS).clamp(0.0, 1.0);
}

/// Report the scenario once at startup so a run is identifiable from its log alone.
pub fn announce(driver: Res<Driver>, args: Res<Args>) {
    info!(
        "seed {} scenario {} units {} | sim v{} | fingerprint {:016x}",
        args.seed,
        args.scenario.name(),
        args.units,
        salient_sim::SIM_VERSION,
        driver.fingerprint()
    );
}

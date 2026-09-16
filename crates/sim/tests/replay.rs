//! The golden replay test.
//!
//! This is the keystone. One assertion covers the whole simulation: a seed, a scenario and an
//! order log reach an expected fingerprint at every checkpoint, so any change to any part of
//! any tick phase shows up here. It is deliberately the only broad test in the crate —
//! twenty narrow ones would restate the functions rather than prove the system.
//!
//! The pinned values are not claims about correctness. They are claims about *stability*: a
//! change to them is a change to simulation behaviour, and must be deliberate. Regenerate them
//! and say why in the commit. Run the same seed through the `fingerprint` tool on Linux,
//! Windows and macOS and compare the output — that is the half of the proof this test cannot
//! make on its own.

use salient_sim::command::{Action, Target};
use salient_sim::{Fx, Order, Recording, Rules, Scenario, Sim, Vec2};

const SEED: u64 = 1;
const UNITS: u32 = 512;

/// Fingerprints after the scripted match, pinned. Regenerate deliberately, never casually.
const GOLDEN: &[(u32, u64)] = &[
    (0, 0x020622607e2662d7),
    (250, 0x6f27cfe2ac906dc9),
    (500, 0xb8222fe06fe1ad8a),
    (1000, 0x49af2735c3b004d4),
    (2000, 0x2d1217930380d064),
];

/// A short match with orders that actually change the outcome, so the log is load-bearing
/// rather than decorative.
fn scripted() -> Recording {
    let probe = Sim::new(SEED, Rules::default(), Scenario::Drift, UNITS);
    let handles: Vec<_> = probe.state().units.iter().map(|(h, _)| h).take(64).collect();

    let mut orders = Vec::new();
    for (i, chunk) in handles.chunks(8).enumerate() {
        let tick = 10 + i as u32 * 37;
        let to = Vec2::new(Fx::from_int(1_000 + i as i32 * 700), Fx::from_int(20_000));
        orders.push(Order {
            tick,
            issuer: 0,
            target: Target::Units(chunk.to_vec()),
            action: Action::Move { to },
        });
        orders.push(Order {
            tick: tick + 400,
            issuer: 0,
            target: Target::Units(chunk.to_vec()),
            action: Action::Stop,
        });
    }
    orders.sort_by_key(|o| o.tick);

    Recording {
        sim_version: salient_sim::SIM_VERSION,
        seed: SEED,
        rules: Rules::default(),
        scenario: Scenario::Drift,
        units: UNITS,
        orders,
        checkpoints: GOLDEN.to_vec(),
    }
}

#[test]
fn replay_matches_golden() {
    let got = Sim::replay(&scripted());
    if got != GOLDEN {
        let listed: Vec<String> =
            got.iter().map(|(t, f)| format!("    ({t}, 0x{f:016x}),")).collect();
        panic!(
            "simulation behaviour changed. If that was deliberate, pin these and say why:\n{}",
            listed.join("\n")
        );
    }
}

#[test]
fn replay_repeats_exactly() {
    let rec = scripted();
    assert_eq!(Sim::replay(&rec), Sim::replay(&rec));
}

#[test]
fn refuses_stale_version() {
    let mut rec = scripted();
    rec.sim_version = salient_sim::SIM_VERSION + 1;
    let bytes = rec.to_bytes();
    // A recording made by different behaviour must be refused, not played back to a plausible
    // wrong result. A wrong result would look exactly like a real desync.
    match Recording::from_bytes(&bytes) {
        Err(salient_sim::LoadError::SimVersion { .. }) => {}
        other => panic!("expected a version refusal, got {other:?}"),
    }
}

#[test]
fn recording_round_trips() {
    let rec = scripted();
    assert_eq!(Recording::from_bytes(&rec.to_bytes()).unwrap(), rec);
    assert!(Recording::from_bytes(b"not a recording at all").is_err());
}

#[test]
fn snapshot_round_trips() {
    let mut sim = Sim::new(SEED, Rules::default(), Scenario::Drift, UNITS);
    for _ in 0..137 {
        sim.step(&[]);
    }
    let resumed = Sim::restore(&sim.snapshot()).expect("snapshot restores");
    assert_eq!(resumed.fingerprint(), sim.fingerprint());
    assert_eq!(resumed.tick(), sim.tick());
}

#[test]
fn rejects_offmap_order() {
    let sim = Sim::new(SEED, Rules::default(), Scenario::Drift, UNITS);
    let h = sim.state().units.iter().next().unwrap().0;

    let off = Order {
        tick: 0,
        issuer: 0,
        target: Target::Units(vec![h]),
        action: Action::Move { to: Vec2::new(Fx::from_int(-1), Fx::ZERO) },
    };
    assert_eq!(sim.validate(&off), Err(salient_sim::Reject::OffMap));

    let stranger = Order { issuer: 3, ..off.clone() };
    assert_eq!(sim.validate(&stranger), Err(salient_sim::Reject::NotOwned));
}

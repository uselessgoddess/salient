//! Tick budget.
//!
//! Ten hertz gives a hundred milliseconds per tick; the target is ten, a tenfold margin. This
//! measures now so that a regression later has something to be a regression against.

use criterion::{Criterion, criterion_group, criterion_main};
use salient_sim::{Rules, Scenario, Sim};

fn tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick");
    for units in [1_000u32, 10_000, 20_000] {
        group.bench_function(format!("drift/{units}"), |b| {
            let mut sim = Sim::new(1, Rules::default(), Scenario::Drift, units);
            b.iter(|| sim.step(&[]));
        });
    }
    group.finish();
}

criterion_group!(benches, tick);
criterion_main!(benches);

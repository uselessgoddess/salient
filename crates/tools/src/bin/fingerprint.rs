//! Print state fingerprints for a seed.
//!
//! Run this on every target platform and compare the output. If the three disagree, the
//! simulation is not deterministic, and nothing built on top of it is trustworthy. That is why
//! this is a command rather than a paragraph in a document: a check that cannot be run with one
//! line does not get run.

use std::process::ExitCode;

use salient_sim::{Rules, Scenario, Sim};

fn main() -> ExitCode {
    let mut seed = 1u64;
    let mut ticks = 6_000u32;
    let mut units = 512u32;
    let mut scenario = Scenario::Drift;
    let mut every = 500u32;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let next = args.get(i + 1);
        match args[i].as_str() {
            "--seed" => seed = parse(next, "--seed"),
            "--ticks" => ticks = parse(next, "--ticks"),
            "--units" => units = parse(next, "--units"),
            "--every" => every = parse(next, "--every"),
            "--scenario" => {
                scenario = next
                    .and_then(|s| Scenario::parse(s))
                    .unwrap_or_else(|| fail("--scenario expects empty or drift"));
            }
            "--help" | "-h" => {
                println!(
                    "fingerprint [--seed N] [--ticks N] [--units N] [--every N] [--scenario NAME]"
                );
                return ExitCode::SUCCESS;
            }
            other => fail(&format!("unknown argument {other}")),
        }
        i += 2;
    }

    let mut sim = Sim::new(seed, Rules::default(), scenario, units);
    println!("sim_version {}", salient_sim::SIM_VERSION);
    println!("seed {seed} scenario {} units {units} ticks {ticks}", scenario.name());
    while sim.tick() <= ticks {
        if sim.tick().is_multiple_of(every) {
            println!("{:>8} {:016x}", sim.tick(), sim.fingerprint());
        }
        sim.step(&[]);
    }
    ExitCode::SUCCESS
}

fn parse<T: std::str::FromStr>(v: Option<&String>, what: &str) -> T {
    v.and_then(|s| s.parse().ok()).unwrap_or_else(|| fail(&format!("{what} expects a number")))
}

fn fail(msg: &str) -> ! {
    eprintln!("fingerprint: {msg}");
    std::process::exit(2)
}

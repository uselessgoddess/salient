//! Assert the simulation's dependency ban mechanically.
//!
//! Principle II says the simulation depends on no engine, renderer, network, async runtime or
//! clock. A rule a human has to remember is discipline; a rule the build checks is
//! construction. This walks the resolved dependency graph, so a ban cannot be broken by a
//! transitive dependency that nobody noticed.

use std::process::{Command, ExitCode};

/// One representative crate per banned category. A denylist rather than an allowlist because
/// the point is to name the categories the constitution names, not to police every transitive
/// dependency of a serialization library.
const BANNED: &[(&str, &str)] = &[
    ("bevy", "engine"),
    ("wgpu", "renderer"),
    ("winit", "windowing"),
    ("tokio", "async runtime"),
    ("async-std", "async runtime"),
    ("smol", "async runtime"),
    ("quinn", "network"),
    ("mio", "network"),
    ("socket2", "network"),
    ("reqwest", "network"),
    ("hyper", "network"),
    ("rayon", "work-stealing parallelism"),
    ("chrono", "clock"),
    ("time", "clock"),
    ("web-time", "clock"),
];

fn main() -> ExitCode {
    let out = match Command::new(env!("CARGO"))
        .args(["tree", "-p", "salient-sim", "--edges", "normal", "--prefix", "none", "--no-dedupe"])
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            eprintln!("deps-check: cargo tree failed:\n{}", String::from_utf8_lossy(&o.stderr));
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("deps-check: could not run cargo tree: {e}");
            return ExitCode::FAILURE;
        }
    };

    let text = String::from_utf8_lossy(&out);
    let names: Vec<&str> = text
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|n| !n.is_empty() && *n != "(*)")
        .collect();

    let mut found = Vec::new();
    for (crate_name, category) in BANNED {
        if names.iter().any(|n| n == crate_name) {
            found.push((*crate_name, *category));
        }
    }

    if found.is_empty() {
        let mut unique: Vec<&str> = names.clone();
        unique.sort_unstable();
        unique.dedup();
        println!("deps-check: {} crates resolved, none banned", unique.len());
        return ExitCode::SUCCESS;
    }

    eprintln!("deps-check: salient-sim must not depend on any of these (Principle II):");
    for (crate_name, category) in found {
        eprintln!("  {crate_name}  ({category})");
    }
    ExitCode::FAILURE
}

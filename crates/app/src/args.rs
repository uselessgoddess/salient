//! Command line.

use std::process;

use bevy::prelude::Resource;
use salient_sim::{Rules, Scenario};

#[derive(Clone, Debug, Resource)]
pub struct Args {
    pub seed: u64,
    pub scenario: Scenario,
    pub units: u32,
    pub rules: Rules,
    pub replay: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            seed: 1,
            scenario: Scenario::Drift,
            units: 512,
            rules: Rules::default(),
            replay: None,
        }
    }
}

impl Args {
    pub fn parse() -> Args {
        let mut a = Args::default();
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            let next = argv.get(i + 1);
            match argv[i].as_str() {
                "--seed" => a.seed = num(next, "--seed"),
                "--units" => a.units = num(next, "--units"),
                "--scenario" => {
                    a.scenario = next
                        .and_then(|s| Scenario::parse(s))
                        .unwrap_or_else(|| fail("--scenario expects empty or drift"))
                }
                "--replay" => {
                    a.replay = Some(next.unwrap_or_else(|| fail("--replay expects a path")).clone())
                }
                "--help" | "-h" => {
                    println!("salient [--seed N] [--units N] [--scenario NAME] [--replay PATH]");
                    process::exit(0)
                }
                other => fail(&format!("unknown argument {other}")),
            }
            i += 2;
        }
        a
    }
}

fn num<T: std::str::FromStr>(v: Option<&String>, what: &str) -> T {
    v.and_then(|s| s.parse().ok()).unwrap_or_else(|| fail(&format!("{what} expects a number")))
}

fn fail(msg: &str) -> ! {
    eprintln!("salient: {msg}");
    process::exit(2)
}

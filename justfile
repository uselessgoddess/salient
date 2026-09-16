# Every routine command lives here. A command that exists only in a shell history or only
# inside a CI workflow is not a command this project has.

set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

test_runner := "cargo nextest run"
seed := "1"
ticks := "6000"

default:
    @just --list

# --- build -------------------------------------------------------------------

check:
    cargo check --workspace --all-targets

build *args:
    cargo build --workspace {{ args }}

fmt:
    cargo fmt --all

# --- quality -----------------------------------------------------------------

# Two passes: the workspace, then the simulation with its own stricter config. The
# determinism bans are correct for salient-sim and wrong for salient-app, which is built
# on an f32 engine, so they cannot live at the workspace root.

# Lint everything, then lint the simulation again under its determinism bans.
lint: lint-workspace lint-sim

lint-workspace *args:
    cargo clippy --workspace --all-targets {{ args }} -- -D warnings

[unix]
lint-sim *args:
    CLIPPY_CONF_DIR=crates/sim cargo clippy -p salient-sim --all-targets {{ args }} -- -D warnings

[windows]
lint-sim *args:
    $env:CLIPPY_CONF_DIR="crates/sim"; cargo clippy -p salient-sim --all-targets {{ args }} -- -D warnings

test *args:
    {{ test_runner }} --workspace --all-targets {{ args }}

bench *args:
    cargo bench -p salient-sim {{ args }}

# Run before every commit.
pre: fmt lint test

# --- determinism -------------------------------------------------------------

# Run this on every target platform and compare. If the three disagree, nothing built
# on top of the simulation is worth building.

# Print state fingerprints for a seed.
print-fingerprint seed=seed ticks=ticks:
    cargo run -q -p salient-tools --bin fingerprint -- --seed {{ seed }} --ticks {{ ticks }}

# Assert salient-sim resolves no engine, renderer, network, async or clock crate.
deps:
    cargo run -q -p salient-tools --bin deps-check

# Assert no authored art assets exist anywhere. Everything visible is generated.
[unix]
no-assets:
    @! find . -type d -name assets -not -path './target/*' | grep . || (echo "assets/ found; see Principle V" && exit 1)

[windows]
no-assets:
    @if (Get-ChildItem -Recurse -Directory -Filter assets -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch 'target' }) { Write-Error "assets/ found; see Principle V"; exit 1 }

# The full gate. What CI runs.
verify: lint test deps no-assets

# --- run ---------------------------------------------------------------------

run *args:
    cargo run -p salient-app -- {{ args }}

drift seed=seed:
    cargo run -p salient-app -- --seed {{ seed }} --scenario drift

replay file:
    cargo run -p salient-app -- --replay {{ file }}

clean:
    cargo clean

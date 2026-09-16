set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

test_runner := "cargo nextest run"
seed := "1"
ticks := "6000"

default:
    @just --list

# Headers the Bevy shell links against. The simulation needs none of this.
[linux]
setup:
    #!/usr/bin/env bash
    set -eu
    if command -v pacman >/dev/null; then
        sudo pacman -S --needed --noconfirm wayland libxkbcommon alsa-lib systemd-libs
    elif command -v apt-get >/dev/null; then
        sudo apt-get update
        sudo apt-get install -y libwayland-dev libxkbcommon-dev libasound2-dev libudev-dev
    else
        echo "install the wayland, libxkbcommon, alsa and udev headers by hand" >&2
        exit 1
    fi

[macos]
setup:
    @echo "nothing to install"

[windows]
setup:
    @echo "nothing to install"

check:
    cargo check --workspace --all-targets

build *args:
    cargo build --workspace {{ args }}

fmt:
    cargo fmt --all

# --- quality -----------------------------------------------------------------

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

pre: fmt lint test

# --- determinism -------------------------------------------------------------

print-fingerprint seed=seed ticks=ticks:
    cargo run -q -p salient-tools --bin fingerprint -- --seed {{ seed }} --ticks {{ ticks }}

deps:
    cargo run -q -p salient-tools --bin deps-check

verify: lint test deps

run *args:
    cargo run -p salient-app -- {{ args }}

drift seed=seed:
    cargo run -p salient-app -- --seed {{ seed }} --scenario drift

replay file:
    cargo run -p salient-app -- --replay {{ file }}

clean:
    cargo clean

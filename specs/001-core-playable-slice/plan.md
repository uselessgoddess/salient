# Implementation Plan: Core Playable Slice

**Branch**: `001-core-playable-slice` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-core-playable-slice/spec.md`

## Summary

Build Salient's first playable slice as nine increments, each ending in a build that runs and can be
played with, and each posing one question the author answers by hand before the next begins. Three
of those checkpoints are design gates, deliberately placed early relative to their cost.

The simulation is an engine-free Rust crate: fixed-point Q32.32 arithmetic, dense arenas, a ten-hertz
tick, and a single entry point that takes orders and advances state. Determinism is enforced by lint
and proved by a golden recording replayed on three platforms from the first increment onward. Bevy
0.19 is a shell around it that reads state and draws — everything visible is generated, including the
terrain contours, the water hatch, and every unit glyph.

Two decisions carry disproportionate weight and are argued in [research.md](./research.md): supply is
solved by pressure diffusion rather than a flow algorithm (R8), which makes allocation stable and
order-independent almost for free; and all glyphs are drawn as one dynamic mesh rebuilt at simulation
rate with interpolation done in the vertex shader (R9), which is what puts twenty thousand units
within reach of the renderer.

## Technical Context

**Language/Version**: Rust nightly, pinned to an exact date in `rust-toolchain.toml`. Edition 2024.

**Primary Dependencies**: `salient-sim` — `rkyv` 0.8 with `little_endian` and `rustc-hash`, and
nothing else; `Fx` is implemented in-project. `salient-app` — `bevy` 0.19. `salient-tools` — nothing
beyond the simulation.

**Storage**: Files only. Recordings (`seed` + orders + fingerprint checkpoints) and state snapshots,
both `rkyv`. No database, no network persistence.

**Testing**: `cargo test`, with a golden replay test as the keystone; `criterion` for the tick
budget; a dependency-graph check asserting the simulation pulls in no engine crate.

**Target Platform**: Desktop — Linux, Windows, macOS. All three must produce identical fingerprints.

**Project Type**: Rust workspace — an engine-free simulation library, an engine shell, and tools.

**Performance Goals**: Simulation 10 Hz with a per-tick budget of 10 ms at 20,000 units, a tenfold
margin inside the 100 ms a tick actually has. Rendering 60+ fps, interpolated between ticks.

**Constraints**: Bit-identical determinism across platforms and profiles; no floating point in the
simulation; no authored art assets anywhere; `overflow-checks = true` in every profile; no `unsafe`
in the simulation.

**Scale/Scope**: 20,000 units on a 40 x 40 km map. Roughly twelve unit classes plus the commander.
Nine increments.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design — see bottom.*

| Principle | Status | How it is enforced in this plan |
|---|---|---|
| I. Additive Architecture | PASS with one justified exception | Every increment M1–M8 leaves a playable game; each maps to one user story. M0 is the exception, tracked below. |
| II. Isolated Simulation Core | PASS | `salient-sim` has no engine, renderer, network, async, or clock dependency. The whole public surface is fixed in [contracts/sim-api.md](./contracts/sim-api.md) and the ban is checked in CI against the resolved dependency graph, not by review. |
| III. Determinism by Construction | PASS | `Fx` newtype over Q32.32 with `i128` intermediates and a separate wide type for squared magnitudes (R2); in-project transcendentals (R3); `clippy.toml` bans at `deny` (R4); golden replay across three platforms from M0 (quickstart). |
| IV. Parallelism by Disjointness | PASS | No parallelism until M8, and only then in disjoint-write phases or integer reductions (R13). Deferring costs nothing because the principle already constrains how it may be added. |
| V. Code Over Assets | PASS | Contours from marching squares, water hatch generated in the fragment shader, glyphs from primitives (R9, R10). CI asserts no asset directory exists. |
| VI. Few Sharp Tests | PASS with one justified exception | One integration test per increment at the widest boundary that still fails informatively, plus the golden replay covering the whole simulation in a single assertion. The fixed-point primitives are the exception, tracked below. |
| VII. Terse Naming | PASS | Module paths carry meaning; test names are verb-first and at most three words. Reviewed, not automated. |

No unjustified violations. Proceeding.

## Project Structure

### Documentation (this feature)

```text
specs/001-core-playable-slice/
├── plan.md              # This file
├── spec.md              # What the slice must do
├── research.md          # Phase 0 — the decisions and why
├── data-model.md        # Phase 1 — simulation state
├── quickstart.md        # Phase 1 — how to validate each increment by hand
├── contracts/
│   ├── sim-api.md       # The simulation's entire public surface and its guarantees
│   ├── orders.md        # The only input to the simulation
│   └── replay-format.md # Recordings, snapshots, fingerprints
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 — created by /speckit-tasks, not here
```

### Source Code (repository root)

```text
Cargo.toml                    # workspace
rust-toolchain.toml           # pinned nightly
clippy.toml                   # disallowed types and methods for the simulation
justfile                      # every routine command lives here
crates/
├── sim/                      # salient-sim — no engine, no clock, no net, no float
│   ├── src/
│   │   ├── fx/               # Fx, Fx2, sqrt, atan2, sin, cos, generated tables
│   │   ├── arena/            # dense arenas, generational handles
│   │   ├── map/              # seeded generation, height, passability, cost
│   │   ├── motion/           # flow fields, integration, local avoidance
│   │   ├── space/            # uniform grid broad phase
│   │   ├── supply/           # stocks, links, diffusion solver
│   │   ├── build/            # engineers, factories, queues
│   │   ├── combat/           # weapons, damage, wreckage, reclaim
│   │   ├── intel/            # sensors, contacts, uncertainty
│   │   ├── command/          # orders, validation, groups, standing orders
│   │   ├── rules/            # victory predicates, match rules
│   │   ├── front/            # the hostile force
│   │   └── record/           # rkyv state, fingerprint, recordings
│   ├── tests/                # golden replay lives here
│   └── benches/              # tick budget
├── app/                      # salient-app — the Bevy 0.19 shell
│   └── src/
│       ├── view/             # camera, zoom, level of detail
│       ├── draw/             # glyph mesh builder, contours, water, flow lines
│       ├── shader/           # wgsl: glyph interpolation, procedural hatch
│       ├── input/            # selection, order issuing
│       └── dev/              # developer overlays, behind a feature
└── tools/                    # salient-tools — fingerprint, deps-check, replay diff
```

**Shell module convention**: each module in `salient-app` exposes `pub fn plugin(app: &mut App)`
and is registered through `add_plugins`, which is the convention the official Bevy template settled
on and is less ceremony than a plugin struct per module. Composition uses required components, not
bundles — bundles were superseded in Bevy 0.15 and a year-old codebase is the main place they
survive. One-shot reactions use observers rather than event-reader plumbing.

The modules are named for what the shell does — view, draw, shader, input, dev — rather than for
game concepts, because the game is not in this crate. There is deliberately no `core` or `utils`
module: those are where a shell accumulates whatever did not fit, and Principle VII bans the names
for exactly that reason.

**Structure Decision**: Three crates, and the split is not organisational — it is the enforcement
mechanism for Principle II. `salient-sim` cannot accidentally reach for a clock or a renderer because
neither is in its dependency graph, and CI fails if one appears. `salient-tools` exists so that
cross-platform fingerprint comparison and dependency checking are runnable commands rather than
manual procedures, which is what makes the M0 checkpoint enforceable.

## Increment Plan

Each increment is specified by **what it guarantees**, not by the functions it should contain.
Downstream increments depend only on those guarantees, so hand edits between checkpoints do not
invalidate the plan (R14). Checkpoint procedures are in [quickstart.md](./quickstart.md).

| # | Delivers | Guarantees for later increments | Story | Checkpoint |
|---|---|---|---|---|
| **M0** | Workspace, `Fx` and its math, arenas, tick loop, `rkyv` state and fingerprint, recordings, minimal glyph draw with interpolation | The sim-api contract holds; fingerprints match on three platforms; a recording replays | — | Drifting glyphs; cross-platform fingerprint. Weak by design — see Complexity Tracking |
| **M1** | Seeded map generation, contours, water and hatch, the glyph grammar, zoom and level of detail | A map exists and is readable; every domain has a visual identity | US1 | **GATE** — does the visual language work? |
| **M2** | Flow-field pathing, terrain cost per domain, broad-phase grid, selection, move orders | Units go where ordered; spatial queries exist for everything downstream | US1 | 500 units across broken terrain |
| **M3** | Resource sources, stocks, links, diffusion solver, engineers, factories, flow display, the gradient dial | The economy exists and is tunable from one value | US2 | **GATE** — is the supply network interesting or tedious? |
| **M4** | Weapons, damage, death, wreckage, reclaim | Combat exists; the battlefield is a resource landscape | US3 | Build in the wreckage of a battle |
| **M5** | The hostile force, commander, victory conditions as rules, match settings, save and resume | A match starts, is won or lost, and replays to the same ending | US4 | First real game — win one, lose one |
| **M6** | Transports, cargo, forward supply points | Operating away from the network is possible and rewarded | US5 | Do a drop; then shoot down the resupply |
| **M7** | Domain-specific sensors, contacts, age, growing uncertainty | The player sees an intelligence picture, not truth | US6 | **GATE** — is fighting blind tense or annoying? |
| **M8** | Predicate selection, rule-based groups, standing orders, vertex-shader interpolation deferred from M0 (R9), scale-up and any parallelism it needs | 20,000 units are both simulable and commandable | US7 | Three fronts for ten minutes; tick budget benchmark |

**Ordering rationale.** Risk is front-loaded against cost, not against dependency. M1 proves the
entire art strategy in the second increment because a "no" there costs weeks, while the same "no"
discovered at M7 costs the project. M3 answers the design's long-standing open question — whether the
supply network is fun — while the answer is still a tuning value rather than a rewrite. M8 is last
because scale is the one risk that degrades gracefully: falling short means a smaller number, not a
broken game.

**Cutting the plan short is a valid outcome.** M5 is already a complete game. Everything after it is
an additive layer, which is exactly what Principle I promises and the point at which that promise
becomes testable.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| M0 does not leave a playable game, against Principle I | Nothing can be rendered before there is something to render, and the cross-platform determinism proof must come before code is written on top of it | Folding M1 and M2 into M0 to make the first increment playable would push the first checkpoint from roughly two weeks to six, and would delay the determinism proof past the point where a failure is cheap to fix. The exception is bounded: it applies to M0 alone |
| The fixed-point primitives get per-function tests, against Principle VI's preference for one wide test | There is no wider behaviour to test them through — a wrong bit in `atan2` surfaces as a desync a month later in a system that looks unrelated. These functions are the base case of the whole determinism argument | Testing them only through the golden replay was rejected: the replay would catch the error but would point at the symptom rather than the cause, which defeats the purpose of having a single sharp test |

## Post-Design Constitution Re-check

Re-evaluated after Phase 1. No new violations introduced.

Two design choices deserve a note because they strengthen rather than strain the principles. The
diffusion solver (R8) satisfies FR-018's order-independence requirement structurally, by computing
all transfers from a tick-start snapshot, rather than by imposing an ordering convention that
someone would later have to remember — which is Principle III's construction-over-discipline argument
applied to a system that is not about arithmetic. And the single-dynamic-mesh renderer (R9) keeps
twenty thousand units out of the engine's entity system entirely, which means Principle II's boundary
is holding up under the scale target rather than merely being declared.

One item is carried forward rather than resolved: R13 defers all parallelism to M8. If the M8
benchmark shows the budget is threatened in phases whose writes are not disjoint, that is the one
place where Principle IV could force a structural change rather than an additive one. It is recorded
here so that it is a known risk rather than a surprise.

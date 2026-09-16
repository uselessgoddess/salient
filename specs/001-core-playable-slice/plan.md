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

**Scale/Scope**: 20,000 units on a 40 x 40 km map, terrain on one grid of 2048 x 2048 cells at
19.53 m to a cell (R16). Roughly twelve unit classes plus the commander. Nine increments.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design — see bottom.*

*Checked against constitution v1.7.0. Rows VIII and IX were added on 2026-09-17: the constitution
has moved twice since this table was first written, gaining comment discipline in 1.2.0 and the
constants-and-paths rule in 1.7.0.*

| Principle | Status | How it is enforced in this plan |
|---|---|---|
| I. Additive Architecture | PASS with one justified exception | Every increment M1–M8 leaves a playable game; each maps to one user story. M0 is the exception, tracked below. |
| II. Isolated Simulation Core | PASS | `salient-sim` has no engine, renderer, network, async, or clock dependency. The whole public surface is fixed in [contracts/sim-api.md](./contracts/sim-api.md) and the ban is checked in CI against the resolved dependency graph, not by review. |
| III. Determinism by Construction | PASS | `Fx` newtype over Q32.32 with `i128` intermediates and a separate wide type for squared magnitudes (R2); in-project transcendentals (R3); `clippy.toml` bans at `deny` (R4); golden replay across three platforms from M0 (quickstart). |
| IV. Parallelism by Disjointness | PASS | No parallelism until M8, and only then in disjoint-write phases or integer reductions (R13). Deferring costs nothing because the principle already constrains how it may be added. |
| V. Code Over Assets | PASS | Contours, water and hatch are all generated in the fragment shader from a height texture built at run time from the seed; glyphs from primitives (R9, R10). The terrain layer loads nothing at all. The CI asset-directory check was removed in `5a712e6`: an `assets/` directory is visible in the diff that adds it, and the workflow section bans a gate that restates a rule already plain in a hand-edited file. |
| VI. Few Sharp Tests | PASS with one justified exception | One integration test per increment at the widest boundary that still fails informatively, plus the golden replay covering the whole simulation in a single assertion. The fixed-point primitives are the exception, tracked below. |
| VII. Terse Naming | PASS | Module paths carry meaning; test names are verb-first and at most three words. Reviewed, not automated. |
| VIII. Terse Comments | PASS | Rationale for every decision lives in [research.md](./research.md), in this plan, or in the commit that makes the change — never restated in source. The `why` cap is three lines per item; contract documentation on the public surface is exempt and lives in [contracts/](./contracts/). |
| IX. Constants and Paths | PASS | Every tuning value M1 introduces — contour base interval, water fraction, slope threshold, level-of-detail bands, palette tokens — is a named constant on the type that owns it, not a loose `const` at module root. Reviewed, not automated. |

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
│       ├── shader/           # wgsl: glyph interpolation, contours, procedural hatch
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

## M1 Design

*Added 2026-09-17, after the M1 clarification session. This expands one row of the table above; the
increments around it are unchanged.*

M1 delivers seeded map generation, terrain and water display, the glyph grammar, zoom and level of
detail — and then asks whether the visual language works. Everything here is shaped by the fact that
it is a gate: the increment is worth building cheaply, because a "no" means unbuilding it.

### Where the work lands

```text
crates/sim/src/map/
├── mod.rs        # Map, the 2048 grid, cell <-> world conversion
├── height.rs     # fBm over integer value noise (R17)
├── terrain.rs    # sea level from the height histogram, slope, passability (R18)
├── cost.rs       # per-domain movement cost
└── sources.rs    # start in the largest land component, flood fill, source placement (R18)

crates/app/src/draw/
├── terrain.rs    # the height texture and the single quad it is sampled on (R10)
├── palette.rs    # named display tokens
├── shape.rs      # glyph geometry per domain
├── layer.rs      # draw order and the air offset (R11)
├── counter.rs    # formation counters (R19)
└── lod.rs        # zoom bands

crates/app/src/shader/terrain.wgsl   # contours, index contours, water threshold, hatch (R10)
crates/app/src/view/camera.rs        # zoom and pan
```

### Order of work, and why

1. **The simulation side first.** Nothing can be drawn before there is a map, and this is the half
   that has to be deterministic. It ends at a test: one seed, one fingerprint, across runs (T051).
2. **The terrain shader next.** It is the largest single risk in the increment — if contours do not
   read at 40 km, the gate fails on the substrate rather than on the glyphs, and that is worth
   finding out before any glyph exists.
3. **The glyph grammar third**, with the draw order that carries domain (R11).
4. **Camera, level of detail, counters last.** They are the cheapest to change and the most likely
   to be changed by looking.

### What is decided, and what is left to taste

Decided, with the argument recorded in research: grid resolution and height representation (R16),
the noise family (R17), how sea level and passability are derived and why source placement cannot
fail (R18), how contours and water are drawn (R10), and that counters overlay units rather than
replace them (R19).

Left to taste, and settled by looking at the screen rather than by argument: the base contour
interval in metres, the target water fraction, the slope threshold that separates land from cliff,
the level-of-detail band boundaries, glyph sizes, and every palette value. These are tuning values,
not decisions. Each is a named constant on the type that owns it (Principle IX), so moving one is a
one-line edit at a known address rather than a search.

### The determinism boundary in this increment

Everything under `crates/sim/src/map/` is state: it is serialised, fingerprinted, and identical on
three platforms. Everything under `crates/app/src/draw/` is derived from it and is none of those
things — which is why a clustering rule for counters may be chosen for how it looks.

One value crosses the boundary and must not be duplicated: `sea_level`. The simulation derives it,
the shader thresholds on it, and if the two disagree the waterline in the picture stops matching
where ships can go. It is generated state, read by the display, never re-derived there.

### What M1 does not do

No routing and no terrain-aware movement — units still head straight at a destination, and the flow
field that will change that is M2's, along with the coarser structure it needs (R16). No selection
and no orders issued against terrain. No echelon grouping and no view toggle (R19). No parallelism
in generation, though R17 is written so it can be sliced later without changing a bit.

### Task list impact

Applied to [tasks.md](./tasks.md) on 2026-09-17. Three tasks were removed and three added, so the
M1 block still occupies T042 through T059 and nothing outside it was renumbered.

| Was | Now | Change |
|---|---|---|
| — | T042 | New. `Map` and the grid itself had no task; the fields were being written before the type. |
| T042 | T043 | Reworded to name the generator: fBm over integer value noise, `i16` metres, per-cell pure (R17). |
| T043 | T044 | Reworded. Sea level is derived from the height histogram rather than assumed, and slope joins it (R18). |
| T044 | T045 | Unchanged. |
| — | T046 | New. Start in the largest land component, then the reachable flood fill (R18). This is what T047 draws from. |
| T045 | T047 | Reworded. Sources are drawn from the reachable set, which is what removes the failure path. |
| T046 | — | **Gone.** There are no seeds to reject (R18); `map/validate.rs` is never written. |
| T047 | T048 | Unchanged apart from `SimState` being spelled `State` as the code spells it. |
| — | T049 | New. Hash the static terrain once instead of re-serialising 15 MB into every fingerprint — 1.10 ms a measurement at this resolution otherwise (R16). |
| T048 | T050 | Unchanged. |
| T049 | T051 | Unchanged. |
| T050 | — | **Gone.** No marching squares, no contour line mesh (R10). |
| T051 | — | **Gone.** Water needs no region mesh; the shader thresholds the same texture (R10). |
| T053 | T052 | Becomes the height-texture upload and the terrain quad. Constant on-screen line weight stops being a task and becomes a property of the technique. |
| T052 | T053 | Widened from the hatch alone to the whole terrain shader: contours, index contours, water threshold, hatch. |
| T054 | T054 | Reworded. The four domain outlines already landed in M0 — `draw/shape.rs` exists — so what remains is tier markings and their place in the grammar. |
| T055, T056 | T055, T056 | Unchanged. |
| T057 | T057 | Reworded. `view/camera.rs` exists from M0 with zoom and pan; the work is extending the range to reach a single unit. |
| T058 | T058 | Reworded. Bands change what is *added*, not what is swapped: glyphs stay drawn at every zoom with a size floor. |
| T059 | T059 | Becomes formation counters in their own `draw/counter.rs`, per R19. |

Outside the M1 block: T060 widened to cover display tuning constants generally, since the M0 camera
and glyph radius hold loose module-root literals that Principle IX now speaks to. T061, T063 and
T064 are marked done — the benchmark, the fingerprint-comparison job and the dependency check all
exist — with T064 reworded, because the asset-directory half was deleted in `5a712e6`. T067 is new:
the Bevy Linux system libraries, which is why the workspace lint currently fails on the Linux
runner.

### The risk that matters

The gate says no. That is a designed outcome, not a failure — but it is only survivable if M1 stays
small, which is the reason R10's removal of the whole terrain geometry pipeline is worth more than
the code it saves. The second risk is quieter: five observers are needed, and they are the one
input to this increment that cannot be produced by writing code. Line them up before the screenshot
is ready, not after.

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

**Re-checked again 2026-09-17, after the M1 design above.** No new violations. Three notes:

Revising R10 from a geometry pipeline to a shader strengthened Principle V rather than straining it.
The height texture is generated at run time from the seed, so the terrain layer now contains no
loaded data of any kind — not even a mesh built at load and cached.

Principle IV is satisfied in advance rather than deferred here. R17 makes every height cell a pure
function of its coordinates and the seed, which means generation can later be split into disjoint
slices without changing a bit. Nothing is parallelised now, and nothing has to be unpicked to
parallelise it later.

A second carried item joins R13. R16 records a measured 27.5 ms for a flow field swept over the full
terrain grid, against a 10 ms tick target. That is not a problem for M1, which does no routing, and
it is not an argument for a coarser grid — but it does mean M2 opens with a structural question
already posed and already costed, rather than discovering it during implementation.

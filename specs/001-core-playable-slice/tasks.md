---

description: "Task list for increments M0 and M1 of the core playable slice"
---

# Tasks: Core Playable Slice — M0 and M1

**Input**: Design documents from `/specs/001-core-playable-slice/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/](./contracts/)

**Scope**: This list covers **M0 and M1 only** — the first two of the nine increments in
[plan.md](./plan.md). M2 through M8 get their own task lists, generated after the M1 gate is
answered. Generating all nine now would be a list nobody can hold, and the M1 gate may change what
comes after it.

**Tests**: Included, but only the sharp ones. Principle VI of the constitution prefers one test that
proves a system over twenty that restate its functions, so this list carries the golden replay test,
the fixed-point reference tests, and the map determinism test — and nothing else.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Independent of other tasks in the same phase — different files, no ordering constraint
- **[Story]**: Which user story the task serves
- Exact file paths are given for every task

## Path Conventions

Rust workspace as laid out in [plan.md](./plan.md): `crates/sim` (engine-free simulation),
`crates/app` (Bevy shell), `crates/tools` (command-line utilities).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The workspace, and the machinery that enforces the constitution mechanically rather
than by memory.

- [X] T001 Create the workspace with members `crates/sim`, `crates/app`, `crates/tools` in `Cargo.toml`
- [X] T002 [P] Pin an exact nightly toolchain date in `rust-toolchain.toml`
- [X] T003 [P] Declare the determinism bans as `disallowed-types` and `disallowed-methods` — `f32`, `f64`, `std::collections::HashMap`, `std::collections::HashSet`, `std::time::Instant`, `std::time::SystemTime`, `rand` convenience entry points — in `clippy.toml`
- [X] T004 Add the workspace lints table and set `overflow-checks = true` on every profile including release in `Cargo.toml`
- [X] T005 [P] Declare `salient-sim` with only `rkyv` (feature `little_endian`) and `rustc-hash`, plus `#![forbid(unsafe_code)]`, in `crates/sim/Cargo.toml`
- [X] T006 [P] Declare `salient-app` with `bevy` 0.19 and `salient-sim` in `crates/app/Cargo.toml`
- [X] T007 [P] Declare `salient-tools` with `salient-sim` in `crates/tools/Cargo.toml`
- [X] T008 Write the justfile with recipes for check, lint, fmt, test, bench, run, and the determinism checks, so every routine command is discoverable with `just --list`, in `justfile`
- [X] T009 Add a CI workflow running fmt, `clippy -D warnings`, and tests on Linux, Windows, and macOS in `.github/workflows/ci.yml`

---

## Phase 2: Foundational — Increment M0 (Blocking Prerequisites)

**Purpose**: The simulation skeleton and the proof that it is deterministic. Everything in every
later increment sits on this.

**⚠️ CRITICAL**: No user story work begins until the cross-platform fingerprint check at the end of
this phase passes. A determinism failure discovered here costs hours; the same failure discovered
after M3 costs weeks.

### Fixed-point arithmetic

- [X] T010 [P] Implement `Fx` as Q32.32 over `i64` in-project — addition, subtraction, negation, comparison, and multiplication and division through `i128` intermediates — in `crates/sim/src/fx/mod.rs`
- [X] T011 [P] Implement `Fx2`, the wide type for squared magnitudes, with no conversion path that truncates to `i64`, in `crates/sim/src/fx/wide.rs`
- [X] T012 Implement integer Newton square root over `i128` returning `Fx` in `crates/sim/src/fx/sqrt.rs`
- [X] T013 [P] Pin the Q32.32 series coefficients and mathematical constants, generated once and written out, in `crates/sim/src/fx/consts.rs` (was: a compile-time lookup table; see research R3 for why the table was dropped)
- [X] T014 Implement `sin`, `cos`, and `atan2` over angles in turns by truncated series with octant reduction, in `crates/sim/src/fx/trig.rs`
- [X] T015 Implement `Vec2` over `Fx` with `len`, `len_sq` returning `Fx2`, `normalize`, and dot product, in `crates/sim/src/fx/vec.rs`
- [X] T016 Write reference tests pinning `sqrt`, `sin`, `cos`, and `atan2` against precomputed high-precision values in `crates/sim/src/fx/tests.rs`

### Storage and identity

- [X] T017 [P] Implement the generational `Handle` and the dense `Arena` with explicit index-order iteration and deterministic slot reuse, in `crates/sim/src/arena/mod.rs`

### Simulation core

- [X] T018 Define the `State` root holding arenas, tick, seed, generator, rules and scenario, in `crates/sim/src/state.rs`
- [X] T019 [P] Define `Order`, `OrderTarget`, and `OrderAction` exactly as fixed in `contracts/orders.md`, in `crates/sim/src/command/order.rs`
- [X] T020 [P] Define `MatchRules` with the victory condition set and the supply gradient value in `crates/sim/src/rules/mod.rs`
- [X] T021 Implement `Sim::validate` rejecting unknown handles, targets not owned by the issuer, and actions the target class does not support, in `crates/sim/src/command/validate.rs`
- [X] T022 [P] Define `StepReport` carrying units created and destroyed, orders rejected, and conditions met, in `crates/sim/src/report.rs`
- [X] T023 Implement `Sim` with `new`, `step`, `state`, `tick`, `fingerprint`, `snapshot`, `restore`, and `SIM_VERSION`, exposing nothing else, in `crates/sim/src/lib.rs`

### Serialization, fingerprints, recordings

- [X] T024 Derive `rkyv` on `SimState` and implement snapshot and restore in `crates/sim/src/record/snapshot.rs`
- [X] T025 Implement the fingerprint as a `rustc-hash` digest over the `rkyv` bytes in `crates/sim/src/record/fingerprint.rs`
- [X] T026 Implement the recording header, order list, and checkpoint list with read and write per `contracts/replay-format.md`, in `crates/sim/src/record/recording.rs`
- [X] T027 Implement `SIM_VERSION` mismatch refusal with an explanatory error rather than best-effort playback, in `crates/sim/src/record/mod.rs`

### Minimal motion, so M0 has something to look at

- [X] T028 [P] Define the `Unit` arena entry with position, velocity, facing, domain, and owner in `crates/sim/src/unit.rs`
- [X] T029 Implement the velocity integration phase advancing every unit once per tick in `crates/sim/src/motion/integrate.rs`
- [X] T030 Implement the `drift` scenario spawning a mixed-domain force with constant velocities in `crates/sim/src/scenario.rs`

### The keystone test

- [X] T031 Write the golden replay test asserting that a seed plus a recorded order log reaches an expected fingerprint at every checkpoint, in `crates/sim/tests/replay.rs`

### Engine shell

- [X] T032 [P] Parse `--seed`, `--scenario`, `--units`, `--rules`, and `--replay` in `crates/app/src/args.rs`
- [X] T033 Build the Bevy app with the pale ground colour and a camera framing the whole world, in `crates/app/src/main.rs` and `crates/app/src/view/camera.rs`
- [X] T034 Implement the driver stepping the simulation at fixed cadence and publishing the interpolation alpha, never letting the renderer write to the simulation, in `crates/app/src/driver.rs`
- [X] T035 [P] Define the glyph shape primitives as vertex lists in `crates/app/src/draw/shape.rs`
- [X] T036 Build the glyph mesh from the shape primitives, one mesh per domain rather than one entity per unit, in `crates/app/src/draw/glyph.rs`
- [X] T037 Interpolate between the previous and current tick when building the mesh, taking the shorter arc for headings, in `crates/app/src/draw/glyph.rs` (the vertex-shader version is deferred to M8; see research R9)
- [X] T038 Give each domain its own mesh, material and draw order, lowest for subsurface and highest for air, in `crates/app/src/draw/glyph.rs`
- [X] T039 Wire the draw plugin to spawn the layers once and rebuild them each frame in `crates/app/src/draw/mod.rs`

### Tools that make the checkpoint enforceable

- [X] T040 [P] Implement the `fingerprint` binary printing checkpoint fingerprints for a seed and tick count in `crates/tools/src/bin/fingerprint.rs`
- [X] T041 [P] Implement the `deps-check` binary asserting `salient-sim` resolves no engine, renderer, network, async, or clock crate, in `crates/tools/src/bin/deps-check.rs`

**Checkpoint M0**: Run the M0 section of [quickstart.md](./quickstart.md). Glyphs drift smoothly
despite the ten-hertz tick; the replay test passes; the `fingerprint` tool prints identical values on
Linux, Windows, and macOS. **If the three platforms disagree, stop** — nothing built above a broken
determinism proof is worth building.

**Question to answer by hand**: is the `Fx` API workable, and is the sim/render boundary comfortable
to work across? This is the cheapest moment in the project to change either.

---

## Phase 3: User Story 1 — Read the battlefield (Priority: P1) 🎯 Increment M1

**Goal**: A generated map that reads as a staff tactical display, with all four domains
simultaneously legible and no view toggles anywhere.

**Independent Test**: Generate a map from a seed, spawn a mixed force covering all four domains, and
confirm from a static screenshot that an observer identifies every unit's domain without
interacting — then confirm the same seed reproduces the same map.

**Scope note**: User Story 1 also covers movement (acceptance scenario 4), which belongs to M2 and is
not in this list. Scenarios 1, 2, 3, and 6 are satisfied here; scenario 5 was satisfied in M0.

**Revised 2026-09-17** against the M1 clarifications and research R10 and R16 through R19. Three
tasks were removed and their numbers reused, so the block is still T042 through T059; see the
before-and-after mapping in [plan.md](./plan.md#task-list-impact).

### Map generation

- [ ] T042 [US1] Define `Map` over the 2048 x 2048 grid with cell-to-world conversion, keeping the grid constants on the type rather than at module root (R16), in `crates/sim/src/map/mod.rs`
- [ ] T043 [US1] Generate the seeded height field as fBm over integer value noise — four to six octaves, `i16` metres, every cell a pure function of `(x, y, seed)` with no floating point (R17) — in `crates/sim/src/map/height.rs`
- [ ] T044 [US1] Derive sea level from the height histogram at the target water fraction, then the water mask, the slope field, and the per-domain passability bitset (R18), in `crates/sim/src/map/terrain.rs`
- [ ] T045 [P] [US1] Derive the per-cell movement cost grid in `crates/sim/src/map/cost.rs`
- [ ] T046 [P] [US1] Place the player's start in the largest land-connected component and flood-fill the reachable set from it (R18) in `crates/sim/src/map/reach.rs`
- [ ] T047 [US1] Draw resource source positions from the reachable set with the seeded generator, so placement has no failure path and no retry (FR-005, R18), in `crates/sim/src/map/sources.rs`
- [ ] T048 [US1] Fold `Map` into `State` so terrain participates in the fingerprint in `crates/sim/src/state.rs`
- [ ] T049 [US1] Hash the static terrain once at generation and fold that digest into the per-tick fingerprint instead of re-serialising the grid every time (R16: 1.10 ms per measurement at this resolution otherwise), in `crates/sim/src/record/fingerprint.rs`
- [ ] T050 [US1] Implement the `showcase` scenario placing land units on high ground, ships in a bay, submarines directly beneath those ships, and aircraft crossing both, in `crates/sim/src/scenario.rs`
- [ ] T051 [US1] Write the map determinism test asserting one seed yields one fingerprint across runs in `crates/sim/tests/map.rs`

### Terrain and water display

- [ ] T052 [US1] Upload the height field as a 2048 x 2048 R16 texture once per match and draw the terrain on a single quad, passing `sea_level` through from the simulation rather than re-deriving it (R10), in `crates/app/src/draw/terrain.rs`
- [ ] T053 [US1] Write the terrain fragment shader — contour lines from the height texture with width taken from the screen-space derivative, a power-of-two interval derived from scale with adjacent intervals cross-faded and any interval past legibility faded out, every fifth line an index contour, water thresholded at sea level, and the depth-driven hatch (R10) — in `crates/app/src/shader/terrain.wgsl`

### Glyph grammar and domain separation

- [ ] T054 [P] [US1] Extend the glyph table with tier markings and reserve their position in the grammar — the four domain outlines already landed in M0 — in `crates/app/src/draw/shape.rs`
- [ ] T055 [US1] Implement the draw order that carries the meaning: subsurface before the water layer so the hatch crosses it, land and surface after it, air last, in `crates/app/src/draw/layer.rs`
- [ ] T056 [US1] Give air glyphs a zoom-scaled parallax offset and a soft ground shadow ellipse at their true position in `crates/app/src/draw/layer.rs`

### Zoom, level of detail, and formation counters

- [ ] T057 [P] [US1] Extend the M0 camera's zoom range to run from the whole 40 km map down to a single unit, keeping pan speed proportional to scale, in `crates/app/src/view/camera.rs`
- [ ] T058 [US1] Implement level-of-detail bands that add rather than swap: every unit glyph stays drawn at every zoom with a floor on its on-screen size, and a band governs what is drawn over them (R19), in `crates/app/src/draw/lod.rs`
- [ ] T059 [US1] Implement formation counters — shape from the dominant domain, colour from the owner, extent covering the ground the group holds — bucketed render-side and rebuilt at tick rate (R19), in `crates/app/src/draw/counter.rs`

**Checkpoint M1 — GATE**: Run the M1 section of [quickstart.md](./quickstart.md). Do the
self-checks first — same seed twice, zoom sweep, waterline against passability, every unit still
drawn when zoomed out. Then the gate itself, which you cannot run alone: five observers who know the
genre and have not seen the project, one still screenshot each, at most one wrong domain
identification across the whole sample (SC-003).

**Gate question**: does the visual language work? Everything else rests on the answer, which is why
it is being asked in the second increment. A "no" here costs weeks; the same "no" at M7 costs the
project. If the answer is no, revise the spec and re-plan from this point — reaching a gate and
turning back is the plan working, not failing.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Things that serve both increments and everything after them. None of these block the
M1 gate.

- [ ] T060 [P] Define the display palette and the display tuning constants as named tokens on the types that own them rather than as loose module-root literals — the M0 camera's zoom limits and the glyph radius are the existing examples (Principle IX) — in `crates/app/src/draw/palette.rs`
- [X] T061 [P] Add the tick budget benchmark, measuring now so later regressions have a baseline, in `crates/sim/benches/tick.rs`
- [ ] T062 [P] Implement the `replay-diff` binary bisecting divergence from checkpoint fingerprints in `crates/tools/src/bin/replay-diff.rs`
- [X] T063 Add the CI job comparing fingerprints produced by the three platform runners in `.github/workflows/ci.yml`
- [X] T064 Add the CI check running `deps-check` in `.github/workflows/ci.yml` — the asset-directory check originally paired with it was removed in `5a712e6`, because an `assets/` directory is visible in the diff that adds it and the workflow section bans a gate that restates a rule already plain in a hand-edited file
- [ ] T065 [P] Write build and run instructions in `README.md`
- [ ] T066 Run both checkpoint procedures in [quickstart.md](./quickstart.md) end to end and record the answers to the two hand questions
- [ ] T067 Install the Bevy Linux system libraries so the workspace lint builds the shell on all three CI runners, as a `just setup` recipe rather than a step that exists only inside the workflow, in `justfile` and `.github/workflows/ci.yml`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2, M0)**: depends on Setup. **Blocks everything.**
- **User Story 1 (Phase 3, M1)**: depends on the M0 checkpoint passing, including the cross-platform
  fingerprint comparison
- **Polish (Phase 4)**: independent of the M1 gate; may be done at any point after Phase 2

### Within Phase 2

- T010 and T011 come before everything else in the phase — all arithmetic depends on them
- T012 through T015 depend on T010 and T011; T014 additionally depends on T013
- T016 depends on T012 through T015
- T018 depends on T017; T021 depends on T018 and T019; T023 depends on T018, T021, T022
- T024 through T027 depend on T018; T025 depends on T024
- T029 depends on T015 and T028; T030 depends on T029
- T031 depends on T023, T026, and T030 — it is the phase's exit criterion
- T034 depends on T023; T036 depends on T035 and T034; T038 depends on T037; T039 depends on T036 and T038
- T040 depends on T025; T041 depends only on Phase 1

### Within Phase 3

- T043 depends on T042; T044 depends on T043; T045 and T046 depend on T044; T047 depends on T046
- T048 depends on T042 through T047; T049, T050 and T051 depend on T048
- T052 depends on T048 — it reads `height` and `sea_level`; T053 depends on T052
- T055 depends on T053 and T054; T056 depends on T055
- T058 depends on T057; T059 depends on T058

### Parallel Opportunities

Marked `[P]` tasks touch different files and have no ordering constraint within their phase.

- **Phase 1**: T002, T003, T005, T006, T007 after T001; T008 and T009 independently
- **Phase 2**: T010 with T011; then T013 with T017, T019, T020, T022, T028; then T032, T035, T037, T040, T041
- **Phase 3**: T045 with T046 once T044 lands; T054 and T057 at any point, since neither touches the
  simulation; T050 with T051 once T048 lands
- **Phase 4**: T060, T062, T065

Note that `[P]` here means "safe to reorder or hand to a parallel agent", not "needs a second
developer". This is a solo project; the value of the marker is that it identifies what can be
batched without thinking about order.

---

## Implementation Strategy

### The two increments are not equal in kind

M0 produces **no playable game** — it produces drifting glyphs and a determinism proof. This is the
one acknowledged exception to Principle I in the plan, and it is bounded to this increment. Its real
output is the cross-platform fingerprint match, which is why T040 and T041 exist as tools rather than
as manual procedures.

M1 produces the first thing worth looking at, and the first gate.

### Order of work

1. Phase 1 in one sitting — it is configuration, and it is what makes the constitution enforceable
2. Phase 2 arithmetic first (T010 through T016), because a wrong bit here poisons everything above
3. Phase 2 core and recording (T017 through T031) until the golden replay test passes
4. Phase 2 shell (T032 through T041) until glyphs drift smoothly on screen
5. **Stop at Checkpoint M0.** Run the three-platform fingerprint check. Answer the hand question
6. Phase 3 simulation side (T042 through T051), then display side (T052 through T059)
7. **Stop at the M1 gate.** Do the self-checks, then put a screenshot in front of five people who
   have not seen it. Answer the gate question
8. Phase 4 whenever convenient

### Hand edits between increments are expected

The plan specifies increments by what they guarantee, not by the functions they contain
(research R14). If Phase 2 is edited by hand after its checkpoint, Phase 3 still holds as long as the
`contracts/sim-api.md` guarantees do. If a hand edit breaks one of those guarantees, that is a
contract change and the contract should be updated first.

### Next task list

M2 tasks are generated after the M1 gate is answered, not before. If the gate answer is "no", the
spec is revised first and M2 may look different.

---

## Notes

- `[P]` means different files and no ordering constraint within the phase
- `[US1]` maps a task to User Story 1 for traceability back to the spec
- Commit after each task or logical group; the golden replay test is the thing that tells you whether
  a commit was safe
- Only three tests appear in this list on purpose — Principle VI prefers one test that proves a
  system over twenty that restate its functions

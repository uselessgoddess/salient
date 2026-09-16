# Quickstart: Validating Each Increment

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-09-16

This is the hands-on companion to the increment plan. Each section is a checkpoint: what to run,
what to look at, and the question to answer before starting the next increment. Sections marked
**GATE** are where the answer may change the design rather than confirm it — stop and decide there.

## Prerequisites

```bash
rustup toolchain install nightly          # exact date pinned in rust-toolchain.toml
cargo build --workspace
```

Standing checks, expected to pass at every checkpoint from M0 onward:

```bash
cargo test -p salient-sim                 # includes the golden replay test
cargo clippy --workspace -- -D warnings   # determinism bans are deny-level here
cargo run -p salient-tools --bin deps-check   # asserts salient-sim pulls in no engine crate
```

---

## M0 — Skeleton that ticks and draws

```bash
cargo run -p salient-app -- --seed 1 --scenario drift
cargo test -p salient-sim replay
```

**Look at**: a field of glyphs drifting in straight lines. Motion must be smooth, not stepped,
even though the simulation updates ten times a second.

**Expect**: the replay test passes; running the same seed twice produces identical fingerprints at
every checkpoint.

**Cross-platform check** (do it once, here, not later):

```bash
cargo run -p salient-tools --bin fingerprint -- --seed 1 --ticks 6000
```

Run on Linux, Windows, and macOS. The printed fingerprints must be identical. If they are not, stop
— nothing built after this point is worth building until they are.

**Question**: is the `Fx` API workable in practice, and is the sim/render boundary comfortable to
work across? This is the cheapest moment in the project to change either.

---

## M1 — The map reads as a staff display **(GATE)**

```bash
cargo run -p salient-app -- --seed 7 --scenario showcase
```

The showcase scenario places a mixed force: land units on high ground, ships in a bay, submarines
directly beneath those ships, and aircraft crossing both.

**Check yourself first**, since these are cheap and a failure here makes the gate pointless:

- The same seed twice gives the same coastline, the same contours, and sources in the same places.
- Zoom from the whole 40 km to a single unit and back. Contour lines stay the same weight on screen
  throughout, intervals appear and fade rather than snapping, and nothing collapses into fill.
- The waterline in the picture agrees with where ships can go. A disagreement means the shader and
  the simulation are reading different sea levels (R10).
- Zoomed fully out, every unit is still drawn. Formation counters sit over the force, not instead of
  it.

**Then run the gate, which you cannot run yourself.** SC-003 needs five observers who know the genre
and have not seen this project. Give each one the same still screenshot, no interaction, no
explanation, and ask them to name the domain of every unit they can see. Record the answers.

**Expect**: at most one wrong identification across the whole sample. The author is not an eligible
observer — a grammar cannot be read cold by the person who drew it, which is the whole reason the
threshold is stated in observers rather than in confidence.

**GATE question**: does the visual language work? Everything in this project rests on the answer,
and it is being asked in the second increment specifically so that a "no" is survivable. A "no" here
costs weeks. The same "no" discovered at M7 costs the project.

If the answer is no, the thing to change is the spec, not the shader. Reaching a gate and turning
back is the plan working.

---

## M2 — Move a force

```bash
cargo run -p salient-app -- --seed 7 --scenario march --units 500
```

**Look at**: box-select and order 500 units across broken terrain and around a lake. Watch them
route, crowd, and arrive.

**Expect**: US1 scenario 4 — all of them arrive, none permanently stuck. Path cost respects terrain
and each domain stays where it belongs.

**Question**: does movement read as competent? Is the ten-hertz cadence visible anywhere it should
not be?

---

## M3 — Build and supply **(GATE)**

```bash
cargo run -p salient-app -- --seed 7 --scenario economy
```

**Do**: claim three sources, connect them, build a factory, produce continuously. Then cut the
route and watch. Then open the tuning panel and move the supply gradient value across its range
while playing.

**Expect**: US2 scenarios 1 through 5. At the permissive extreme the economy behaves as one shared
pool with no positional effect; at the restrictive extreme a disconnected builder halts within
seconds. Constrained links are visibly marked.

**GATE question**: is the supply network interesting to operate, or is it tedious? This was flagged
as the design's main risk from the beginning. If the answer is "tedious", the fix is the gradient
value, not a rewrite — find the setting that is fun, and if no setting is fun, flatten toward the
pool and say so.

---

## M4 — Fight and reclaim

```bash
cargo run -p salient-app -- --seed 7 --scenario skirmish
```

**Do**: let two forces fight to a conclusion. Then send disconnected engineers into the wreckage
and build there.

**Expect**: US3 scenarios 1 through 4, and SC-007 — engineers sustain full build rate on reclaim
alone for at least as long as the engagement lasted.

**Question**: does "ground you fought over is ground you can operate on" come across while playing,
or does it need to be more pronounced?

---

## M5 — A game with an opponent

```bash
cargo run -p salient-app -- --seed 7 --rules build-loss
cargo run -p salient-app -- --seed 7 --rules commander-loss
cargo run -p salient-app -- --replay last.rec
```

**Do**: play to a win and to a loss. Play the same seed under both rule sets. Save mid-match, quit,
resume. Replay a finished match.

**Expect**: US4 scenarios 1 through 7. Under `commander-loss` the match ends when the commander
dies; under `build-loss` it continues with factories intact. The replay ends by the condition the
original ended by, without reselecting it. The resumed match reaches the same outcome as an
uninterrupted run.

**Question**: is it a game yet? This is the first checkpoint where the honest answer can be "no, and
here is what is missing".

---

## M6 — Beyond the network

```bash
cargo run -p salient-app -- --seed 7 --scenario drop
```

**Do**: load a transport, cross the map, unload, build a forward base with no connection home. Then
do it again and shoot down the resupply flight.

**Expect**: US5 scenarios 1 through 4, and SC-006 — a single delivery sustains production for at
least two minutes unconnected. A destroyed loaded transport leaves its cargo as wreckage.

**Question**: does the drop feel like it pays off, or like it is being punished? The design says
rewarded. Check that the numbers agree.

---

## M7 — Incomplete information **(GATE)**

```bash
cargo run -p salient-app -- --seed 7 --scenario intel
```

**Do**: track a hostile force, let it leave sensor coverage, keep playing. Order an attack on a
stale contact and see what is actually there. Send a submarine past a force with no subsurface
coverage.

**Expect**: US6 scenarios 1 through 5, and SC-011 — the contact persists and its uncertainty grows
visibly; reacquisition snaps it back; the undetected submarine produces no contact at all.

**GATE question**: is fighting on stale information tense, or is it merely annoying? This is what
separates the game from a conventional RTS, and it is the one pillar whose appeal cannot be argued
in advance. If it is annoying, the tuning surface is decay rate and sensor coverage before the
pillar itself is reconsidered.

---

## M8 — Command at scale

```bash
cargo run -p salient-app -- --seed 7 --scenario mass --units 20000
cargo bench -p salient-sim tick_budget
```

**Do**: define rule-based groups, give them standing orders, then lose most of a group and let
factories replace it. Run three fronts for ten minutes without touching replacements.

**Expect**: US7 scenarios 1 through 5, SC-004, and SC-001 — 20,000 units without the simulation
falling behind. The benchmark must hold the stated per-tick budget.

**Question**: at 20,000 units, is the limit the machine or the player? The plan predicts the player.
If it is the machine, the parallelism work of R13 is the answer, and it starts here rather than
earlier for the reasons given there.

---

## When a checkpoint fails

Three distinct outcomes, and they are not the same:

- **A test fails** — a defect. Fix and continue; no plan change.
- **A checkpoint question gets a bad answer at a non-gate increment** — a tuning problem. Adjust and
  continue; record what was changed.
- **A GATE question gets a bad answer** — a design problem. Stop, revise the spec, and re-plan from
  that point. This is what the gates are for, and reaching one and turning back is the plan working
  rather than the plan failing.

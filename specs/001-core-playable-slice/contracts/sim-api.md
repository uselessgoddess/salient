# Contract: Simulation Public Surface

**Stability**: This is the contract every other layer depends on. Increments are specified against
it, so hand edits inside `salient-sim` are safe as long as this surface and its guarantees hold.

## The whole surface

```text
Sim::new(seed, rules, scenario, units) -> Sim
Sim::step(&mut self, &[Order])   -> StepReport
Sim::state(&self)                -> &SimState        // read-only
Sim::fingerprint(&self)          -> u64
Sim::tick(&self)                 -> u32
Sim::snapshot(&self)             -> Vec<u8>
Sim::restore(bytes)              -> Result<Sim, RestoreError>
Sim::validate(&self, &Order)     -> Result<(), Reject>
Sim::replay(&Recording)          -> Vec<(tick, fingerprint)>
SIM_VERSION: u32
```

Nothing else is public. In particular there is no setter, no `state_mut`, and no way to inject a
change that did not arrive as an `Order`.

`new` takes the scenario and unit count alongside the seed and rules because those four are the
whole of a match's setup — which is why those four plus the order log are the whole of a recording.
`replay` exists so the golden test and the divergence bisector share one implementation and neither
needs to know how a tick works.

## Guarantees

- **G1 — Purity.** `step` is a pure function of `(state, orders)`. Two `Sim` values with equal
  fingerprints, stepped with equal orders, produce equal fingerprints. Forever, on every platform.
- **G2 — Total.** `step` never panics and never returns an error. Invalid input is rejected by
  `validate` at the boundary; anything reaching `step` is already well-formed.
- **G3 — Closed.** `step` performs no I/O, reads no clock, consults no environment, and allocates
  nothing whose address can influence behaviour.
- **G4 — Fixed cadence.** One `step` is one tick. The simulation has no concept of elapsed real
  time, frames, or speed. Pausing, fast-forwarding, and catching up are the caller's business.
- **G5 — Bounded work.** Work per `step` is a function of state, never of wall-clock budget. Any
  amortised computation is sliced by a fixed count per tick, never by a timer.
- **G6 — Observable only.** `state` hands out an immutable view. A renderer, an AI, a recorder, and
  a network peer are indistinguishable to the simulation.

## Dependency ban

`salient-sim` MUST NOT depend, directly or transitively, on: any engine or windowing crate, any
renderer, any networking crate, any async runtime, any clock, or any randomness source other than
its own seeded generator. Enforced by a CI check over the resolved dependency graph, not by review.

## StepReport

`step` returns what the outside world needs to react to, so that nobody has to diff state to find
out what happened: units created and destroyed, orders rejected, victory condition met, unit cap
reached. It is derived from the tick and carries no authority — discarding it changes nothing.

## Caller responsibilities

- Collect orders for a tick and pass them in a stable order. That order is recorded and is what
  makes conflict resolution deterministic.
- Call `step` at the intended cadence. Falling behind is the caller's problem to detect and report.
- Never assume `state` is stable across a `step`. Handles may be reused.

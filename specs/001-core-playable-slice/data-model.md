# Phase 1 Data Model: Core Playable Slice

**Feature**: [spec.md](./spec.md) | **Research**: [research.md](./research.md) | **Date**: 2026-09-16

All simulation state lives in dense arenas indexed by generational handles. Every field below is
`Fx` (Q32.32), an integer, or a handle — no floating point appears anywhere in this model
(Principle III). `Fx2` denotes the wide type reserved for squared magnitudes.

The whole of `SimState` is the unit of serialization, snapshotting, and fingerprinting: anything
listed here participates in the state fingerprint, and anything not listed here must not affect
simulation outcomes.

---

## Identity

**Handle**: `{ slot: u32, generation: u32 }`. Arena slots are reused; the generation
distinguishes a live entity from a dead one that occupied the same slot. Handles are stable within a
match and are part of the fingerprint, which means slot reuse order is itself simulation state and
must be deterministic.

**Player**: `u8`. Index into a small fixed set. Player 0 is the human; the hostile force occupies
its own index. Neutral ownership is a reserved value.

---

## World

### Map

Generated entirely from a seed and immutable for the duration of a match (Assumption: terrain is
static).

| Field | Type | Notes |
|---|---|---|
| `seed` | `u64` | The only input to generation |
| `extent` | `(Fx, Fx)` | World size; 40 km x 40 km at target scale |
| `height` | grid of `i16` | Elevation, one sample per terrain cell |
| `sea_level` | `i16` | Cells below this are water |
| `passability` | grid of `u8` bitset | One bit per domain |
| `move_cost` | grid of `u8` | Terrain cost multiplier per cell, per FR-012 |
| `sources` | `Vec<ResourceSource>` | Placement is part of generation |

**Validation**: generation MUST reject a seed whose map has no player-reachable resource sources
(Edge case: unusable layout). Rejection is deterministic — the same seed always fails.

### Resource Source

| Field | Type | Notes |
|---|---|---|
| `pos` | `(Fx, Fx)` | Fixed for the match |
| `kind` | enum | One of the two resource kinds |
| `yield_rate` | `Fx` | Per tick when claimed |
| `owner` | `Option<Player>` | `None` until claimed |

**State transitions**: `unclaimed -> claimed(player)` when an extractor structure completes on it;
`claimed -> unclaimed` when that structure dies.

---

## Entities

### Unit

| Field | Type | Notes |
|---|---|---|
| `pos`, `vel` | `(Fx, Fx)` | Integrated each tick |
| `facing` | `Fx` | Heading in turns; 1.0 is a full revolution |
| `domain` | enum | `Land \| Surface \| Subsurface \| Air`. Exactly one, immutable |
| `class` | handle | Into the unit type table; carries speed, health, weapons, roles |
| `owner` | `Player` | |
| `health` | `Fx` | |
| `stock` | `Option<Stock>` | Present only for builders and transports |
| `cargo` | `Option<Cargo>` | Transports only |
| `orders` | queue of `Order` | Per-unit queue; see Order |
| `group` | `Option<GroupId>` | Derived each tick from group rules, never assigned directly |

**Validation**: `domain` MUST be consistent with the terrain under `pos` at all times — a land unit
may never occupy a cell whose land passability bit is clear (FR-012). Orders that would violate this
are rejected at issue time, not at execution time (Edge case: order into impassable terrain).

**State transitions**: `alive -> dead` on health reaching zero, which spawns Wreckage at `pos` and
converts any `cargo` to Wreckage as well (FR-014, Edge case: loaded transport destroyed).

### Structure

| Field | Type | Notes |
|---|---|---|
| `pos` | `(Fx, Fx)` | Fixed |
| `kind` | enum | `Factory \| Extractor \| Sensor \| Defence \| SupplyNode` |
| `owner` | `Player` | |
| `health` | `Fx` | |
| `build_progress` | `Fx` | Below completion the structure is inert but persists |
| `stock` | `Stock` | Every structure holds one (FR-015) |
| `queue` | `Vec<ClassHandle>` | Factories only |

**State transitions**: `under_construction -> complete` at full progress. A structure whose supply is
cut mid-construction **persists and continues at the decayed rate** rather than being cancelled
(Edge case). `complete -> dead` spawns Wreckage.

### Commander

Not a separate arena — a Unit whose class is flagged `is_commander`. Present in every match as the
player's initial builder (FR-038). Whether its death ends the match is a property of Match Rules, not
of this entity.

### Wreckage

| Field | Type | Notes |
|---|---|---|
| `pos` | `(Fx, Fx)` | |
| `remaining` | `Fx` per kind | A fraction of the source entity's cost |

**Validation**: reclaim by two opposing engineers in the same tick MUST be resolved by a rule
independent of arena order — each claimant's draw is computed against the tick-start remainder and
scaled down proportionally if the total would exceed it (Edge case: contested wreckage).

---

## Supply

### Stock

The local buffer that makes the economy a gradient rather than a gate (FR-015, FR-016).

| Field | Type | Notes |
|---|---|---|
| `held` | `Fx` per kind | Current contents |
| `capacity` | `Fx` per kind | |

**Invariant**: consumption draws from `held` and continues at a reduced floor rate when `held`
reaches zero. It never hard-stops on disconnection.

### Supply Link

| Field | Type | Notes |
|---|---|---|
| `a`, `b` | handles | The two nodes it connects |
| `capacity` | `Fx` per tick | Finite; the constraint that makes geography matter |
| `flow_last_tick` | `Fx` | Display only (FR-010), but part of state |

**Solver invariant** (FR-018): all transfers for a tick are computed from stock levels as they were
at tick start, then applied together. No link may observe another link's transfer within the same
tick. This is what makes allocation independent of iteration order.

### Forward Supply Point

A Structure of kind `SupplyNode` created by unloading a transport (FR-021). Identical to any other
node except that it has no link to the rest of the network until one is built, so it drains and is
refilled only by further deliveries.

---

## Command

### Order

The only input to the simulation and the only thing a recording stores (FR-002).

| Field | Type | Notes |
|---|---|---|
| `tick` | `u32` | The tick at which it takes effect |
| `issuer` | `Player` | |
| `target` | `OrderTarget` | Either a set of unit handles or a `GroupId` |
| `action` | `OrderAction` | `Move \| Attack \| Build \| Reclaim \| Load \| Unload \| Stop` |
| `params` | position, handle, or class | Action-dependent |

**Validation**: rejected at the boundary, never asserted deep inside (Constitution: simulation
prohibitions). Rejection reasons include impassable destination, unowned target, and unknown handle.

**Conflict rule** (Edge case): two orders affecting the same unit in the same tick are applied in
the order they appear in the tick's order list, which is itself deterministic because it is the
recorded order. The later order wins.

### Group

| Field | Type | Notes |
|---|---|---|
| `id` | `GroupId` | |
| `rule` | predicate | Over class, role, domain, and state (FR-031) |
| `standing` | `Option<Order>` | Adopted by units joining later (FR-033) |

**Membership is derived, never stored as a list** (FR-032). It is re-evaluated each tick, so newly
produced units join automatically and destroyed ones leave without bookkeeping.

**Precedence** (FR-034): a unit with a non-empty personal order queue ignores its group's standing
order. When its queue empties, it adopts the standing order again.

**Overlap rule** (Edge case): a unit may match several group rules. It belongs to all of them for
selection purposes, but adopts the standing order of the lowest-numbered matching group, so the
resulting order is always defined.

---

## Intelligence

### Contact

The player's belief about a hostile entity. Contacts are **per-player derived state**: they are
part of the fingerprint, but they are never the source of truth about the world.

| Field | Type | Notes |
|---|---|---|
| `observer` | `Player` | Whose picture this belongs to |
| `last_pos` | `(Fx, Fx)` | Position when last observed |
| `last_seen` | `u32` | Tick of last observation |
| `uncertainty` | `Fx` | Radius, grows with age since `last_seen` (FR-027) |
| `domain`, `class` | inferred | May be partial |

**State transitions**: `current` while inside sensor coverage; `stale` once coverage is lost, with
`uncertainty` growing per tick; back to `current` on reacquisition, snapping `last_pos` to truth
(FR-026, US6 scenario 3). A contact for a domain the observer has no coverage for is never created
at all (FR-028).

---

## Match

### Match Rules

| Field | Type | Notes |
|---|---|---|
| `victory` | set of conditions | `CommanderLoss \| AllUnitsLost \| BuildCapabilityLost` (FR-039) |
| `supply_gradient` | `Fx` | The single tuning value of FR-019 |
| `unit_cap` | `u32` | Enforced visibly (Edge case) |

**Extension rule** (FR-042): a victory condition is a predicate over `SimState` evaluated once per
tick. Adding one MUST touch only its own predicate and the enum.

### Hostile Force

| Field | Type | Notes |
|---|---|---|
| `held` | sector bitset | Over the coarse sector grid |
| `spread_rate` | `Fx` | Sectors gained per tick under contest |
| `pressure` | `Fx` | Derived from `held.count()` (FR-023) |

### Recording

Not simulation state — the serialized form of a match.

| Field | Type | Notes |
|---|---|---|
| `sim_version` | `u32` | Refused on mismatch (FR-003) |
| `seed` | `u64` | |
| `rules` | `MatchRules` | Restored on replay (FR-041) |
| `orders` | `Vec<Order>` | Ordered by tick |
| `checkpoints` | `Vec<(u32, u64)>` | Tick and fingerprint, for divergence bisection (FR-004) |

---

## What is deliberately not state

These influence what the player sees but never what happens, and therefore never participate in the
fingerprint: camera position and zoom, selection, display filters, the interpolation factor between
ticks, and every renderer-side buffer. If any of these could change an outcome, the sim/render
boundary of Principle II has been breached.

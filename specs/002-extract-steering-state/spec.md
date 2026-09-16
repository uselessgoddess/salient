# Feature Specification: Steering State as Its Own Subsystem

**Feature Branch**: `002-extract-steering-state`

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "Вынести goal из Unit — всё-таки это отдельная подсистема."

## Scope of This Specification

A unit record currently carries the destination it was last ordered to. That value is not a
property of a unit; it is the residue of an order. It is written by the order system, read by
the movement system, and is meaningless to everything else — yet it sits in the one record that
every other system touches.

This spec moves order-issued movement intent out of the unit record and into a subsystem of its
own, and splits the movement tick into steering (units that were told to go somewhere) and
integration (every unit, whether ordered or not).

**The simulation's behaviour does not change.** Every unit follows the same path at the same
speed and stops in the same place. The only externally visible change is that the state
fingerprint takes new values, because the serialised shape of state changes. That is a
bookkeeping consequence, not a behavioural one, and the golden fingerprints are regenerated
deliberately with the reason recorded.

**Deliberately excluded from this change**, each its own decision later:

- Converting unit storage to struct-of-arrays. Measurement does not support it: a 20,000-unit
  tick currently costs 37 µs against a 10 ms gate, a margin of roughly 270×.
- Moving movement tunables (speed, arrival radius) out of the movement module into match rules.
- Collision, mutual avoidance, and any spatial index. These are the systems that will eventually
  make the layout question real; they are not part of this change.
- Long-range routing. Units still head straight at their destination.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Order-issued intent lives outside the unit record (Priority: P1)

A destination given by an order is stored as steering intent belonging to a unit, not as a field
inside the unit. The order system writes intent; the steering system reads it and clears it on
arrival; nothing else sees it. A unit record describes only what a unit is — where it is, how
fast it is going, which way it faces, which medium it occupies, who owns it, how healthy it is.

**Why this priority**: This is the whole change, and it is the one that stops a leak rather than
fixing a symptom. Every order-driven behaviour still to come — attack target, patrol route, build
queue, formation slot, transport assignment — is another value that is absent for most units most
of the time. If the first one lives in the unit record, all of them will, and the record becomes
the place things go when they do not fit anywhere else.

**Independent Test**: Issue movement orders to part of a force and confirm the ordered units move
and arrive exactly as before, while the unit record exposes no destination field to any caller.

**Acceptance Scenarios**:

1. **Given** a unit with no orders, **When** the match runs, **Then** it carries no steering
   intent and its stored description contains nothing about a destination.
2. **Given** a move order against a unit, **When** the order is applied, **Then** intent for that
   unit exists and the unit's own record is unchanged apart from the motion that follows.
3. **Given** a unit that reaches its destination, **When** it arrives, **Then** its intent is gone
   and its velocity is zero.
4. **Given** a stop order against a unit under orders, **When** the order is applied, **Then** its
   intent is gone and its velocity is zero.
5. **Given** a stop order against a unit that has no intent, **When** the order is applied,
   **Then** nothing happens and no error is raised.
6. **Given** a unit already under orders, **When** a second move order targets it, **Then** the
   later destination replaces the earlier one, as before.

---

### User Story 2 - Steering can be deleted without breaking the game (Priority: P2)

Removing the steering subsystem leaves a simulation that still runs. Units still exist, still
carry velocity, still have that velocity integrated into position each tick, still serialise,
still fingerprint. What is lost is the ability to be told where to go — and nothing else.

**Why this priority**: The constitution requires every subsystem except the core to be deletable
without making the game unplayable, and requires each one to state what is lost if it is removed.
While intent lives inside the unit record, that claim cannot be true of movement: deleting it
would mean editing the record every other system reads. This story is what converts the principle
from an aspiration into a fact about this subsystem.

**Independent Test**: Run the simulation with the steering phase not registered, over a scenario
whose units already have velocity, and confirm the run completes, positions advance, and a
fingerprint is still produced.

**Acceptance Scenarios**:

1. **Given** the steering phase is not run, **When** the simulation advances 2,000 ticks, **Then**
   it completes without panicking and every unit's position has advanced by its velocity.
2. **Given** the steering phase is not run, **When** state is serialised and restored, **Then**
   the restored state matches the original.
3. **Given** the steering subsystem is present, **When** a reader asks what is lost by deleting
   it, **Then** the answer is recorded in one sentence alongside the subsystem.

---

### Edge Cases

- **A unit dies while under orders.** Its intent must not survive it. Nothing may later steer a
  unit that no longer exists.
- **A dead unit's storage slot is reused by a new unit.** This is the sharpest risk the change
  introduces. Unit storage already distinguishes a live unit from a dead one that occupied the
  same slot; steering intent, once stored separately, must make the same distinction or a freshly
  built unit will inherit a dead one's destination and walk away on its own. That failure would
  be reproducible, silent, and would look exactly like a movement bug rather than a storage bug.
- **Intent referring to a unit that was never created.** Must be impossible to observe, not merely
  unlikely.
- **No unit has intent.** Steering does no work; integration still runs for every unit.
- **Every unit has intent.** Steering costs what the movement pass costs today.
- **Intent survives a snapshot.** Saving and restoring mid-move must resume to the same
  destination, not drop it.
- **A destination that is exactly at the unit's position.** Arrival on the first tick, with no
  division by a zero-length vector.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A unit's stored description MUST NOT contain a destination or any other value
  written only by the order system.
- **FR-002**: Steering intent MUST be addressable per unit, and MUST NOT be observable for a unit
  that is not alive.
- **FR-003**: Steering intent MUST NOT be inherited by a unit that reuses the storage slot of a
  unit that held intent.
- **FR-004**: A move order MUST set the target unit's intent, replacing any intent already held.
- **FR-005**: A stop order MUST clear intent and zero velocity, and MUST be harmless against a
  unit holding no intent.
- **FR-006**: Arrival MUST clear intent and zero velocity, at the same distance threshold as
  before this change.
- **FR-007**: Integration MUST apply to every live unit, whether or not it holds intent.
- **FR-008**: For a given seed, scenario, and order log, every unit's position, velocity, and
  facing at every tick MUST be identical to the values produced before this change.
- **FR-009**: Iteration over steering intent MUST be in an explicit, reproducible order that is a
  property of the data, not of a container's internal layout.
- **FR-010**: Steering intent MUST survive serialisation and restoration unchanged.
- **FR-011**: The golden replay fingerprints MUST be regenerated, with the reason recorded in the
  commit that makes the change. `SIM_VERSION` MUST NOT be bumped: the project is pre-release, no
  stored artifact outlives the build that produced it, and the constitution scopes version
  discipline to the point where one does.
- **FR-012**: The steering subsystem MUST be removable without editing the unit record, unit
  storage, or any other subsystem, and MUST carry a one-sentence statement of what its removal
  costs.
- **FR-013**: The equivalence check proving FR-008 MUST be a migration-time verification, not a
  committed test. Once the change has landed there is no future change that it could fail for,
  and a test that cannot fail for a reason worth fixing must not be kept.

### Key Entities

- **Unit**: What a unit *is*. Position, velocity, facing, medium, owner, health. Every field is
  either written every tick by integration or describes the unit's identity. No field belongs to
  a single subsystem that could be removed.
- **Steering intent**: What a unit was *told*. A destination, belonging to exactly one live unit,
  created by an order and destroyed by arrival, by a stop order, or by the unit's death. Absent
  for most units in a typical match.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For the same seed and order log, 100% of units hold identical positions, velocities,
  and facings at every checkpoint before and after the change.
- **SC-002**: With steering removed, a 2,000-tick run completes, produces a fingerprint, and
  panics zero times.
- **SC-003**: The unit record contains zero fields written by exactly one removable subsystem.
- **SC-004**: A 20,000-unit tick stays within the 10 ms gate. The measured cost before this change
  is 37 µs; any result within the gate is a pass, and the figure is recorded so that a later
  regression has something to be a regression against.
- **SC-005**: Adding a second order-driven behaviour later requires zero changes to the unit
  record. Verified the next time one is added, not now.

## Assumptions

- Steering intent is stored densely, one potential entry per unit storage slot, in the first
  implementation. A sparse form costing only what commanded units cost is a later change, and the
  measurement that would justify it — the fraction of units under orders in a real match — does
  not exist yet, because the only scenario today puts every unit in motion.
- Movement speed and arrival radius stay in the movement module in this change. Moving them into
  match rules is a separate decision.
- Group-addressed orders remain unimplemented, exactly as now.
- Units still head straight at their destination. Routing around terrain and around each other is
  out of scope, and is the work that will eventually make the storage-layout question worth
  reopening.
- The existing golden replay test is the regression gate for this change; no new permanent test is
  expected beyond what already exists, per FR-013.
- Governed by constitution v1.6.0: Principle I (every subsystem deletable, and no type
  accumulating one subsystem's state — the rule this change exists to satisfy), Principle III
  (determinism by construction, explicit iteration order), Principle VI (few sharp tests),
  Principle IX (constants and paths), and the pre-release scoping of version discipline in the
  workflow section.
- The project is pre-release: no build has been handed to anyone, no replay or save is kept beyond
  the run that made it, and there is no networked match. Should any of those change before this
  work lands, FR-011 reverts to requiring a version bump and the refusal path needs a test again.

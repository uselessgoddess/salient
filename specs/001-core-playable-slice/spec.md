# Feature Specification: Core Playable Slice

**Feature Branch**: `001-core-playable-slice`

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "Salient — an operational-scale real-time strategy game in the visual language of a staff tactical display. The player grows a supply network, holds a front, and commands at a scale where the map is the interface. This specification covers the first playable slice: the smallest version that is already a game and that exercises every load-bearing design decision."

## Scope of This Specification

This spec covers the **first playable slice** of Salient, not the finished game. The slice is
chosen so that every architecturally risky decision is proven inside it, and so that each user
story below leaves a playable game if the ones after it are never built.

**Deliberately excluded from this slice**, each planned as its own later specification:
tapping into an opponent's supply network, unit capture, teleport links, unit tiers above the
first, experimental units, shields, strategic weapons, structure upgrades, a skirmish opponent
that builds a base, authored scenarios, multiplayer, multiple factions, and a persistent
campaign layer carrying a front between matches.

Excluding these is a scope decision, not a design change. All of them were designed for and
remain compatible with the model described here.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read the battlefield and move a force (Priority: P1)

The player opens a generated map and sees it as a staff tactical display: terrain as contour
lines, water as hatching, units as geometric glyphs. Land, surface, subsurface, and air units are
all visible at once and the player can tell which is which at a glance, without switching views.
The player selects units and orders them to move; they pathfind around terrain and each other.

**Why this priority**: This is the visual language the entire game is built on and the movement
substrate everything else sits on. If the display cannot be read without help, no later feature
can rescue the game. It is also the smallest slice that proves reproducibility end to end.

**Independent Test**: Generate a map from a seed, spawn a mixed force of all four domains, issue
movement orders, and confirm that an observer identifies every unit's domain correctly without
interacting with the display, and that units reach their destinations without stalling.

**Acceptance Scenarios**:

1. **Given** a map generated from a seed, **When** the same seed is used again, **Then** the
   resulting terrain, water, and resource placement are identical.
2. **Given** a submarine and a surface ship occupying the same position, **When** the player looks
   at the display, **Then** the submarine reads as beneath the surface and the ship as on it,
   with no view toggle used.
3. **Given** an aircraft over land, **When** the player looks at the display, **Then** the
   aircraft is distinguishable from the ground units below it by its offset and ground shadow.
4. **Given** 500 units ordered to a single destination across broken terrain, **When** they move,
   **Then** all of them arrive and none become permanently stuck.
5. **Given** a recorded sequence of orders and a seed, **When** the match is replayed, **Then**
   every unit ends in an identical position and state.
6. **Given** the player zooms out, **When** the display crosses a detail threshold, **Then**
   individual units resolve into aggregate markings without the player losing track of where
   forces are.
7. **Given** a match is saved partway through, **When** it is resumed, **Then** play continues to
   the same outcome as an uninterrupted run of the same orders.

---

### User Story 2 - Build and sustain a force (Priority: P2)

The player claims resource sources, connects them into a supply network, and builds engineers,
factories, and units. Every producer and builder holds a local stock of resources. The network
refills those stocks; it does not gate whether building is allowed. A builder that is connected
works at full rate; a builder that is disconnected keeps working, more slowly, until its stock
runs out.

**Why this priority**: This is the growth loop, and it is where the central balance decision
lives. It must be playable early enough that the supply model can be judged by feel rather than
by argument.

**Independent Test**: Start from a single base, claim and connect three resource sources, build a
factory, and produce units continuously. Then sever the connection and confirm production
continues at a decaying rate rather than halting.

**Acceptance Scenarios**:

1. **Given** a connected factory with a full local stock, **When** it builds, **Then** it builds
   at full rate and its stock stays full.
2. **Given** a factory whose supply route is cut, **When** it continues building, **Then** its
   rate declines as its local stock drains and reaches a floor rather than zero-on-cut.
3. **Given** a supply route with capacity lower than downstream demand, **When** demand exceeds
   capacity, **Then** the shortfall is distributed by a rule the player can observe on the
   display, and the constrained link is visibly marked.
4. **Given** a single tuning value for how fast local stock drains relative to refill, **When**
   it is set to its permissive extreme, **Then** the economy behaves as a single shared pool with
   no positional effect.
5. **Given** the same tuning value set to its restrictive extreme, **When** a builder is
   disconnected, **Then** it halts almost immediately.

---

### User Story 3 - Fight, and leave a battlefield behind (Priority: P3)

Units engage automatically within weapon range, take damage, and die. Destroyed units leave
wreckage. Engineers reclaim wreckage into resources on the spot, with no supply connection
required, so ground that has been fought over is easier to operate on than empty ground.

**Why this priority**: Combat makes the previous two stories mean something. Reclaim is bundled
here rather than later because it is what makes contested ground self-supplying, which is the
first half of the answer to "why push forward".

**Independent Test**: Place two opposing forces in contact, let them fight to a conclusion, then
send engineers into the resulting wreckage field and confirm they sustain construction there
while fully disconnected from any supply route.

**Acceptance Scenarios**:

1. **Given** two hostile units within weapon range, **When** neither is ordered otherwise,
   **Then** they engage without player input.
2. **Given** a unit is destroyed, **When** it dies, **Then** wreckage carrying a fraction of its
   cost appears at its position.
3. **Given** a disconnected engineer standing in a wreckage field, **When** it builds, **Then**
   it draws from reclaim and sustains full build rate while wreckage remains.
4. **Given** a wreckage field is fully reclaimed, **When** an engineer continues to build there,
   **Then** it falls back to draining its local stock.

---

### User Story 4 - Face an opponent that pushes back (Priority: P4)

A hostile force occupies part of the map and expands by simple, legible rules. The pressure it
applies grows with the territory it holds, so ignoring it is punished and pushing it back is
rewarded. The player wins or loses against it.

**Why this priority**: This turns a sandbox into a game. It is deliberately a rule-driven system
rather than a reasoning opponent, so that a playable game exists regardless of whether a more
capable opponent is ever built.

**Independent Test**: Start a match against the expanding force and play to a win and to a loss.
Confirm that the pressure it generates tracks the territory it holds, and that the same seed and
order sequence produce the same outcome every time.

**Acceptance Scenarios**:

1. **Given** the hostile force holds territory, **When** time passes, **Then** the force it
   commits scales with the area held.
2. **Given** the player destroys a hostile holding, **When** the territory is retaken, **Then**
   the pressure generated by the opponent measurably drops.
3. **Given** the player takes no action, **When** enough time passes, **Then** the player loses.
4. **Given** identical seed and orders, **When** the match is replayed, **Then** the opponent
   behaves identically.
5. **Given** a match set to end on commander loss, **When** the commander is destroyed, **Then**
   the match ends in defeat regardless of what forces remain.
6. **Given** a match set to end on loss of build capability, **When** the commander is destroyed
   while factories remain, **Then** the match continues.
7. **Given** a recorded match, **When** it is replayed, **Then** it ends by the same condition it
   originally ended by, without that condition being reselected by hand.

---

### User Story 5 - Operate beyond the network (Priority: P5)

The player loads resources into transports, moves them, and unloads them to create a forward
supply point. Engineers dropped behind the front can be sustained by these deliveries, letting
the player establish and hold a base far from their own territory. Transports are visible on the
display and can be intercepted, so the supply line is a target.

**Why this priority**: This is what makes the supply model a space of choices rather than a leash.
It is placed after combat because it is only meaningful once there is a front to get behind, and
it is separable because the game is complete without it.

**Independent Test**: Load a transport, fly it across the map, unload at an unclaimed location,
and build a functioning forward base there with no connection to the home network. Then destroy
an inbound transport and confirm the forward base decays.

**Acceptance Scenarios**:

1. **Given** a transport carrying resources, **When** it unloads at a position, **Then** a supply
   point exists there that nearby builders can draw from.
2. **Given** a forward base with no deliveries arriving, **When** its stock is exhausted, **Then**
   production there decays to the disconnected floor rather than stopping instantly.
3. **Given** a transport in flight, **When** it is destroyed, **Then** its cargo is lost and
   appears as wreckage at the crash position.
4. **Given** a forward base is established, **When** the player inspects the display, **Then**
   the base and its deliveries are visibly distinct from network-connected holdings.

---

### User Story 6 - Fight on incomplete information (Priority: P6)

The player does not see the truth of the map; the player sees an intelligence picture. Hostile
units appear as contacts detected by sensors, and each contact carries an age and a confidence.
When a contact is no longer observed, it does not vanish — it degrades: its last known position
widens into an area of uncertainty that grows over time.

**Why this priority**: This is what separates Salient from a conventional real-time strategy game,
and it is what justifies the display fiction. It is placed late because everything beneath it must
work first, and the game remains playable with full visibility if it is never built.

**Independent Test**: Run a match with sensors enabled, move a hostile force through and out of
sensor coverage, and confirm the player's picture shows a degrading contact rather than either a
tracked unit or nothing at all.

**Acceptance Scenarios**:

1. **Given** a hostile unit inside sensor coverage, **When** the player looks at the display,
   **Then** a contact is shown at its true position and marked current.
2. **Given** a tracked hostile unit leaves sensor coverage, **When** time passes, **Then** the
   contact remains, is marked stale, and its area of uncertainty grows.
3. **Given** a stale contact, **When** sensors reacquire the unit, **Then** the contact snaps back
   to the true position and is marked current again.
4. **Given** a submerged unit outside subsurface sensor coverage, **When** the player looks at the
   display, **Then** no contact is shown for it at all.
5. **Given** the player orders an attack on a stale contact, **When** the units arrive, **Then**
   they act on what is actually there, which may be nothing.

---

### User Story 7 - Command a large force without repetitive work (Priority: P7)

The player selects units by what they are rather than by where they are on screen: all bombers,
all idle engineers, everything damaged. Groups are defined by a rule rather than a fixed list, so
newly produced units join the group they belong to automatically. Orders live on the group, so
replacements inherit the standing task without the player reissuing anything.

**Why this priority**: This is what makes the target scale playable by one person. It is last
because it is an amplifier: it has nothing to amplify until the force-building and combat layers
exist, and the game is playable without it at small unit counts.

**Independent Test**: Define a rule-based group, give it a standing order, then destroy most of it
and let factories replace the losses. Confirm replacements join the group and resume the standing
order with no further player input.

**Acceptance Scenarios**:

1. **Given** a group defined by a rule, **When** a new unit matching the rule is produced,
   **Then** it joins the group automatically.
2. **Given** a group with a standing order, **When** a unit joins the group, **Then** it adopts
   the standing order without the player reissuing it.
3. **Given** a mixed force on screen, **When** the player selects by a class filter, **Then** only
   units of that class are selected, regardless of what else is nearby.
4. **Given** a group rule that currently matches no units, **When** it is used, **Then** it
   remains valid and begins matching as soon as a qualifying unit exists.
5. **Given** a unit belongs to a group, **When** the player issues it a direct order, **Then** the
   direct order takes precedence over the standing order until the unit becomes idle.

---

### Edge Cases

- A builder's supply route is cut in the middle of construction: the partially built structure
  persists and construction continues at the decayed rate rather than being cancelled.
- A supply network is split into two disconnected halves by a destroyed link: each half continues
  independently from its own sources.
- Demand permanently exceeds total supply capacity: allocation is stable and does not oscillate
  between consumers from tick to tick.
- A unit is ordered into terrain it cannot enter: the order is rejected at the moment it is issued
  rather than leaving the unit pushing against an obstacle.
- A transport is destroyed while loaded: its cargo becomes wreckage rather than disappearing.
- A forward supply point is exhausted with builders still drawing from it: those builders fall
  back to their own local stock.
- A wreckage field is contested by both sides' engineers simultaneously: reclaim is resolved
  without either side gaining from ordering-dependent behaviour.
- Two orders affecting the same unit are issued within the same tick: resolution is defined and
  identical on every run.
- A group rule matches a unit that is already in another group: membership rules are unambiguous
  and the unit's standing order is defined.
- A recording is loaded against a build whose simulation behaviour has changed: the recording is
  refused with an explanation rather than played back to a wrong outcome.
- The map is generated from a seed producing an unusable layout, such as no reachable resource
  sources: generation detects and rejects such layouts.
- Unit count reaches the supported maximum and production continues: the limit is enforced
  visibly rather than by silent failure.

## Requirements *(mandatory)*

### Functional Requirements

**Reproducibility and recording**

- **FR-001**: The game MUST produce identical outcomes from identical starting seed and order
  sequence, on every supported platform and in every build configuration.
- **FR-002**: The game MUST be able to record a match as a seed plus an order sequence, and replay
  that recording to an identical outcome.
- **FR-003**: Recordings MUST identify the simulation behaviour they were produced by, and MUST be
  refused rather than played back when that behaviour has changed.
- **FR-004**: The game MUST be able to report a state fingerprint at any point in a match, such
  that two runs can be compared and the first point of divergence located.

**Map and display**

- **FR-005**: Maps MUST be generated from a seed, including terrain elevation, water extent, and
  resource source placement.
- **FR-006**: The display MUST render terrain, water, units, and structures from generated
  geometry only, with no authored image assets.
- **FR-007**: The display MUST distinguish land, surface, subsurface, and air units simultaneously
  without requiring the player to switch modes or filters.
- **FR-008**: Unit glyphs MUST encode domain by shape and role by marking, using a consistent
  visual grammar across all unit types.
- **FR-009**: The display MUST support continuous zoom, resolving individual units into aggregate
  markings as the view widens.
- **FR-010**: The display MUST mark supply routes with their throughput and highlight routes that
  are constraining demand.

**Movement and combat**

- **FR-011**: Units MUST move to ordered destinations, routing around impassable terrain and
  around each other.
- **FR-012**: Movement cost MUST vary with terrain, and each unit domain MUST be restricted to the
  terrain it can occupy.
- **FR-013**: Units MUST engage hostile units within weapon range without player input, unless
  ordered otherwise.
- **FR-014**: Destroyed units MUST leave wreckage carrying a fraction of their cost at the
  position of destruction.

**Economy and supply**

- **FR-015**: Every builder, producer, and supply point MUST hold its own local stock of
  resources.
- **FR-016**: Construction and production MUST draw from local stock, and MUST continue at a
  reduced rate when local stock is depleted rather than stopping entirely.
- **FR-017**: The supply network MUST transfer resources from sources to local stocks along
  routes with finite throughput.
- **FR-018**: When demand exceeds a route's throughput, allocation among consumers MUST be stable
  across ticks and independent of internal ordering.
- **FR-019**: A single tuning value MUST control the balance between local stock drain and network
  refill, spanning from a fully shared economy at one extreme to a strictly local one at the other,
  without code changes.
- **FR-020**: Engineers MUST be able to reclaim wreckage into resources with no supply connection
  required.
- **FR-021**: Transports MUST be able to load resources, move them, and unload them to establish a
  supply point at an arbitrary position.

**Opponent**

- **FR-022**: The game MUST provide a hostile force that expands across the map by rules, without
  requiring a reasoning opponent.
- **FR-023**: The pressure applied by the hostile force MUST scale with the territory it holds, so
  that retaking territory measurably reduces pressure.
- **FR-024**: The match MUST have defined win and loss conditions against the hostile force.

**Intelligence**

- **FR-025**: The player MUST see hostile forces only as sensor-derived contacts, never as ground
  truth.
- **FR-026**: Each contact MUST carry an observation age and MUST be visually distinguishable as
  current or stale.
- **FR-027**: A contact that is no longer observed MUST persist with a growing area of
  uncertainty rather than disappearing immediately.
- **FR-028**: Sensor coverage MUST be domain-specific, so that detecting a subsurface unit
  requires different coverage than detecting a surface or air unit.
- **FR-029**: Orders issued against a stale contact MUST execute against actual conditions at the
  destination, including the possibility that nothing is there.

**Command**

- **FR-030**: The player MUST be able to select units by class, role, and state, in addition to
  selecting by position.
- **FR-031**: Groups MUST be definable by a rule rather than a fixed membership list.
- **FR-032**: Units newly produced that match a group's rule MUST join that group without player
  action.
- **FR-033**: Orders MUST be assignable to a group such that units joining later adopt the
  standing order.
- **FR-034**: A direct order to a unit MUST take precedence over its group's standing order until
  the unit becomes idle.

**Scale**

- **FR-035**: The game MUST sustain 20,000 simultaneous units on a 40 x 40 km map without the
  simulation falling behind real time.
- **FR-036**: Units MUST appear to move smoothly at the display's refresh rate rather than
  advancing in visible discrete steps.

**Structure**

- **FR-037**: Each layer above the core MUST be removable without making the game unplayable, and
  the build MUST be able to demonstrate this.

**Match rules and session**

- **FR-038**: Every match MUST begin with a commander unit acting as the player's initial builder,
  regardless of which victory conditions are in effect. The opening of a match MUST NOT differ
  between rule sets.
- **FR-039**: Victory and defeat conditions MUST be selectable per match rather than fixed,
  offering at minimum: defeat on loss of the commander, defeat on loss of all units, and defeat on
  loss of the ability to build.
- **FR-040**: The default defeat condition MUST be loss of the ability to build.
- **FR-041**: The rule set in effect MUST be recorded with the match and MUST be restored when the
  match is replayed.
- **FR-042**: Adding a victory condition MUST require no change to any system other than the rule
  that evaluates it.
- **FR-043**: A session MUST be a bounded match on a generated map, beginning from a seed and
  ending in a win or a loss.
- **FR-044**: A match MUST be able to be saved and resumed without altering its outcome.

### Key Entities

- **Map**: A generated battlefield. Holds elevation, water extent, passability per domain, and
  the positions of resource sources. Fully determined by its seed.
- **Resource Source**: A fixed position on the map that yields resources when claimed. The
  original anchor of the supply network.
- **Supply Node**: Any position holding a local stock — a structure, a builder, or a forward
  supply point dropped by a transport.
- **Supply Route**: A connection between supply nodes with a finite throughput. Carries resources
  from sources toward consumers and can be severed.
- **Unit**: A mobile entity belonging to exactly one domain (land, surface, subsurface, air), with
  a position, health, a local stock where applicable, and zero or more weapons.
- **Structure**: A fixed entity that produces, builds, senses, or defends. Holds a local stock.
- **Wreckage**: Reclaimable resources left at the position of a destroyed unit or structure.
- **Order**: An instruction issued at a specific moment against a unit or a group. The only input
  to the simulation and the only thing a recording stores.
- **Group**: A rule defining membership plus a standing order. Membership is evaluated
  continuously rather than fixed at creation.
- **Contact**: The player's belief about a hostile entity — last observed position, observation
  age, area of uncertainty, and inferred domain and class.
- **Hostile Force**: The rule-driven opponent. Holds territory, expands, and generates pressure in
  proportion to what it holds.
- **Commander**: The player's initial builder, present in every match. Whether its loss ends the
  match is a property of the match rules, not of the unit.
- **Match Rules**: The victory and defeat conditions in effect, chosen when a match starts and
  stored with it. Each condition is an independent rule evaluated against match state.
- **Recording**: A seed, an ordered sequence of orders, the match rules, and the simulation
  behaviour identifier needed to replay them faithfully.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A match with 20,000 units on a 40 x 40 km map runs without the simulation falling
  behind real time on a current consumer machine.
- **SC-002**: A recorded match replays to an identical outcome on Linux, Windows, and macOS, with
  matching state fingerprints at every checkpoint.
- **SC-003**: An observer viewing a static screenshot correctly identifies the domain of every
  visible unit, with no view toggles, filters, or hovering, at least 95% of the time.
- **SC-004**: A player maintains a 2,000-unit force across three separate fronts for ten minutes
  without manually assigning a single replacement unit to a task.
- **SC-005**: Moving one tuning value takes the economy from behaving as a single shared pool to
  halting disconnected builders within seconds, with no code change and no rebuild of game logic.
- **SC-006**: A forward base established by a single transport delivery sustains production for at
  least two minutes with no connection to the player's network.
- **SC-007**: Engineers operating in the wreckage of a large engagement sustain full construction
  rate with no supply connection for at least as long as the engagement itself lasted.
- **SC-008**: Removing any single layer above the core from the build leaves a game that still
  starts, plays, and can be won or lost.
- **SC-009**: A player familiar with the genre completes a first match without consulting
  documentation.
- **SC-010**: When two runs are made to diverge deliberately, the first diverging moment is
  located from the state fingerprints in under five minutes.
- **SC-011**: When a tracked hostile force leaves sensor coverage, the player can still describe
  roughly where it is, and the displayed uncertainty grows visibly the longer it goes unobserved.
- **SC-012**: Switching a match between defeat-on-commander-loss and defeat-on-build-loss changes
  only which condition ends the match, with no other difference observable during play.

## Assumptions

- **Target player**: someone familiar with large-scale real-time strategy, specifically the
  Supreme Commander lineage. First-run comprehension is measured against that audience, not
  against a newcomer to the genre.
- **Single player for this slice**: multiplayer is excluded here, but every requirement in the
  Reproducibility group exists specifically so that multiplayer can be added later without
  reworking the simulation. Excluding it now is a scope decision, not an architectural one.
- **Two resource kinds**: the economy uses two resource kinds, following genre convention, with
  the same local-stock and network rules applied to both.
- **Unit roster for this slice**: roughly twelve unit types, about three per domain, covering
  build, transport, direct fire, and sensing roles, plus the commander. Breadth of roster is
  explicitly not a goal of this slice.
- **The campaign is a later layer, not a variant**: a persistent campaign is built above bounded
  matches, carrying state between them, rather than replacing them. A match is the unit of
  simulation either way, so nothing in this slice is discarded when the campaign is added.
- **Saving and replaying are consequences, not features**: both follow from the reproducibility
  requirements above. Neither needs to be designed separately, and neither is disabled in any
  mode.
- **First tier only**: all units in this slice are first tier. The visual grammar reserves markings
  for higher tiers, but no higher tier exists yet.
- **Aircraft do not refuel or land**: aircraft remain airborne indefinitely, following genre
  convention, unless later design work says otherwise.
- **Terrain is static**: terrain elevation and water extent do not change during a match. Only
  structures, units, and wreckage change.
- **Simulation update rate is fixed and low**, with visuals interpolated between updates. This is
  assumed rather than derived, because both the unit-count target and later multiplayer depend on
  it.
- **Supply routing is automatic**: the player designates where the network should reach, not how
  resources are routed. The player is a commander, not a logistician; if routing becomes a
  micromanagement task, the design has failed.
- **No authored art assets exist or will exist.** This constrains what can be specified as a
  visual requirement and is treated as fixed.

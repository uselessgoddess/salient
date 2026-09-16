# Specification Quality Checklist: Core Playable Slice

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-16
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

All items pass as of iteration 2.

**Iteration 1 fixes:**

- FR-035 named the scale targets directly instead of deferring to "the target unit count".
- FR-036 was reworded from "independent of simulation update rate" to an observable statement
  about visible motion, removing an internal concept from a functional requirement.

**Iteration 2 — clarifications resolved:**

- The original FR-038 conflated two separable things: the commander as a unit and the commander
  as a defeat condition. Split. The commander now always exists as the initial builder (FR-038),
  so match openings are identical under every rule set, and the defeat condition became a
  selectable per-match rule (FR-039) defaulting to loss of build capability (FR-040).
- The original FR-039 posed bounded match versus persistent campaign as alternatives. They are
  not: a campaign is composed of matches, so the match is the unit of simulation either way. The
  slice specifies the bounded match (FR-043); the campaign layer is recorded as a later addition
  in Scope and Assumptions.
- Saving and replaying were added as consequences of the reproducibility requirements (FR-044)
  rather than as separate features, and are explicitly never disabled in any mode.
- Coverage added for the new requirements: User Story 1 scenario 7 (save and resume), User Story 4
  scenarios 5 to 7 (defeat conditions and their persistence through replay), SC-012 (rule swap
  changes nothing but the ending condition), and two new key entities, Commander and Match Rules.

**Iteration 3 — M1 clarifications (2026-09-17):** all 16 items still pass; no item changed state.
Two were strengthened rather than merely re-confirmed:

- *Success criteria are measurable* was passing on a criterion that could not actually be run.
  SC-003 asked for correct domain identification "at least 95% of the time" without naming a
  sample, and the author cannot read a grammar cold. It now names five genre-familiar observers new
  to the project, one static screenshot each, and at most one wrong identification across the whole
  sample.
- *Requirements are testable and unambiguous* was passing on FR-009's "aggregate markings", a term
  used twice and defined nowhere. A formation counter is now an overlay drawn over a concentration
  of force, with individual unit glyphs and their attacks still drawn beneath it at every zoom
  level, which also resolves the apparent conflict with FR-007's ban on mode switching: nothing is
  hidden, so there is nothing to switch.

Three items pass with exceptions worth recording rather than hiding, the same way spec 002 records
its own:

- **"No implementation details leak"** holds except in the Clarifications section, deliberately. The
  contour answer names a shader, a height texture and the marching-squares mesh it replaces, because
  the substance of that answer is that it *supersedes* research decision R10, and a decision cannot
  be recorded as superseded without naming what it supersedes.
- **Map resolution** (2048 x 2048, 19.53 m per cell) sits in Key Entities as a dimension of the data
  model. It is technical, and it stays because FR-006's legibility requirement and the routing
  assumption both rest on it.
- **The 13 ms / 5 ms / 27.5 ms figures** in the routing assumption are measured baselines, present
  for the same reason SC-004 carries one in spec 002: a future regression needs something to be a
  regression against.

Ready for `/speckit-plan`.

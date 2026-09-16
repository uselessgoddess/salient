# Specification Quality Checklist: Steering State as Its Own Subsystem

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-16 | **Revised**: 2026-09-16 (constitution v1.6.0)
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

**Revised 2026-09-16 against constitution v1.6.0.** The original draft required a `SIM_VERSION`
bump and proof that a pre-change recording is refused on load. Constitution 1.6.0 scopes version
discipline to the point where an artifact outlives the build that made it, which this project has
not reached. Removed accordingly: User Story 3, two functional requirements, one success
criterion. Remaining items renumbered contiguously; no gaps, no dangling references. The golden
replay regeneration survives the revision, because that is the part that carries the signal.

Three items passed with qualifications worth recording rather than hiding:

- **"Written for non-technical stakeholders"** is satisfied as far as the subject allows. This
  feature has no end-user-visible change by design — its whole point is that the game plays
  identically afterwards. The audience is whoever extends or deletes a subsystem next. Each user
  story is therefore written in terms of what that person can do afterwards that they could not
  do before, rather than in terms of structure.

- **"No implementation details"** holds with two deliberate exceptions. `SIM_VERSION` appears
  once, in FR-011, to state that it is deliberately *not* bumped and why — recording a decision
  not taken, since a later reader would otherwise assume it was an oversight. The 37 µs figure
  appears because SC-004 is a non-regression criterion and a regression needs a baseline.

- **SC-005** ("adding a second order-driven behaviour requires zero changes to the unit record")
  is the real payoff of this change and cannot be verified until that behaviour is added. It is
  stated as a deferred check rather than dropped, because dropping it would leave the
  specification without its actual justification.

One requirement is unusual and deliberate: **FR-013** forbids keeping the equivalence check as a
permanent test. Proving the refactor preserves behaviour matters exactly once. Afterwards there
is no change it could fail for, and Principle VI forbids keeping a test that cannot fail for a
reason worth fixing.

The sharpest risk in this change is recorded under Edge Cases rather than as a requirement alone:
slot reuse. Unit storage already separates a live unit from a dead one that held the same slot;
steering intent must do the same or a new unit inherits a dead one's destination. FR-003 states
it, and it deserves the plan's attention first.

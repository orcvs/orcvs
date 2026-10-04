# 01 — Retire the Sequence value

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:**

Complete ADR 0063’s removal of Sequence values and their Functions while
keeping ordinary scalar computation and visible multi-root chords working.
The existing scalar form is already available; retain it through the migration
and delete the unused Sequence form only after its callers have been removed.

## Acceptance criteria

- [ ] Remove Range, Note Range, Reverse, Concatenate, Select and Replace and retire every colon-family spelling as unknown.
- [ ] No Function accepts or answers a Sequence, and declarations no longer contain pervasion or Sequence-width metadata.
- [ ] Reservation uses one Atom’s Cell pair for each root value Function; variable-width Sequence projection is removed.
- [ ] Scalar arithmetic, conversion, Tick and feedback Functions, spatial delivery and Terminal Output retain their observable behavior.
- [ ] Several Timed Play roots activated by one Bang produce every expected chord note in deterministic order.
- [ ] No remaining test or doctest constructs a Sequence; replace obsolete coverage with surviving behavior rather than retaining a second value model for tests.
- [ ] The domain glossary and Function reference retire Sequence entries and broadcasting clauses alongside the implementation.
- [ ] Run the affected-crate gates and the workspace pre-pull-request gates before completing this removal.

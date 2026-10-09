# 01 — Retire the Sequence value

Status: resolved
Blocked by: None — can start immediately

**What to build:**

Complete ADR 0063’s removal of Sequence values and their Functions while
keeping ordinary scalar computation and visible multi-root chords working.
The existing scalar form is already available; retain it through the migration
and delete the unused Sequence form only after its callers have been removed.

## Acceptance criteria

- [x] Remove Range, Note Range, Reverse, Concatenate, Select and Replace and retire every colon-family spelling as unknown.
- [x] No Function accepts or answers a Sequence, and declarations no longer contain pervasion or Sequence-width metadata.
- [x] Reservation uses one Atom’s Cell pair for each root value Function; variable-width Sequence projection is removed.
- [x] Scalar arithmetic, conversion, Tick and feedback Functions, spatial delivery and Terminal Output retain their observable behavior.
- [x] Several Timed Play roots activated by one Bang produce every expected chord note in deterministic order.
- [x] No remaining test or doctest constructs a Sequence; replace obsolete coverage with surviving behavior rather than retaining a second value model for tests.
- [x] The domain glossary and Function reference retire Sequence entries and broadcasting clauses alongside the implementation.
- [x] Run the affected-crate gates and the workspace pre-pull-request gates before completing this removal.

## Comments

Resolved by PR #199 (`eca29a15`), which shipped tickets 01 and 02 together. On `main` no `Sequence` reference remains in `lang`, `orcvs` or `console` source, and `one_bang_activates_every_aligned_timed_play_root_as_one_chord` in `orcvs/src/source/tick.rs` covers the chord criterion.

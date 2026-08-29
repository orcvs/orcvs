# 0015: Retire the identity test Function

## Status

Accepted

## Context

`id` was introduced to exercise parsing, nesting, and result commits. It did
not add a musical, temporal, spatial, or Sequence capability to Orcvs. Keeping
a vocabulary entry solely for tests makes the language less deliberate and
uses one of the terse two-Cell names that should carry meaningful behaviour.

## Decision

Remove `id` from the Orcvs Function vocabulary. Tests that used identity as a
convenient wrapper use real arithmetic Expressions instead.

Identity may be reconsidered only if a concrete composition or Sequence use
demonstrates behaviour that pervasive Functions and Portals do not provide.

## Consequences

- Every shipped Function represents user-facing capability rather than test
  scaffolding.
- Parser-capacity and result-commit tests become slightly less compact, but
  exercise the same nested prefix grammar users encounter.
- Existing experimental Source containing `id` no longer parses it as a
  Function.

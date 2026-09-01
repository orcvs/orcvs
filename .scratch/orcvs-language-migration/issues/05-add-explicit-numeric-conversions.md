# 05 — Add explicit Number and Note conversions

**What to build:** Add the numeric conversion Functions `.v` and `.^` from ADR 0021. Both have a
fixed result type and are idempotent for that type; neither enables implicit conversion in another
Function.

**Blocked by:** 01, 04

**Status:** resolved

**Tags:** release/v1

- [x] During evaluation, `.v Number` returns the same Number and `.v Note` returns its underlying MIDI Number.
- [x] During evaluation, `.^ Note` returns the same Note and `.^ Number` converts `00`–`7F` to the corresponding Note.
- [x] `.^` diagnoses Numbers `80`–`FF` and produces no result.
- [x] Both spellings parse and round-trip through `Display` without colliding with another Language Unit.
- [x] Source literal operands are monomorphic (`.v Note`, `.^ Number`); identity applies only to
      already-typed values supplied through evaluation.
- [x] Existing Arithmetic and Play tests prove that no implicit Number/Note coercion remains.

## Downstream integration

- [ ] Sequence broadcasting and atomic failure are owned by
      `.scratch/sequence-values/issues/02-broadcast-atomic-functions-over-sequences.md`.
- [ ] No-coercion coverage for time, feedback, and random Functions is owned by their matching
      tickets under `.scratch/tick-functions/issues/`.

## Comments

The `.` prefix provides the numeric namespace: `v` lowers a Note to its underlying Number and `^`
raises an in-range Number to a Note. Other Function families may reuse either suffix.

This scalar conversion ticket is resolved at the parser and interpreter seams, including
exhaustive MIDI-domain identity/conversion coverage, rejection of `80`–`FF`, nested idempotence,
and strict Number/Note handling in existing Arithmetic and Play Functions. The downstream section
keeps unavailable Sequence, time, feedback, and random integration work visible without making
this ticket depend on the tickets it unblocks. Exhaustive Function-level conversion laws belong to
`.scratch/property-testing/issues/05-exhaustive-arithmetic-and-note-conversion.md`.

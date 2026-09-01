# 05 — Add explicit Number and Note conversions

**What to build:** Add the numeric conversion Functions `.v` and `.^` from ADR 0021. Both have a
fixed result type and are idempotent for that type; neither enables implicit conversion in another
Function.

**Blocked by:** 01, 04

**Status:** ready-for-agent

**Tags:** release/v1

- [x] During evaluation, `.v Number` returns the same Number and `.v Note` returns its underlying MIDI Number.
- [x] During evaluation, `.^ Note` returns the same Note and `.^ Number` converts `00`–`7F` to the corresponding Note.
- [x] `.^` diagnoses Numbers `80`–`FF` and produces no result.
- [ ] Both Functions extend atom-wise across Sequences and return no partial Sequence if one element fails.
- [x] Both spellings parse and round-trip through `Display` without colliding with another Language Unit.
- [x] Source literal operands are monomorphic (`.v Note`, `.^ Number`); identity applies only to
      already-typed values supplied through evaluation.
- [ ] Arithmetic, time, feedback, random, and Play tests prove that no implicit Number/Note coercion remains.

## Comments

The `.` prefix provides the numeric namespace: `v` lowers a Note to its underlying Number and `^`
raises an in-range Number to a Note. Other Function families may reuse either suffix.

The scalar conversion contract is implemented at the parser and interpreter seams, including
exhaustive MIDI-domain identity/conversion coverage, rejection of `80`–`FF`, nested idempotence,
and strict Number/Note handling in existing Arithmetic and Play Functions. Sequence broadcasting
remains blocked on `.scratch/sequence-values/issues/01-add-the-sequence-language-value.md`; time,
feedback, and random Functions likewise do not exist yet, so those two acceptance items remain
open rather than being represented by speculative infrastructure or tests.

Downstream coverage is assigned explicitly: conversion broadcasting and atomic failure belong to
`.scratch/sequence-values/issues/02-broadcast-atomic-functions-over-sequences.md`; Clock, Delay,
Euclidean, feedback, and Random Note-rejection tests belong to their matching tickets under
`.scratch/tick-functions/issues/`; and exhaustive Function-level conversion laws belong to
`.scratch/property-testing/issues/05-exhaustive-arithmetic-and-note-conversion.md`.

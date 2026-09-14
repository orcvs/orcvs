# 07 — Correct the ticket statements flagged in review

**What to build:** Reconcile four ticket statements that a review found to disagree with the decided
design or with the current state of the code.

**Status:** resolved

**Tags:** release/v1

- [x] `v1-roadmap-wayfinding/issues/04` no longer makes the `sequence-values/01` dependency wait on a
      prototype to choose a representation. `^^`, `vv`, `<<`, and `>>` are root-only Source Functions
      with no operand or Sequence behaviour, as CONTEXT.md already states.
- [x] `spatial-tick-planning/issues/03` defines Directional Bang emission against the function's full
      two-Cell Span: the matching two-Cell Self-Banging Function sits in the two Cells
      immediately outside the active Directional Bang Function. The present wording says "adjacent",
      which admits a one-Cell reading.
- [x] `midi-output-family/issues/01` carries `Status: resolved`. Its criteria are all complete and
      the PlayCommand implementation has landed.
- [x] `native-midi/issues/02` states whether the WASM leg of the feature matrix uses default features
      or `--no-default-features`, and requires `wasm32-unknown-unknown` alongside the native targets
      for default features, persistence, `--no-default-features`, and `--no-default-features` with
      persistence.

## Answer

All four review-flagged statements now agree with the decided design and with the tickets they
name.

`04`'s Answer no longer makes `sequence-values/01` or Directional Bang movement wait on a
prototype. `^^`, `vv`, `<<`, and `>>` are root-only Source Functions with no operand or Sequence
behaviour. `map.md` says the same. Item 2 was already edited on `spatial-tick-planning/03` during
the 2026-09-04 alignment. Item 3 was already true of `midi-output-family/01`. Item 4 is recorded
in `native-midi/02`'s "What landed": the WASM leg of `check_wasm` uses default features, compiles
`wasm32-unknown-unknown` for both `orcvs` feature states, and `trunk build`s default and
`--no-default-features`. Native crossings remain the four cells that ticket already lists.

## Comments

Found by a CodeRabbit review of the `01-add-proptest-for-native-targets` branch. None of these files
are in that branch's diff, so correcting them there would have widened the change; they are collected
here instead.

These are ticket-text corrections, not code changes. The `midi-output-family` item is pure
bookkeeping and can be taken alone. The `spatial-tick-planning` item is the one with teeth: the
one-Cell reading of "adjacent" would produce a different emission rule, so settle it before that
ticket is implemented.

### Progress, 2026-09-04

Items 2 and 3 are done, during the `release/v1` issue alignment.

Item 2: line 17 of `spatial-tick-planning/issues/03` said the emission lands "adjacent to itself".
It now uses the words from the Directional Bang Function entry at `CONTEXT.md:68` — "the two Cells
immediately outside its own two-Cell Span" — which admits no one-Cell reading. This edit is made
once, here, as item 2; do not repeat it under section 6 of `v1-release/alignment-changes.md`.

Item 3 was already true when checked: line 9 of `midi-output-family/issues/01` reads
`**Status:** resolved`. The item is bookkeeping that had been done and not ticked.

Items 1 and 4 stay open and still own their edits.

### Progress, 2026-09-15

Items 1 and 4 closed. `04` and `map.md` dropped the prototype wait. `native-midi/02` already
stated the WASM matrix when that ticket resolved; this item ticks that record rather than
rewriting it. `spatial-tick-planning/03` resolved the same day: its leftover ADR 0009 line is
`spatial-tick-planning/09` and is not `release/v1`.

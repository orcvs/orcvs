# 07 — Correct the ticket statements flagged in review

**What to build:** Reconcile four ticket statements that a review found to disagree with the decided
design or with the current state of the code.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `v1-roadmap-wayfinding/issues/04` no longer makes the `sequence-values/01` dependency wait on a
      prototype to choose a representation. `^^`, `vv`, `<<`, and `>>` are root-only Source Functions
      with no operand or Sequence behaviour, as CONTEXT.md already states.
- [ ] `spatial-tick-planning/issues/03` defines Directional Bang emission against the function's full
      two-Cell footprint: the matching two-Cell Self-Banging Function sits in the two Cells
      immediately outside the active Directional Bang Function. The present wording says "adjacent",
      which admits a one-Cell reading.
- [ ] `midi-output-family/issues/01` carries `Status: resolved`. Its criteria are all complete and the
      PlayCommand implementation has landed, but the file still says `ready-for-agent`.
- [ ] `native-midi/issues/02` states whether the WASM leg of the feature matrix uses default features
      or `--no-default-features`, and requires `wasm32-unknown-unknown` alongside the native targets
      for default features, persistence, `--no-default-features`, and `--no-default-features` with
      persistence.

## Comments

Found by a CodeRabbit review of the `01-add-proptest-for-native-targets` branch. None of these files
are in that branch's diff, so correcting them there would have widened the change; they are collected
here instead.

These are ticket-text corrections, not code changes. The `midi-output-family` item is pure
bookkeeping and can be taken alone. The `spatial-tick-planning` item is the one with teeth: the
one-Cell reading of "adjacent" would produce a different emission rule, so settle it before that
ticket is implemented.

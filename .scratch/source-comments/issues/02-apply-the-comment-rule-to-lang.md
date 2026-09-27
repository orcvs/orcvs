# 02 — Apply the comment rule to `lang`

**What to build:** Every comment in `lang/` — source, tests and benches — held to `docs/agents/comments.md`, one sentence at a time. Lineage is removed or reduced to the invariant it was guarding; citations that only record provenance, or that point at a resolved ticket, go. Comments only: no code, test or behaviour changes.

**Blocked by:** 01

**Status:** resolved

- [x] No comment in `lang/` narrates what the code replaced, retired, relocated or used to do, or a rejected alternative, except as the thing a present-tense constraint forbids
- [x] Each remaining `ADR NNNN` or `.scratch/` citation names a constraint the code cannot show, and each `.scratch/` one points at an open ticket
- [x] Every `SAFETY:` comment keeps its invariants, every doctest survives, and every intra-doc link resolves
- [x] The scoped gate from `AGENTS.md` passes for `lang` and `orcvs`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` passes

## Comments

**2026-09-27 — resolved in [#168](https://github.com/orcvs/orcvs/pull/168)** (commit "Hold lang's comments to the source-comment rule"). Lineage removed from `lang/src`, `lang/tests` and `lang/benches`, or reduced to the invariant it guarded. ADR citations that only labelled a concept are dropped: ADR 0016 as the name of the `!` family and of each Play Command, ADR 0012 as the source of the Tick-reading Functions' formulas, orderings and explicit inputs and as the label on the scalar exceptions, ADR 0035's Comment, ADR 0018's recovery, ADR 0015. Kept citations name a constraint the code cannot show: 0001 (a Tick Plan does not interpret musical intent), 0003, 0006 (the Portal offset geometry and how a refused destination is handled), 0007 (a Sequence becomes Source only under the complete-fit rule, with no literal-Sequence reading), 0009, 0012 in two places only (Increment and Interpolation hold no state across Ticks, in `atom.rs`; Euclidean answers zero steps before comparing hits, in `functions/tick.rs`), the ADR 0013 seed layout, 0016 where it forbids a change (Timed Play's length is required though unread, a Pitch Bend is not scaled into fourteen bits, a data byte is not scaled or clamped, and a control's value is named for its domain), 0021, 0025, 0028, 0029, 0032, 0036 and 0039. `.scratch` citations to resolved memory-verification/01, sequence-values/02 and spatial-tick-planning/06 are dropped. The one to open allocation-reduction/02 stays, because the comment states the limitation it tracks. The module comment of `lang/tests/allocation.rs` also names `.scratch/memory-verification/issues/`: that is not a ticket citation but the place a failing allocation assertion is recorded before it is relaxed, the rule `memory-verification/spec.md` states, so it stays while that effort is open. Unchanged: `SAFETY:` comments, doctests, intra-doc links and the measured `broadcast` inlining figures. The scoped gates for `lang` and `orcvs`, the doctests and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` pass. `lang/Cargo.toml` comments are left to source-comments/05.

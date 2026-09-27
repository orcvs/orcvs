# 02 — Apply the comment rule to `lang`

**What to build:** Every comment in `lang/` — source, tests and benches — held to `docs/agents/comments.md`, one sentence at a time. Lineage is removed or reduced to the invariant it was guarding; citations that only record provenance, or that point at a resolved ticket, go. Comments only: no code, test or behaviour changes.

**Blocked by:** 01

**Status:** resolved

- [x] No comment in `lang/` narrates what the code replaced, retired, relocated or used to do, or a rejected alternative, except as the thing a present-tense constraint forbids
- [x] Each remaining `ADR NNNN` or `.scratch/` citation names a constraint the code cannot show, and each `.scratch/` one points at an open ticket
- [x] Every `SAFETY:` comment keeps its invariants, every doctest survives, and every intra-doc link resolves
- [x] The scoped gate from `AGENTS.md` passes for `lang` and `orcvs`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` passes

## Comments

**2026-09-27 — resolved in [#168](https://github.com/orcvs/orcvs/pull/168)** (commit "Hold lang's comments to the source-comment rule"). Lineage removed from `lang/src`, `lang/tests` and `lang/benches`, or reduced to the invariant it guarded. ADR citations that only labelled a concept are dropped (ADR 0016's Plays, ADR 0012's formulas, ADR 0035's Comment, ADR 0018's recovery, ADR 0015). Kept citations name a constraint the code cannot show: 0003, 0009, 0021, 0025, 0028, 0029, 0036, 0039, and the ADR 0013 seed layout. `.scratch` citations to resolved memory-verification/01, sequence-values/02 and spatial-tick-planning/06 are dropped. The one to open allocation-reduction/02 stays, because the comment states the limitation it tracks. Unchanged: `SAFETY:` comments, doctests, intra-doc links and the measured `broadcast` inlining figures. The scoped gates for `lang` and `orcvs`, the doctests and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` pass. `lang/Cargo.toml` comments are left to source-comments/05.

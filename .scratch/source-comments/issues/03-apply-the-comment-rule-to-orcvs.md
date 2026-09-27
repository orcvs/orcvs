# 03 — Apply the comment rule to `orcvs`

**What to build:** Every comment in `orcvs/` — source, tests and benches — held to `docs/agents/comments.md`, one sentence at a time. Lineage is removed or reduced to the invariant it was guarding; citations that only record provenance, or that point at a resolved ticket, go. Comments only: no code, test or behaviour changes.

**Blocked by:** 01

**Status:** resolved

- [x] No comment in `orcvs/` narrates what the code replaced, retired, relocated or used to do, or a rejected alternative, except as the thing a present-tense constraint forbids
- [x] Each remaining `ADR NNNN` or `.scratch/` citation names a constraint the code cannot show, and each `.scratch/` one points at an open ticket
- [x] Every `SAFETY:` comment keeps its invariants, every doctest survives, and every intra-doc link resolves
- [x] The scoped gate from `AGENTS.md` passes for `orcvs` and `console`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` passes

## Comments

**2026-09-27 — resolved in [#171](https://github.com/orcvs/orcvs/pull/171)** (commit "Hold orcvs's comments to the source-comment rule"). Lineage removed from `orcvs/src`, `orcvs/tests` and `orcvs/benches`, or reduced to the invariant it guarded as a present-tense "Do not" constraint. ADR citations that only recorded provenance or labelled a rule the code shows are dropped; kept citations name a constraint the code cannot show (0001, 0002, 0003, 0004, 0006, 0007, 0009, 0012, 0015, 0016, 0020, 0028, 0032, 0034, 0036, 0037, 0041, 0054). Every cited `.scratch` ticket was resolved or wontfix, so each citation is dropped; the `.scratch/memory-verification/issues/` process pointer in `tests/allocation.rs` stays, as in `lang`. Future-tense sentences made false by landed work are restated in the present tense. Unchanged: `SAFETY:` comments, doctests, intra-doc links and measured figures. The scoped gates for `orcvs` and `console`, the doctests and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` pass. `orcvs/Cargo.toml` comments are left to source-comments/05; the dedup, sizing and contradiction sweep stays with source-audit/16.

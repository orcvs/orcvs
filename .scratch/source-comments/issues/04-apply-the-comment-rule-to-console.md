# 04 — Apply the comment rule to `console`

**What to build:** Every comment in `console/` — source, tests and benches — held to `docs/agents/comments.md`, one sentence at a time. Lineage is removed or reduced to the invariant it was guarding; citations that only record provenance, or that point at a resolved ticket, go. Comments only: no code, test or behaviour changes.

**Blocked by:** 01

**Status:** resolved

- [x] No comment in `console/` narrates what the code replaced, retired, relocated or used to do, or a rejected alternative, except as the thing a present-tense constraint forbids
- [x] Each remaining `ADR NNNN` or `.scratch/` citation names a constraint the code cannot show, and each `.scratch/` one points at an open ticket
- [x] Every `SAFETY:` comment keeps its invariants, every doctest survives, and every intra-doc link resolves
- [x] The scoped gate from `AGENTS.md` passes for `console`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` passes

## Comments

**2026-09-27 — resolved in [#175](https://github.com/orcvs/orcvs/pull/175)** (commit "Hold console's comments to the source-comment rule"). Lineage removed from `console/src`, `console/tests` and `console/benches`, or reduced to the invariant it guarded as a present-tense constraint: defect and slice labels, dated retunes and captures, "replaced"/"used to", rejected alternatives. Every `.scratch` ticket citation is dropped: each cited theming, syntax-highlighting, function-reference, source-paint, restyle-egui-console and source-playback-engine ticket is resolved or wontfix, and the one open ticket cited, console-testing/03, was cited as provenance rather than as the limitation it tracks. Citations of `.scratch/theming/schema.md`, the Theme format contract ADR 0053 names as authoritative, stay where they state a format rule. ADR citations fall from 75 to 20 across `console/src`, `console/tests` and `console/benches`; kept ones name a constraint the code cannot show (0008 and 0019, 0038, 0040, 0044/0052, 0045, 0053, 0054). `SAFETY:` comments, doctests and intra-doc links are unchanged. `console/Cargo.toml` comments are left to 05.

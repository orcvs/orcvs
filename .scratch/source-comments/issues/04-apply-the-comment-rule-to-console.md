# 04 — Apply the comment rule to `console`

**What to build:** Every comment in `console/` — source, tests and benches — held to `docs/agents/comments.md`, one sentence at a time. Lineage is removed or reduced to the invariant it was guarding; citations that only record provenance, or that point at a resolved ticket, go. Comments only: no code, test or behaviour changes.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] No comment in `console/` narrates what the code replaced, retired, relocated or used to do, or a rejected alternative, except as the thing a present-tense constraint forbids
- [ ] Each remaining `ADR NNNN` or `.scratch/` citation names a constraint the code cannot show, and each `.scratch/` one points at an open ticket
- [ ] Every `SAFETY:` comment keeps its invariants, every doctest survives, and every intra-doc link resolves
- [ ] The scoped gate from `AGENTS.md` passes for `console`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` passes

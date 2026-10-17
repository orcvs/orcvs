# 01 — A pulse answers whether it fired, and its Bang Output writes it

Status: ready-for-agent

**What to build:** Equality, Delay and Euclidean answer a pulse. Their Bang
Output writes `**` or two spaces on every Turn, root or nested, and a nested
one returns those Cells. The `answers_only_bang()` arm in
`tick/execution.rs` is removed. See `../spec.md`, step 1.

## Acceptance criteria

- [ ] No pulse Function answers `Atom::Empty`.
- [ ] A quiet pulse clears its Output Portal, root or nested, with no
      Function check in `orcvs`.
- [ ] A quiet nested pulse leaves its parent invalid, with the
      unwritten-operand diagnostic.
- [ ] Every other existing tick and lang test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`.

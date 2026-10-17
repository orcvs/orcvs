# 02 — A Copy answers the Cells it read

Status: ready-for-agent

**What to build:** A Copy validates its Input Portal as today and answers the
pair it read, blank included. `orcvs` writes that pair and returns it
verbatim. Both `copies_language_unit()` arms, in `tick/execution.rs` and
`tick/execution/operands.rs`, are removed. See `../spec.md`, step 2.

## Acceptance criteria

- [ ] A Copy of `"  "` answers two blank Cells, not `Atom::Empty`.
- [ ] A Copy writes and returns blank Cells with no Function check in
      `orcvs`.
- [ ] A copied `**` still activates the roots aligned with its destination.
- [ ] Every existing Copy, Track and Bang activation test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`.

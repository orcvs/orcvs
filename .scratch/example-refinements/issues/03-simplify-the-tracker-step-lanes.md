# 03 — Simplify the Tracker's step lanes

Status: ready-for-agent

Each Tracker pattern row is `|| |  |   |  | ... |  | ||`: the Comment's `||`, a space, and a single
`|` marking the first voice's step, and the same on the right. The `||` can mark the step itself:
`||  |   |  | ... |  ||`.

## Acceptance criteria

- [ ] Each pattern row's Comment opens at column 10 and closes at column 63, directly beside the
      first and last voices' steps.
- [ ] The inner `|` marks are unchanged, and every voice reads the same column.
- [ ] `examples/tracker.md` shows the new rows.

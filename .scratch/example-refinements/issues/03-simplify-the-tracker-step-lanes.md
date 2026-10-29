# 03 — Simplify the Tracker's step lanes

Status: resolved

Each Tracker pattern row is `|| |  |   |  | ... |  | ||`: the Comment's `||`, a space, and a single
`|` marking the first voice's step, and the same on the right. The `||` can mark the step itself:
`||  |   |  | ... |  ||`.

## Acceptance criteria

- [x] Each pattern row's Comment opens at column 10 and closes at column 63, directly beside the
      first and last voices' steps.
- [x] The inner `|` marks are unchanged, and every voice reads the same column.
- [x] `examples/tracker.md` shows the new rows.

## Comments

### 2026-10-29 — Resolved

Each pattern row's Comment now runs from column 10 to 64: its `||` sits directly beside channel
`00`'s step at columns 12 and 13 and channel `07`'s at 61 and 62. The Reads still select columns
12, 19, ... 61, and every step is still framed `|  |` to a test, since the Comment's inner `|` is
the step's mark on that side.

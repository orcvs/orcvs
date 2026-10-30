# 01 — Start every example on Row 2

Status: resolved

Every shipped example in `examples/` leaves its first row blank and starts on the second, so the
Source has a row of space above it.

## Acceptance criteria

- [x] Row 0 of every shipped example is empty.
- [x] Every shipped example plays as before, its Playback tests reading each Cell one row lower.
- [x] Each guide's layout diagram and coordinates match.

## Comments

### 2026-10-29 — Resolved

Every shipped example gains an empty first line, so each Cell sits one row lower. Every Read is
relative and every Clock counts Ticks, so nothing plays differently; the Playback, live-editing
and loopback tests read each Cell one row lower, and `every_example_leaves_its_first_row_empty`
keeps row 0 empty.

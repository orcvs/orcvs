# 02 — Stack the 808's instruments

Status: resolved

The 808's kick, snare and hat stand side by side, 38 columns wide, so a phone shows only the kick.
Stack them: each instrument's `|| name ||` label, its machine below it, and a blank row before the
next.

## Acceptance criteria

- [x] The 808 is at most 12 columns wide.
- [x] It plays the same bar, with nothing diagnosed.
- [x] `examples/808.md` shows the stacked layout.

## Comments

### 2026-10-29 — Resolved

Kick, snare and hat stand at rows 1, 5 and 10, each `|| name ||` label over its machine with a
blank row before the next. Each machine keeps its shape, so the snare's Clock still writes into
Equality's `left` and every Delay's `**` still activates its Play.
`the_808_stacks_its_instruments_twelve_columns_wide` keeps the layout, and the bar test still
passes with nothing diagnosed.

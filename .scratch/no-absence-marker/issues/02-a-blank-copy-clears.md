# 02: A Copy of two blank Cells clears its Output Portal

**What to build:** A Copy validates its Input Portal as it does today, then
answers a clear when the pair it read is blank, so the Source tick writes and
returns two spaces with no check for which Function answered. Every other Copy
answers the Language Unit it read, as today. A copied `**`
still activates the roots aligned with its destination. See `../spec.md`,
step 2.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] A Copy of two blank Cells answers a clear, not the Absence Marker.
- [x] A Copy writes and returns blank Cells with no Function check in the
      Source tick.
- [x] A copied `**` still activates the roots aligned with its destination.
- [x] Every existing Copy, Track and Bang activation test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-17: A Copy, and the Reads and Track that read through it, answer `Interpretation::Clear` for two empty Cells. `Function::copies_language_unit` had no other caller, so it is removed. `empty_input_answers_a_clear` pins the answer.

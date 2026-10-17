# 02: A Copy of two blank Cells clears its Output Portal

**What to build:** A Copy validates its Input Portal as it does today, then
answers a clear when the pair it read is blank, so the Source tick writes and
returns two spaces with no check for which Function answered. Every other Copy
answers the Language Unit it read, as today. A copied `**`
still activates the roots aligned with its destination. See `../spec.md`,
step 2.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] A Copy of two blank Cells answers a clear, not the Absence Marker.
- [ ] A Copy writes and returns blank Cells with no Function check in the
      Source tick.
- [ ] A copied `**` still activates the roots aligned with its destination.
- [ ] Every existing Copy, Track and Bang activation test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`.

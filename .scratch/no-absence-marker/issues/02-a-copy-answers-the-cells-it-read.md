# 02: A Copy answers the Cells it read

**What to build:** A Copy validates its Input Portal as it does today, then
answers the pair it read, blank included. The Source tick writes that pair and
returns it verbatim, with no check for which Function answered. A copied `**`
still activates the roots aligned with its destination. See `../spec.md`,
step 2.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] A Copy of two blank Cells answers those Cells, not the Absence Marker.
- [ ] A Copy writes and returns blank Cells with no Function check in the
      Source tick.
- [ ] A copied `**` still activates the roots aligned with its destination.
- [ ] Every existing Copy, Track and Bang activation test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`.

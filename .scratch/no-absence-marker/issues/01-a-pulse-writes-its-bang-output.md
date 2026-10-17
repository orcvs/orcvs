# 01: A pulse writes its Bang Output

**What to build:** Equality, Delay and Euclidean answer whether they Banged.
Their Bang Output writes `**` on a Tick they Bang and clears its two Cells on
a Tick they don't, whether the Function is a root or nested, and nothing in
the Source tick checks which Function answered. A nested pulse returns the
Cells its Output wrote. So on a Tick it doesn't Bang, its parent's operand is
unwritten, and the parent is diagnosed as invalid input. Toby confirmed this
as expected on 2026-10-17. See `../spec.md`, step 1.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] No pulse Function answers the Absence Marker.
- [ ] A pulse that doesn't Bang clears its Output Portal, root or nested,
      with no Function check in the Source tick.
- [ ] A Write whose value is an unequal Equality is diagnosed with the
      unwritten-operand message and writes nothing.
- [ ] Every other existing test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`.

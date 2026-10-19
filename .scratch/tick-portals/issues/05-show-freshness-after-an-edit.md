# 05: Show Tick Portals' freshness after an edit

**What to build:** After an edit the last record is kept but marked retained,
and it paints only the entries whose owner Function still stands at its anchor.
Editing a paused tracker keeps the other voices' playheads lit, and a Function
the edit changed stops claiming accesses it no longer makes. See `../spec.md`
section 5.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] A Source revision answers its record as current, or as retained when an
      edit has landed since the commit.
- [ ] An older Source revision keeps the record it was read with.
- [ ] A newly opened or loaded Source has no record until its first Tick.
- [ ] A retained record paints only entries whose owner still stands.
- [ ] ADR 0077 states the freshness contract.
- [ ] Tests for each of the above.

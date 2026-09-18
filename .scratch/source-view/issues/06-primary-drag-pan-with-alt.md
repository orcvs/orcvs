# 06: Primary-drag Pan with Alt

**What to build:** Holding Alt (Option on macOS) and dragging with the primary button Pans the Source View, so a trackpad with no middle button can Pan by dragging. Space was considered and refused: it is Source input. See ADR 0045.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Alt with a primary drag Pans by exactly what the pointer moved, bounded by the Grid's edges.
- [ ] A primary click alone, and a primary drag without Alt, still select a Cell and do not Pan.
- [ ] Alt with a primary drag writes nothing to the Source.

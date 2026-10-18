# 02: Commit Tick Portals with the Tick and expose their freshness

**What to build:** Carry the record through `PlannedTick` and `Source::execute`
into the one `Source::commit_tick`, store it with the revision the commit
leaves, and answer it from `SourceRevision::tick_portals()` as `Current` or
`Retained` (spec sections 4-5). ADR 0077 and the `CONTEXT.md` term.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] One commit path stores the plan's writes and the record together.
- [ ] Tests: a refused plan publishes no record and the locked fallback does;
      an empty Tick replaces the record; an edit makes it `Retained`; an older
      `SourceRevision` keeps its own; a new Source has none.
- [ ] ADR 0077; `CONTEXT.md` defines Tick Portals.

# 01: Record Tick Portals during execution

**What to build:** `TickPortals`, `PortalUse` and their types (spec sections
1-3), recorded at the execution funnels in `orcvs/src/source/tick/execution.rs`
and returned from `tick::plan` beside the Tick Plan.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `InputSpan` and `OutputSpan` with private constructors and `span()`.
- [ ] `PortalError` promoted to `pub`; `WriteRefusal` and `BundleRefusal` cover
      every refusal named in the spec.
- [ ] Reads recorded after the writer wait; writes for every delivered site;
      locks with their verdict; bundle parts in order.
- [ ] Owner anchor and executed Function captured from the schedule and `Turns`.
- [ ] Tests: writer-wait retry leaves one read; refused writes and bundle parts
      keep their reasons; order and owners hold when the Tick rewrites Source.

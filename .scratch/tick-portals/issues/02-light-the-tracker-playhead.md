# 02: Light the tracker playhead

**What to build:** The tracer bullet. Each Turn's read is recorded as Tick
Portals and committed with the Tick, and the Cells it read paint as Input
Portal. Playing the Orca tracker example lights each voice's `&v` read as it
steps down its column, as Orca lights its ports. See `../spec.md` sections 1-4
and 6.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] A read is recorded once the Turn passes the writer wait, with its owner
      (anchor and executed Function, captured during execution), its origin
      (Declared or Selected), and its site (a resolved Input Span, or the Portal
      Error). A resolved site means a site was read, not that a valid value was.
- [ ] Input Spans can be made only from the Input Portal a Turn resolved.
- [ ] The Tick Plan and its Tick Portals are planned together and committed
      through the one commit path on both the optimistic and the locked
      attempt. A refused attempt publishes neither, and a committed Tick that
      read nothing replaces the previous record with an empty one.
- [ ] A Source revision answers the record committed with it, and the Render
      Frame folds its resolved reads into the Cells as Input Portal without a
      new allocation per frame, pinned by an allocation assertion.
- [ ] Tests: a Turn that waits for a writer and is retaken leaves one read; a
      refused plan publishes no record and the locked attempt does; an empty
      Tick replaces the record; a kittest shows the tracker's reads lit as it
      steps.
- [ ] ADR 0077 records the record and which events belong; `CONTEXT.md`
      defines Tick Portals as an observation of a committed Tick, not a Portal.

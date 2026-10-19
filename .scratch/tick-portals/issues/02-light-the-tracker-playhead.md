# 02: Light the tracker playhead

**What to build:** The tracer bullet. Each Turn's read is recorded as Tick
Portals and committed with the Tick, and the Cells it read paint as Input
Portal. Playing the Orca tracker example lights each voice's `&v` read as it
steps down its column, as Orca lights its ports. See `../spec.md` sections 1-4
and 6.

**Blocked by:** 01

**Status:** resolved

- [x] A read is recorded once the Turn passes the writer wait, with its owner
      (anchor and executed Function, captured during execution), its origin
      (Declared or Selected), and its site (a resolved Input Span, or the Portal
      Error). A resolved site means a site was read, not that a valid value was.
- [x] Input Spans can be made only from the Input Portal a Turn resolved.
- [x] The Tick Plan and its Tick Portals are planned together and committed
      through the one commit path on both the optimistic and the locked
      attempt. A refused attempt publishes neither, and a committed Tick that
      read nothing replaces the previous record with an empty one.
- [x] A Source revision answers the record committed with it, and the Render
      Frame folds its resolved reads into the Cells as Input Portal without a
      new allocation per frame, pinned by an allocation assertion.
- [x] Tests: a Turn that waits for a writer and is retaken leaves one read; a
      refused plan publishes no record and the locked attempt does; an empty
      Tick replaces the record; a kittest shows the tracker's reads lit as it
      steps.
- [x] ADR 0077 records the record and which events belong; `CONTEXT.md`
      defines Tick Portals as an observation of a committed Tick, not a Portal.

## Comments

2026-10-19 — `tick::plan` answers the Tick Plan, its `TickPortals` and the Turn
states together; `PlannedTick` carries the record, and both the optimistic
commit and `Source::execute` hand plan and record to `Source::commit_tick`,
which stores the record with the revision the commit left. `SourceRevision`
carries the record only while the Source is still at that revision, so until
ticket 05 a revision an edit made answers none rather than a stale record.
`PortalEvent` holds `Read { origin, site }` alone; tickets 03-04 add `Write`,
`Lock` and `Bundle` beside it. A read is recorded after the writer wait, beside
the test-only `interpret` record, so a retaken Turn records once. The Render
Frame sets the Input role in place over the Cell vector it has already built,
rather than inside the per-Cell closure, so the fold is one pass over the
record's Spans and allocates nothing (16 blocks against 15 when the fold was
made to collect). The kittest drives one tracker voice, `.+02~.0104` over `&v`,
through four Ticks taken with `SourceCommander::execute` while Playback is
stopped, and reads the painted `input_portal.background` runs.

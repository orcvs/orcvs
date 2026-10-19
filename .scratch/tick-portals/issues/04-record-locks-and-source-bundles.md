# 04: Record locks and Source Bundles

**What to build:** Tick Portals hold every Halt lock with its verdict and every
Source Bundle part in the order it ran, so the record is the complete list of a
Tick's Portal accesses. Locks and bundles are recorded only; they do not paint.
See `../spec.md` section 1.

**Blocked by:** 02

**Status:** resolved

- [x] A lock is recorded with its site and verdict.
- [x] Bundle parts are recorded in order: vacating the producer's Span, placing
      at the destination, leaving a Bang Function after a refused move, and a
      Bang Function's clear.
- [x] A refused destination keeps its reason: geometric, with the attempted
      site where one exists, or occupied.
- [x] Tests: a mover's vacate and place, a blocked move leaving `**`, a refused
      emission, a Bang Function's clear, a lock and a missed lock.

## Comments

2026-10-19 — `PortalEvent` gains `Lock { site, verdict }` and
`Bundle { part, outcome }`. `lock_portal` maps `dependencies::Locked` to the
public `LockVerdict` (`Locked(root)`, `Missed(root)`, `OrderingDefect`,
`NotARoot`, `Unchanged`; `OrderAfter` is the schedule phase's answer and
reads as `Unchanged`) and records it with `PortalAccess::lock_site()`, so an
off-Grid site keeps its `PortalError`. `deliver_source_effect` records
`Vacate`, then `Place`, as a move writes them; a refused move records `Place`
as `Err(BundleRefusal)` and then `LeftBang`; a refused emission records `Place`
alone; `fire` records `Fire` with the cleared Span. `BundleRefusal` is
`Portal { error, at }`, with `at` the destination's Position when the
displacement resolved and admission at it did not, or `Occupied(OutputSpan)`
when the destination was admitted and not vacant. The fold skips locks and
bundles, so neither paints. The ticket-03 mover test now holds the mover's two
bundle parts ahead of the reads and writes that follow it. Every test went red
on its own assertion before the recording landed.

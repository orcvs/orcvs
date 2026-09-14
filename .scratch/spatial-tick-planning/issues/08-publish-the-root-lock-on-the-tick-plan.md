# 08 — Publish the root lock on the Tick Plan

**What to build:** A successful Halt lock is a named Effect on the Tick Plan: it withholds the
Expression root in the row below the Halt, and a reader of the plan can see that lock without
inferring it from an unchanged Grid. Empty target stays a no-op. Occupied non-root still diagnoses
and invents no lock. Commit still writes no Cell for a lock. The lock site is the row below, the
same Portal Halt already names — not south, and not an ordinary result.

**Blocked by:** None — 05 is resolved; this is the Tick Plan interface that ticket left unpublished.

**Status:** resolved

- [x] A Tick whose Halt locks a root publishes a lock Effect that names that root.
- [x] An empty target publishes no lock and no diagnostic.
- [x] An occupied non-root publishes a diagnostic and no lock.
- [x] A suppressed Halt still publishes no lock of its own.
- [x] Commit leaves Source unchanged for a lock; the withheld root does not take its Turn.
- [x] Halt tests assert the plan's lock Effect. They do not treat "the Grid looks the same" as
      proof the lock ran.
- [x] Stating a lock as a Tick outcome is a first-class plan, not a panic.

# 04: Record locks and Source Bundles

**What to build:** Tick Portals hold every Halt lock with its verdict and every
Source Bundle part in the order it ran, so the record is the complete list of a
Tick's Portal accesses. Locks and bundles are recorded only; they do not paint.
See `../spec.md` section 1.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] A lock is recorded with its site and verdict.
- [ ] Bundle parts are recorded in order: vacating the producer's Span, placing
      at the destination, leaving a Bang Function after a refused move, and a
      Bang Function's clear.
- [ ] A refused destination keeps its reason: geometric, with the attempted
      site where one exists, or occupied.
- [ ] Tests: a mover's vacate and place, a blocked move leaving `**`, a refused
      emission, a Bang Function's clear, a lock and a missed lock.

# 05 — Retire the untyped editing seam

**What to build:** Every caller addresses Cells with a Grid-minted index, and the number-taking
forms are gone. With them goes the out-of-range error: a caller can no longer name a Cell the Grid
does not have, so the state that error described stops being reachable.

**Blocked by:** 04 — Offer the editing seam a typed Cell index.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The console, persistence and every test address Cells with a typed index.
- [ ] The number-taking editing forms are deleted.
- [ ] The out-of-range error variant and the check that produced it are deleted.
- [ ] The two tests that asserted that error are gone, not rewritten — they described a state that
      can no longer occur.
- [ ] The Source still refuses content it cannot store, which is a separate rule and keeps its own
      error.

## Comments

Migrate in batches if the single commit is unwieldy — production callers first, then tests — since
the expand step keeps each batch green.

The out-of-range variant has one construction site and two assertions, both tests. Every production
caller derives its index from a Position the Grid minted, so it cannot fire in production today; it
exists to describe a state the type system already forbids one frame earlier.

# 08 — A Tick still lands while a destination change is processed

**What to build:** A performer changing MIDI destination mid-run can trust that Ticks keep landing
on time while the engine's task processes the installation request. The suite states this guarantee
explicitly, replacing the old design's implicit hope that enumeration is fast enough not to miss a
deadline.

**Blocked by:** None — 03 landed; the install path exists.

**Status:** ready-for-agent

- [ ] A test starts a running Orcvs with playback active, queues a destination installation, and
      asserts that a Tick still lands on schedule while the installation is processed.
- [ ] The test crosses the production selection and Playback interfaces rather than reaching into
      private engine state.
- [ ] The test does not sleep, pump a run loop, or depend on how quickly a queue drains; it asserts
      timing behaviour directly.

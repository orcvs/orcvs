# 07 — ADR 0041 records Install instead of Discover and Select

**What to build:** A maintainer reading ADR 0041 sees the selection model the codebase actually
ships: the console opens the port, and Playback receives an already-open connection on the ordered
queue as an `Install` request. The ADR no longer describes discovery and `Select(destination)`
requests that no longer exist.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] ADR 0041 describes installation of an already-open connection rather than discovery and
      selection requests processed inside the engine's task.
- [x] The ADR still states that the engine's task remains the sole owner of the connection, note
      schedule, and lifecycle state once the connection is installed.
- [x] The ADR still records that selection crosses the seam as data on the ordered queue, not as
      transitions supplied by callers.
- [x] No other ADR or domain doc contradicts the console-opens / Playback-installs boundary without
      an explicit decision recorded elsewhere.

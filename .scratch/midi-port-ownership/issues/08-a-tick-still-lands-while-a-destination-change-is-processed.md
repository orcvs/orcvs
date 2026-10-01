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

## Comments

### Independent implementation audit — 2026-09-29

Still open. Installation runs inside the Tick-owning actor (`orcvs/src/playback.rs:1514`),
and the old connection's safety reset calls synchronous `MidiConnection::send` before replacement
(`orcvs/src/midi.rs:138-146`). An arbitrarily slow send can therefore delay a Tick. Define the
bounded fake-connection timing scenario before writing the assertion; if the intended guarantee
includes arbitrarily slow installation, that requires a design decision, not just another test.
`orcvs/tests/midi_selection_handle.rs:106` installs before playback starts. Existing reconnect
tests at `orcvs/src/midi.rs:953,1032` prove eventual output and schedule clearing, not an exact
deadline across a mid-run installation. Four selection integration tests passed in this audit.

### Issue audit against d3fd1b27 — 2026-10-01

Still open; no test asserts a Tick landing across a mid-run installation. References in the
2026-09-29 comment moved: `orcvs/src/midi.rs:172-183` (`install_connection`, safety reset at
`:174-178`), `:999` and `:1079`; `orcvs/tests/midi_selection_handle.rs:117`. `playback.rs:1514`
holds. Pin the bounded fake-connection timing scenario before an agent picks this up.

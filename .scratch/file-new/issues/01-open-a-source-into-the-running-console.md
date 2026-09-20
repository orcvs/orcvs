# 01 — Open a Source into the running console

**What to build:** Loading the Function reference goes through one named Open operation, so the
console has a single answer to "replace the environment with this Source" rather than one sequence
written out at its only call site. Nothing a viewer can see changes.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

This is the prefactor. `02` adds a second caller, and without this it would be a second copy of the
same four-step sequence — build the Orcvs, rebuild MIDI device selection over its new handle, rest
the Source View, report a failure — with the two free to drift apart.

- [ ] One operation takes a Source and opens it: it replaces the running Orcvs, rebuilds MIDI
      device selection over the new Orcvs's handle, and rests the Source View. A failure to build
      the Orcvs is reported and leaves the console on the Source it already had.
- [ ] `File > Load Function reference` calls it and behaves exactly as it does today, including
      that a previously selected MIDI destination does not carry over and that Bpm returns to its
      default — both existing consequences of replacing the whole Orcvs, neither introduced here.
- [ ] Theme, Cursor effects, Source colours and Diagnostics visibility are untouched by an Open,
      and a test says so rather than leaving it to the reader.
- [ ] The operation names what it does in the console's own vocabulary and its documentation says
      which facts an Open replaces and which it leaves standing.
- [ ] No behaviour changes. Every existing console test passes unchanged.
- [ ] The scoped gates for `console` pass.

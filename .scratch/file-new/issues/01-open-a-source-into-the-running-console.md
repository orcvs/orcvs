# 01 — Open a Source into the running console

**What to build:** Loading the Function reference goes through one named Open operation, so the
console has a single answer to "replace the environment with this Source" rather than one sequence
written out at its only call site. Nothing a viewer can see changes.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

This is the prefactor. `02` adds a second caller, and without this it would be a second copy of the
same four-step sequence — build the Orcvs, rebuild MIDI device selection over its new handle, rest
the Source View, report a failure — with the two free to drift apart.

- [x] One operation takes a Source and opens it: it replaces the running Orcvs, rebuilds MIDI
      device selection over the new Orcvs's handle, and rests the Source View. A failure to build
      the Orcvs is reported and leaves the console on the Source it already had.
- [x] `File > Load Function reference` calls it and behaves exactly as it does today, including
      that a previously selected MIDI destination does not carry over and that Bpm returns to its
      default — both existing consequences of replacing the whole Orcvs, neither introduced here.
- [x] Theme, Cursor effects, Source colours and Diagnostics visibility are untouched by an Open,
      and a test says so rather than leaving it to the reader.
- [x] The operation names what it does in the console's own vocabulary and its documentation says
      which facts an Open replaces and which it leaves standing.
- [x] No behaviour changes. Every existing console test passes unchanged.
- [x] The scoped gates for `console` pass.

## Comments

`Console::open(&mut self, source: Source)` is the operation, and `environment` beside it is the
pair of steps it shares with `Console::new` — build the Orcvs, wake the Panel on its Playback
Engine's publications, and rebuild MIDI device selection over its handle — so the constructor and
the Open cannot drift either. `load_function_reference` is now one line over `open`.

Two deliberate differences from the sequence this replaced, neither visible to a viewer:

- The Panel wake is started for the new Orcvs. `load_function_reference` never started one, so
  after a load the Panel repainted on a Tick only because the Run Clock and the Cursor Effect
  already asked for frames; the old wake loop ended when the replaced engine's watch closed. The
  console now holds the `egui::Context` it was created in so an Open can start the wake without a
  new parameter.
- The failure report reads "failed to open a Source" rather than naming the Function reference,
  because the Open no longer knows which caller asked. It reaches the developer console only.

Tests: `kittest_tests::an_open_leaves_every_setting_standing` (Theme selection and presentation,
Cursor effects, Diagnostics visibility) and
`kittest_tests::an_open_that_cannot_start_leaves_the_running_source_standing` (an Open asked for
outside a runtime, the one way the Orcvs build fails). Every existing console test passes
unchanged, `storage_tests::loading_the_function_reference_replaces_the_source_and_its_grid`
included.

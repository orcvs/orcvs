# 02 — Output-only Playback rejects a MIDI adapter on every public constructor

**What to build:** A caller who wires a MIDI adapter into output-only Playback is stopped by the
compiler wherever they try it, instead of receiving a Playback Engine whose MIDI output can never
be discovered, selected or connected — silent output with no error and no diagnostic. The
restriction that already guards application-level construction guards engine-level construction
too, so there is no public path left that accepts the adapter and quietly does nothing with it.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Constructing a Playback Engine directly with a MIDI output adapter fails to compile, on the
      same terms as constructing an output-only running Orcvs with one.
- [x] Every public constructor that produces output-only Playback carries the restriction; no
      public path accepts an adapter it cannot publish for.
- [x] A compile-fail test covers the engine constructor, extending the existing compile-fail
      coverage that proves an output-only running Orcvs exposes no selection handle.
- [x] MIDI-specific construction is unaffected: it still accepts the MIDI adapter and still returns
      the lifecycle handle and the restricted selection handle together.
- [x] Existing output-only construction with non-MIDI adapters compiles unchanged, including the
      in-memory adapter used across the test suite and the benches.

# 05: The shipped examples play with no diagnostics

**What to build:** The shipped Melody, Tracker and Knobs examples play with no diagnostics on their empty steps, and the documentation describes the optional note. See `../spec.md`.

**Blocked by:** 03, 04

**Status:** resolved

- [x] The shipped tracker and knobs tests in playback assert no diagnostics on empty steps, and the exact Language Map check that #376 dropped is restored.
- [x] The Melody written-out test no longer expects `expected a note` diagnostics on its blank steps.
- [x] The console's live-editing kittest shows a note being typed going from diagnosed to playing, with no silent undiagnosed state in between.
- [x] The Function reference documents each Play's note as optional and says a half-written note is an error.
- [x] CONTEXT.md describes the optional note in existing vocabulary. "Rest" and "silent Play" stay in the Play Function's Avoid list, and no new term is added.
- [x] The Melody, Tracker, Knobs and Sequencer example docs match the new behaviour.
- [x] `mise run site_build` and `mise run site_test` pass, since the site renders the Function reference.
